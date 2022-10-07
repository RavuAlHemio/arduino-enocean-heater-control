mod xml_magic;


use std::collections::BTreeMap;
use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Read, Write};
use std::num::ParseIntError;
use std::path::PathBuf;

use clap::Parser;
use reqwest;
use serde::{Deserialize, Serialize};
use sxd_document::Package;

use crate::xml_magic::{DocumentExt, ElementExt, copy_element_children};


#[derive(Parser)]
struct Opts {
    pub output_file: PathBuf,

    #[arg(default_value = "http://tools.enocean-alliance.org/EEPViewer/")]
    pub base_url: String,

    #[arg(short, long)]
    pub cache_path: Option<PathBuf>,
}


#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
struct TelegramType {
    pub name: String,
    #[serde(rename = "function")] pub functions: FunctionOrFunctions,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(untagged)]
enum FunctionOrFunctions {
    Function(Function),
    Functions(Vec<Function>),
}
impl FunctionOrFunctions {
    pub fn into_vec(self) -> Vec<Function> {
        match self {
            FunctionOrFunctions::Function(t) => vec![t],
            FunctionOrFunctions::Functions(ts) => ts,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
struct Function {
    pub value: String,
    pub name: String,
    #[serde(rename = "type")] pub types: TypeOrTypes,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(untagged)]
enum TypeOrTypes {
    Type(Type),
    Types(Vec<Type>),
}
impl TypeOrTypes {
    fn into_vec(self) -> Vec<Type> {
        match self {
            TypeOrTypes::Type(t) => vec![t],
            TypeOrTypes::Types(ts) => ts,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
struct Type {
    pub value: String,
    pub name: String,
}

fn u8_from_hex_0x(prefixed_hex_string: &str, string_type: &str) -> u8 {
    if let Some(hex_string) = prefixed_hex_string.strip_prefix("0x") {
        match u8::from_str_radix(hex_string, 16) {
            Ok(v) => v,
            Err(e) => panic!(
                "failed to parse {} {:?} (originally {:?}) as hex string: {}",
                string_type, hex_string, prefixed_hex_string, e,
            ),
        }
    } else {
        panic!("failed to strip 0x prefix of {} {:?}", string_type, prefixed_hex_string);
    }
}

fn u8_from_hex(hex_string: &str, string_type: &str) -> u8 {
    match u8::from_str_radix(hex_string, 16) {
        Ok(v) => v,
        Err(e) => panic!(
            "failed to parse {} {:?} as hex string: {}",
            string_type, hex_string, e,
        ),
    }
}

trait StrExt {
    fn parse_hex_u8(&self) -> Result<u8, ParseIntError>;
}
impl StrExt for &str {
    fn parse_hex_u8(&self) -> Result<u8, ParseIntError> {
        u8::from_str_radix(self, 16)
    }
}

async fn get_bytes_from_url(url: &str) -> Vec<u8> {
    if let Some(path) = url.strip_prefix("file://") {
        let mut file = File::open(path)
            .expect("failed to open file");
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)
            .expect("failed to read file");
        buf
    } else {
        // reqwest it instead
        let response = reqwest::get(url)
            .await.expect("failed to request URL");
        if response.status().as_u16() >= 300 {
            panic!("requesting URL {:?} failed with HTTP error code {}", url, response.status());
        }
        let bytes = response.bytes()
            .await.expect("failed to obtain bytes from HTTP response");
        bytes.into()
    }
}

async fn get_string_from_url(url: &str) -> String {
    let bytes = get_bytes_from_url(url).await;
    String::from_utf8(bytes)
        .expect("failed to decode obtained bytes as UTF-8")
}


#[tokio::main]
async fn main() {
    let opts = Opts::parse();

    let eep_types_url = format!("{}/eep-viewer-desc.json", opts.base_url);
    let eep_types_string = get_string_from_url(&eep_types_url)
        .await;
    let eep_types: BTreeMap<String, TelegramType> = serde_json::from_str(&eep_types_string)
        .expect("failed to parse EEP types as JSON");

    // prepare the merged document
    let merged_doc_package = Package::new();
    let merged_doc = merged_doc_package.as_document();

    let eep_elem = merged_doc.create_element("eep");
    merged_doc.root().append_child(eep_elem);

    let profile_elem = merged_doc.create_element("profile");
    eep_elem.append_child(profile_elem);

    for (telegram_code_string, telegram_type) in eep_types {
        let telegram_code = u8_from_hex(&telegram_code_string, "telegram code");

        let rorg_elem = merged_doc.create_element("rorg");
        profile_elem.append_child(rorg_elem);

        let number_elem = merged_doc.create_element("number");
        number_elem.set_text(&format!("0x{:02X}", telegram_code));
        rorg_elem.append_child(number_elem);

        let title_elem = merged_doc.create_element("title");
        title_elem.set_text(&telegram_type.name);
        rorg_elem.append_child(title_elem);

        for function in telegram_type.functions.into_vec() {
            let function_code = u8_from_hex_0x(&function.value, "function code");

            let func_elem = merged_doc.create_element("func");
            rorg_elem.append_child(func_elem);

            let number_elem = merged_doc.create_element("number");
            number_elem.set_text(&format!("0x{:02X}", function_code));
            func_elem.append_child(number_elem);

            let title_elem = merged_doc.create_element("title");
            title_elem.set_text(&function.name);
            func_elem.append_child(title_elem);

            for ty in function.types.into_vec() {
                let type_code = u8_from_hex_0x(&ty.value, "type code");

                // obtain the XML file for this EEP
                let eep_url = format!(
                    "{0}/profiles/{1:02X}/{2:02X}/{3:02X}/{1:02X}-{2:02X}-{3:02X}.xml",
                    opts.base_url,
                    telegram_code,
                    function_code,
                    type_code,
                );
                eprintln!("crunching {}", eep_url);
                let eep_xml_bytes = get_bytes_from_url(&eep_url)
                    .await;
                if let Some(cache_path) = &opts.cache_path {
                    let mut this_cache_path = cache_path.clone();
                    this_cache_path.push("profiles");
                    this_cache_path.push(format!("{:02X}", telegram_code));
                    this_cache_path.push(format!("{:02X}", function_code));
                    this_cache_path.push(format!("{:02X}", type_code));
                    this_cache_path.push(format!(
                        "{:02X}-{:02X}-{:02X}.xml",
                        telegram_code, function_code, type_code,
                    ));

                    // ensure the directories exist
                    create_dir_all(this_cache_path.parent().unwrap())
                        .expect("failed to create directory for XML");
                    let mut cache_file = File::create(&this_cache_path)
                        .expect("failed to create XML cache");
                    cache_file.write_all(&eep_xml_bytes)
                        .expect("failed to write XML cache");
                }

                let eep_xml_string = match String::from_utf8(eep_xml_bytes.clone()) {
                    Ok(s) => s,
                    Err(_) => {
                        eprintln!("  failed to parse as UTF-8; trying as ISO-8859-1...");
                        eep_xml_bytes.iter()
                            .map(|b| char::from_u32((*b).into()).unwrap())
                            .collect()
                    },
                };
                let eep_xml_package = sxd_document::parser::parse(&eep_xml_string)
                    .expect("failed to parse EEP XML");

                // go down to type level, verifying that we have the correct type
                let eep_rorg_elem = eep_xml_package
                    .as_document()
                    .root_element().expect("no root element")
                    .first_element_child_with_tag_name("profile").expect("no profile element")
                    .first_element_child_with_tag_name("rorg").expect("no rorg element");
                let rorg_tag = eep_rorg_elem
                    .first_element_child_with_tag_name("number").expect("no rorg/number element")
                    .first_text_child().expect("rorg/number element has no text")
                    .strip_prefix("0x").expect("rorg/number element does not start with 0x")
                    .parse_hex_u8().expect("failed to parse rorg/number element as u8");
                if rorg_tag != telegram_code {
                    eprintln!(
                        "{} defines invalid telegram code {:02X} (expected {:02X}); skipping",
                        eep_url,
                        rorg_tag,
                        telegram_code,
                    );
                    continue;
                }
                let eep_func_elem = eep_rorg_elem
                    .first_element_child_with_tag_name("func").expect("no func element");
                let func_tag = eep_func_elem
                    .first_element_child_with_tag_name("number").expect("no func/number element")
                    .first_text_child().expect("func/number element has no text")
                    .strip_prefix("0x").expect("func/number element does not start with 0x")
                    .parse_hex_u8().expect("failed to parse func/number element as u8");
                if func_tag != function_code {
                    eprintln!(
                        "{} defines invalid function code {:02X}-{:02X} (expected {:02X}-{:02X}); skipping",
                        eep_url,
                        rorg_tag, func_tag,
                        telegram_code, function_code,
                    );
                    continue;
                }
                let eep_type_elem = eep_func_elem
                    .first_element_child_with_tag_name("type").expect("no type element");
                let type_tag = eep_type_elem
                    .first_element_child_with_tag_name("number").expect("no type/number element")
                    .first_text_child().expect("type/number element has no text")
                    .strip_prefix("0x").expect("type/number element does not start with 0x")
                    .parse_hex_u8().expect("failed to parse type/number element as u8");
                if type_tag != type_code {
                    eprintln!(
                        "{} defines invalid type code {:02X}-{:02X}-{:02X} (expected {:02X}-{:02X}-{:02X}); skipping",
                        eep_url,
                        rorg_tag, func_tag, type_tag,
                        telegram_code, function_code, type_code,
                    );
                    continue;
                }

                // only now that there is no risk of skipping do we create the new element
                let type_elem = merged_doc.create_element("type");
                func_elem.append_child(type_elem);

                // copy all that
                copy_element_children(&eep_type_elem, &type_elem);
            }
        }
    }

    eprintln!("serializing...");

    // serialize the resulting XML as UTF-16
    {
        let mut out_buf = Vec::new();
        {
            let writer = sxd_document::writer::Writer::new()
                .set_single_quotes(false);
            writer.format_document(&merged_doc, &mut out_buf)
                .expect("failed to write document");
        }

        let out_string = String::from_utf8(out_buf)
            .expect("serialized document was not valid UTF-8");

        let out_file = File::create(&opts.output_file)
            .expect("failed to create output file");
        let mut out_file_buf = BufWriter::new(out_file);
        for word in out_string.encode_utf16() {
            out_file_buf.write_all(&word.to_le_bytes())
                .expect("failed to write into output file")
        }
    }
}
