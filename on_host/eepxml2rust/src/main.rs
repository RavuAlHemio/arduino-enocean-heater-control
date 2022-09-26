pub(crate) mod model;
pub(crate) mod xpath_ext;


use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::num::ParseIntError;
use std::path::PathBuf;

use askama::Template;
use clap::Parser;
use sxd_document;
use sxd_xpath;

use crate::model::{
    Eeps, EnumeratedProperty, EnumValue, Func, NumericProperty, Property, PropertyCommon,
    RawOnlyProperty, Rorg, Type,
};
use crate::xpath_ext::{FactoryExt, XPathExt};


#[derive(Parser)]
struct Args {
    #[clap(help = "Path to the eep.xml file to read.", long_help = "Path to the eep.xml file to read. Can be downloaded using the \"Bulk Download\" feature of the EnOcean Alliance's EEP Viewer web application. May have a different name, e.g. eep268.xml.")]
    pub eep_xml: PathBuf,

    #[clap(help = "Path to the eep.rs file to write.")]
    pub eep_rs: PathBuf,
}


fn parse_u32(s: &str) -> Result<u32, ParseIntError> {
    if let Some(rest) = s.strip_prefix("0x") {
        u32::from_str_radix(rest, 16)
    } else if let Some(rest) = s.strip_prefix("0b") {
        u32::from_str_radix(rest, 2)
    } else if let Some(rest) = s.strip_prefix("0o") {
        u32::from_str_radix(rest, 8)
    } else {
        u32::from_str_radix(s, 10)
    }
}


fn main() {
    let args = Args::parse();

    // read eep.xml -- warning: it's in UTF-16LE
    let xml_package = {
        let mut xml_file = File::open(&args.eep_xml)
            .expect("failed to open eep.xml file");

        let mut xml_bytes = Vec::new();
        xml_file.read_to_end(&mut xml_bytes)
            .expect("failed to read eep.xml file");

        if xml_bytes.len() % 2 != 0 {
            panic!("eep.xml file (expected to be encoded using UTF-16LE) has an odd number of bits!");
        }

        let mut xml_units = Vec::with_capacity(xml_bytes.len()/2);
        let mut xml_bytes_iter = xml_bytes.into_iter();
        let mut first_character = true;
        while let Some(first_byte) = xml_bytes_iter.next() {
            // we checked this before...
            let second_byte = xml_bytes_iter.next()
                .expect("UTF-16 first byte without a second byte?!");

            let u16_char = u16::from_le_bytes([first_byte, second_byte]);

            if first_character && u16_char == 0xFEFF {
                // skip it
                first_character = false;
                continue;
            }
            first_character = false;

            xml_units.push(u16_char);
        }

        let xml_string = String::from_utf16(&xml_units)
            .expect("failed to decode eep.xml as UTF-16LE");
        sxd_document::parser::parse(&xml_string)
            .expect("failed to parse eep.xml")
    };

    // prepare some XPath-related objects
    let xpath_factory = sxd_xpath::Factory::new();
    let xpath_ctx = sxd_xpath::Context::new();

    // prepare some element XPaths
    let rorgs_exp = xpath_factory.build_strict("/eep/profile/rorg");
    let funcs_exp = xpath_factory.build_strict("./func");
    let types_exp = xpath_factory.build_strict("./type");
    let cases_exp = xpath_factory.build_strict("./case");
    let data_fields_exp = xpath_factory.build_strict("./datafield");
    let reserved_exp = xpath_factory.build_strict("./reserved");
    let enum_item_exp = xpath_factory.build_strict("./enum/item");

    // prepare some string XPaths
    let number_sxp = xpath_factory.build_strict("./number/text()");
    let title_sxp = xpath_factory.build_strict("./title/text()");
    let bit_offset_sxp = xpath_factory.build_strict("./bitoffs/text()");
    let bit_size_sxp = xpath_factory.build_strict("./bitsize/text()");
    let data_sxp = xpath_factory.build_strict("./data/text()");
    let range_min_sxp = xpath_factory.build_strict("./range/min/text()");
    let range_max_sxp = xpath_factory.build_strict("./range/max/text()");
    let scale_min_sxp = xpath_factory.build_strict("./scale/min/text()");
    let scale_max_sxp = xpath_factory.build_strict("./scale/max/text()");
    let scale_ref_sxp = xpath_factory.build_strict("./scale/ref/text()");
    let unit_sxp = xpath_factory.build_strict("./unit/text()");
    let value_sxp = xpath_factory.build_strict("./value/text()");
    let description_sxp = xpath_factory.build_strict("./description/text()");

    let mut eeps = Eeps {
        rorgs: Vec::new(),
    };

    // run through it
    for rorg in rorgs_exp.eval_strict_nodeset(&xpath_ctx, xml_package.as_document().root()) {
        let rorg_number = number_sxp.eval_strict_stru8(&xpath_ctx, rorg);
        let rorg_title = title_sxp.eval_strict_string(&xpath_ctx, rorg);

        let mut rorg_def = Rorg {
            name: rorg_title,
            code: rorg_number,
            funcs: Vec::new(),
        };

        for func in funcs_exp.eval_strict_nodeset(&xpath_ctx, rorg) {
            let func_number = number_sxp.eval_strict_stru8(&xpath_ctx, func);
            let func_title = title_sxp.eval_strict_string(&xpath_ctx, func);

            let mut func_def = Func {
                name: func_title,
                code: func_number,
                types: Vec::new(),
            };

            for tp in types_exp.eval_strict_nodeset(&xpath_ctx, func) {
                let type_number = number_sxp.eval_strict_stru8(&xpath_ctx, tp);
                let type_title = title_sxp.eval_strict_string(&xpath_ctx, tp);

                let mut type_def = Type {
                    name: type_title,
                    code: type_number,
                    properties: Vec::new(),
                };

                let mut cases = cases_exp.eval_strict_nodeset(&xpath_ctx, tp);
                if cases.len() == 0 {
                    continue;
                }
                let case = cases.swap_remove(0);

                let mut field_duplicate_counters: HashMap<String, usize> = HashMap::new();
                for field in data_fields_exp.eval_strict_nodeset(&xpath_ctx, case) {
                    if reserved_exp.eval_strict_node_exists(&xpath_ctx, field) {
                        // bits are reserved; skip them
                        continue;
                    }

                    let mut field_name = data_sxp.eval_strict_string(&xpath_ctx, field);
                    let bit_offset = bit_offset_sxp.eval_strict_stru32(&xpath_ctx, field);
                    let bit_size = bit_size_sxp.eval_strict_stru32(&xpath_ctx, field);
                    let unit = unit_sxp.eval_strict_string(&xpath_ctx, field)
                        .trim().to_owned();

                    {
                        let dupe_count = field_duplicate_counters
                            .entry(field_name.clone())
                            .or_insert(0);
                        if *dupe_count > 0 {
                            field_name = format!("{} {}", field_name, dupe_count);
                        }
                        *dupe_count += 1;
                    }

                    let raw_primitive_type = if bit_size == 1 {
                        "bool"
                    } else if bit_size <= 8 {
                        "u8"
                    } else if bit_size <= 16 {
                        "u16"
                    } else if bit_size <= 32 {
                        "u32"
                    } else if bit_size <= 64 {
                        "u64"
                    } else if bit_size <= 128 {
                        "u128"
                    } else {
                        "f64"
                    };

                    let common = PropertyCommon {
                        name: field_name,
                        raw_primitive_type,
                        lowest_bit_index: bit_offset.try_into().unwrap(),
                        bit_count: bit_size.try_into().unwrap(),
                        unit: if unit.len() > 0 { Some(unit) } else { None },
                    };

                    let enum_items = enum_item_exp.eval_strict_nodeset(&xpath_ctx, field);
                    let property = if enum_items.len() == 0 {
                        let mut raw_only = false;

                        // protect against unhandled cases
                        if scale_ref_sxp.eval_strict_node_exists(&xpath_ctx, field) {
                            eprintln!("warning: no support for scale references; cannot process property {:?}", common.name);
                            raw_only = true;
                        }
                        if !range_min_sxp.eval_strict_node_exists(&xpath_ctx, field) {
                            eprintln!("warning: no range minimum found; cannot process property {:?}", common.name);
                            raw_only = true;
                        }
                        if !scale_min_sxp.eval_strict_node_exists(&xpath_ctx, field) {
                            eprintln!("warning: no scale minimum found; cannot process property {:?}", common.name);
                            raw_only = true;
                        }

                        // work around BS
                        let range_max_string = range_max_sxp.eval_strict_string(&xpath_ctx, field);
                        if range_max_string.contains(", ") {
                            eprintln!("warning: range maximum {:?} contains more than one value; cannot process property {:?}", range_max_string, common.name);
                            raw_only = true;
                        }
                        if range_max_string.contains("0x") {
                            eprintln!("warning: range maximum {:?} is not a valid float value; cannot process property {:?}", range_max_string, common.name);
                            raw_only = true;
                        }
                        if scale_min_sxp.eval_strict_string(&xpath_ctx, field).contains("..") {
                            eprintln!("warning: scale minimum contains range; cannot process property {:?}", common.name);
                            raw_only = true;
                        }
                        if scale_max_sxp.eval_strict_string(&xpath_ctx, field).contains("(") {
                            eprintln!("warning: scale minimum contains \"(\"; cannot process property {:?}", common.name);
                            raw_only = true;
                        }

                        if raw_only {
                            Property::RawOnly(RawOnlyProperty {
                                common,
                            })
                        } else {
                            let min_range = range_min_sxp.eval_strict_strf64(&xpath_ctx, field);
                            let max_range = range_max_sxp.eval_strict_strf64(&xpath_ctx, field);
                            let min_scale = scale_min_sxp.eval_strict_strf64(&xpath_ctx, field);
                            let max_scale = scale_max_sxp.eval_strict_strf64(&xpath_ctx, field);
                            let num_prop = NumericProperty {
                                common,
                                min_range,
                                max_range,
                                min_scale,
                                max_scale,
                            };
                            Property::Numeric(num_prop)
                        }
                    } else {
                        let mut enum_prop = EnumeratedProperty {
                            common,
                            values: Vec::new(),
                        };
                        let mut enum_item_duplicate_counters: HashMap<String, usize> = HashMap::new();
                        for enum_item in enum_items {
                            let value_string = value_sxp.eval_strict_string(&xpath_ctx, enum_item);
                            if value_string.len() == 0 {
                                // FIXME: possibly has min-max values instead
                                continue;
                            }
                            if value_string.contains('X') {
                                // has don't-care bits
                                continue;
                            }
                            if value_string.contains("...") {
                                // is a range?! this should be min-max
                                continue;
                            }
                            let value: u32 = match parse_u32(&value_string) {
                                Ok(v) => v,
                                Err(e) => panic!(
                                    "failed to parse value {:?} for {} as u32: {}",
                                    value_string, enum_prop.common.name, e,
                                ),
                            };
                            let value_string = if raw_primitive_type == "bool" {
                                if value == 0 {
                                    "false".to_owned()
                                } else if value == 1 {
                                    "true".to_owned()
                                } else {
                                    value.to_string()
                                }
                            } else {
                                value.to_string()
                            };
                            let description = description_sxp.eval_strict_string(&xpath_ctx, enum_item);
                            let dupe_count = enum_item_duplicate_counters
                                .entry(description.clone())
                                .or_insert(0);
                            let modified_description = if *dupe_count == 0 {
                                description
                            } else {
                                format!("{} {}", description, *dupe_count)
                            };
                            *dupe_count += 1;
                            enum_prop.values.push(EnumValue {
                                name: modified_description,
                                value: value_string,
                            });
                        }
                        Property::Enumerated(enum_prop)
                    };
                    type_def.properties.push(property);
                }

                func_def.types.push(type_def);
            }

            rorg_def.funcs.push(func_def);
        }

        eeps.rorgs.push(rorg_def);
    }

    // render
    let rendered_template = eeps.render()
        .expect("failed to render template");

    {
        let mut f = File::create(args.eep_rs)
            .expect("failed to open eep.rs file");
        f.write_all(rendered_template.as_bytes())
            .expect("failed to write eep.rs file");
    }
}
