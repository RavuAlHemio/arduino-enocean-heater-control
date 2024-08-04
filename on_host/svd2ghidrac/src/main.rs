use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use clap::Parser;
use serde::Serialize;
use sxd_document::parser::parse;
use sxd_xpath::nodeset::{Node, Nodeset};
use sxd_xpath::{Context, Factory, Value as XPathValue, XPath};

#[derive(Parser)]
struct Args {
    #[clap(help = "SVD file")]
    pub svd_file: PathBuf,

    #[clap(help = "C header file")]
    pub c_header: PathBuf,

    #[clap(help = "Address space JSON file")]
    pub addr_space_json: PathBuf,

    #[clap(long, help = "Tag struct members as volatile")]
    pub volatile: bool,

    #[clap(long, help = "Also name structs, not only typedefs")]
    pub name_structs: bool,

    #[clap(long = "--bad-peripheral", help = "Index of a peripheral to skip. Can be provided multiple times.")]
    pub bad_peripherals: Vec<usize>,
}


#[derive(Debug, Serialize)]
struct AddressSpaces {
    pub cpu_core: String,
    pub peripheral_address_space: AddressSpace,
    pub registers: Vec<RegisterDef>,
    pub interrupts: Vec<Interrupt>,
}

#[derive(Debug, Serialize)]
struct AddressSpace {
    pub base_address: u32,
    pub length: u32,
}

#[derive(Debug, Serialize)]
struct RegisterDef {
    pub address: u32,
    pub register: String,
    pub type_name: String,
}

#[derive(Debug, Serialize)]
struct Interrupt {
    pub index: u32,
    pub name: String,
}


trait XPathValueExt<'d> {
    fn nodeset(&'d self) -> Option<&'d Nodeset<'d>>;
    fn into_nodeset(self) -> Option<Nodeset<'d>>;
}
impl<'d> XPathValueExt<'d> for XPathValue<'d> {
    fn nodeset(&'d self) -> Option<&'d Nodeset<'d>> {
        match self {
            Self::Nodeset(ns) => Some(ns),
            _ => None,
        }
    }

    fn into_nodeset(self) -> Option<Nodeset<'d>> {
        match self {
            Self::Nodeset(ns) => Some(ns),
            _ => None,
        }
    }
}


trait StringExt {
    fn strip_suffix_or_dont<'a, 'b>(&'a self, suffix: &'b str) -> &'a str;
}
impl StringExt for str {
    fn strip_suffix_or_dont<'a, 'b>(&'a self, suffix: &'b str) -> &'a str {
        if let Some(stripped) = self.strip_suffix(suffix) {
            stripped
        } else {
            self
        }
    }
}


fn make_xpath(xpath_factory: &Factory, path: &str) -> XPath {
    xpath_factory.build(path)
        .expect("failed to build XPath")
        .expect("parsed XPath is None")
}

fn eval_xpath_nodeset<'d, N: Into<Node<'d>>>(context: &Context<'d>, xpath: &XPath, node: N) -> Nodeset<'d> {
    xpath.evaluate(context, node)
        .expect("failed to evaluate XPath")
        .into_nodeset()
        .expect("XPath does not evaluate to nodeset")
}

fn eval_xpath_string<'d, N: Into<Node<'d>>>(context: &Context<'d>, xpath: &XPath, node: N) -> String {
    xpath.evaluate(context, node)
        .expect("failed to evaluate XPath")
        .into_string()
}

fn u32_from_str(s: &str) -> Option<u32> {
    if s.starts_with("0x") {
        u32::from_str_radix(&s[2..], 16).ok()
    } else if s.starts_with("0b") {
        u32::from_str_radix(&s[2..], 2).ok()
    } else if s.starts_with("0o") {
        u32::from_str_radix(&s[2..], 8).ok()
    } else {
        u32::from_str_radix(s, 10).ok()
    }
}


enum Register {
    Singular(String),
    Array(String, u32),
    Reserved(u32),
}


#[inline]
fn print_start<W: Write>(mut writer: W, args: &Args) {
    write!(&mut writer, "  ").unwrap();
    if args.volatile {
        write!(&mut writer, "volatile ").unwrap();
    }
}


fn main() {
    let args = Args::parse();

    // load the SVD file
    let doc_package = {
        let mut svd_file = File::open(&args.svd_file)
            .expect("failed to open SVD file");
        let mut svd_string = String::new();
        svd_file.read_to_string(&mut svd_string)
            .expect("failed to read SVD file");
        parse(&svd_string)
            .expect("failed to parse SVD file")
    };
    let doc = doc_package.as_document();

    let xpath_factory = Factory::new();

    let peripheral_xpath = make_xpath(&xpath_factory, "/device/peripherals/peripheral");
    let register_xpath = make_xpath(&xpath_factory, "./registers/register");
    let field_xpath = make_xpath(&xpath_factory, "./fields/field");

    let name_string_xpath = make_xpath(&xpath_factory, "./name/text()");
    let base_addr_string_xpath = make_xpath(&xpath_factory, "./baseAddress/text()");
    let address_offset_string_xpath = make_xpath(&xpath_factory, "./addressOffset/text()");
    let size_string_xpath = make_xpath(&xpath_factory, "./size/text()");
    let description_string_xpath = make_xpath(&xpath_factory, "./description/text()");
    let bit_offset_string_xpath = make_xpath(&xpath_factory, "./bitOffset/text()");
    let bit_width_string_xpath = make_xpath(&xpath_factory, "./bitWidth/text()");
    let dim_string_xpath = make_xpath(&xpath_factory, "./dim/text()");
    let dim_increment_string_xpath = make_xpath(&xpath_factory, "./dimIncrement/text()");
    let dim_index_string_xpath = make_xpath(&xpath_factory, "./dimIndex/text()");
    let alt_group_string_xpath = make_xpath(&xpath_factory, "./alternateGroup/text()");
    let interrupt_name_string_xpath = make_xpath(&xpath_factory, "./interrupt/name/text()");
    let interrupt_value_string_xpath = make_xpath(&xpath_factory, "./interrupt/value/text()");
    let cpu_core_name_string_xpath = make_xpath(&xpath_factory, "/device/cpu/name/text()");

    let xpath_context = Context::new();

    let cpu_core_name = eval_xpath_string(&xpath_context, &cpu_core_name_string_xpath, doc.root());

    let mut min_peripheral_address: u32 = u32::MAX;
    let mut max_peripheral_address: u32 = u32::MIN;
    let mut register_defs: Vec<RegisterDef> = Vec::new();
    let mut interrupts: Vec<Interrupt> = Vec::new();

    {
        let mut header = File::create(&args.c_header)
            .expect("failed to open C header file");

        writeln!(&mut header, "/* generated by svd2gdt */").unwrap();
        writeln!(&mut header).unwrap();
        writeln!(&mut header, "#ifdef GHIDRA_STDINT").unwrap();
        writeln!(&mut header, "typedef sbyte int8_t;").unwrap();
        writeln!(&mut header, "typedef sword int16_t;").unwrap();
        writeln!(&mut header, "typedef sdword int32_t;").unwrap();
        writeln!(&mut header, "typedef byte uint8_t;").unwrap();
        writeln!(&mut header, "typedef word uint16_t;").unwrap();
        writeln!(&mut header, "typedef dword uint32_t;").unwrap();
        writeln!(&mut header, "#endif").unwrap();
        writeln!(&mut header).unwrap();
        writeln!(&mut header, "#pragma pack(push,1)").unwrap();

        let peripherals = eval_xpath_nodeset(&xpath_context, &peripheral_xpath, doc.root());
        for (i, peripheral) in peripherals.document_order().into_iter().enumerate() {
            eprintln!("{}/{} peripherals output...", i, peripherals.size());

            if args.bad_peripherals.contains(&i) {
                continue;
            }

            let peri_name = eval_xpath_string(&xpath_context, &name_string_xpath, peripheral);
            let peri_descr = eval_xpath_string(&xpath_context, &description_string_xpath, peripheral);
            let base_address_string = eval_xpath_string(&xpath_context, &base_addr_string_xpath, peripheral);
            let base_address = u32_from_str(&base_address_string)
                .expect("failed to parse base address");

            min_peripheral_address = min_peripheral_address.min(base_address);
            max_peripheral_address = max_peripheral_address.max(base_address);

            let peri_interrupt_name = eval_xpath_string(&xpath_context, &interrupt_name_string_xpath, peripheral);
            let peri_interrupt_value_str = eval_xpath_string(&xpath_context, &interrupt_value_string_xpath, peripheral);
            if let Some(peri_interrupt_value) = u32_from_str(&peri_interrupt_value_str) {
                if peri_interrupt_name.len() > 0 {
                    interrupts.push(Interrupt {
                        index: peri_interrupt_value,
                        name: peri_interrupt_name,
                    });
                }
            }

            let registers = eval_xpath_nodeset(&xpath_context, &register_xpath, peripheral);
            let mut offsets_and_registers = Vec::with_capacity(registers.size());
            for register in registers.document_order() {
                let reg_offset_string = eval_xpath_string(&xpath_context, &address_offset_string_xpath, register);
                let reg_offset = u32_from_str(&reg_offset_string)
                    .expect("failed to parse register offset");
                offsets_and_registers.push((reg_offset, register));
            }

            offsets_and_registers.sort_unstable_by_key(|(o, _r)| *o);

            let mut register_info = Vec::new();
            let mut register_pos = 0;
            for (reg_offset, register) in offsets_and_registers {
                let reg_name_orig = eval_xpath_string(&xpath_context, &name_string_xpath, register);
                let reg_descr = eval_xpath_string(&xpath_context, &description_string_xpath, register);
                let reg_size_string = eval_xpath_string(&xpath_context, &size_string_xpath, register);
                let reg_dim_string = eval_xpath_string(&xpath_context, &dim_string_xpath, register);
                let reg_dim_incr_string = eval_xpath_string(&xpath_context, &dim_increment_string_xpath, register);
                let reg_dim_index_string = eval_xpath_string(&xpath_context, &dim_index_string_xpath, register);
                let reg_alt_group_string = eval_xpath_string(&xpath_context, &alt_group_string_xpath, register);

                if reg_alt_group_string.len() > 0 {
                    continue;
                }
                if reg_size_string.len() == 0 {
                    continue;
                }

                let reg_name = reg_name_orig
                    .replace("[%s]", "")
                    .replace("%s", "");
                let reg_size = match u32_from_str(&reg_size_string) {
                    Some(rs) => rs,
                    None => panic!("failed to parse register size {:?}", reg_size_string),
                };
                let reg_dim = if reg_dim_string.len() > 0 {
                    u32_from_str(&reg_dim_string)
                        .expect("failed to parse register dimension")
                } else {
                    1
                };
                let reg_dim_incr = if reg_dim_incr_string.len() > 0 {
                    u32_from_str(&reg_dim_incr_string)
                        .expect("failed to parse register dimension increment")
                } else {
                    reg_size / 8
                };

                if reg_size % 8 != 0 {
                    panic!("register {}.{} does not end on a byte boundary", peri_name, reg_name);
                }

                max_peripheral_address = max_peripheral_address.max(base_address + reg_offset + reg_dim * reg_size / 8);

                if reg_dim_index_string.len() > 0 {
                    if reg_dim_index_string != format!("0-{}", reg_dim-1) {
                        panic!("register {}.{} dimension indexes are unexpected (got {:?}, expected \"0-{}\")", peri_name, reg_name, reg_dim_index_string, reg_dim-1);
                    }
                }

                for i in 0..reg_dim {
                    let reg_type = format!("{}_{}", peri_name, reg_name);
                    let reg_full_name = if reg_dim == 1 {
                        reg_type.clone()
                    } else {
                        format!("{}{}", reg_type, i)
                    };
                    register_defs.push(RegisterDef {
                        address: base_address + reg_offset + i * reg_dim_incr,
                        register: reg_full_name.clone(),
                        type_name: reg_type,
                    });
                }

                register_pos = reg_offset + reg_dim * reg_size / 8;

                writeln!(&mut header).unwrap();
                writeln!(&mut header, "/** {} */", reg_descr).unwrap();
                write!(&mut header, "typedef struct").unwrap();
                if args.name_structs {
                    write!(&mut header, " {}_{}_s", peri_name, reg_name).unwrap();
                }
                writeln!(&mut header, " {{").unwrap();

                let fields = eval_xpath_nodeset(&xpath_context, &field_xpath, register);
                let mut fields_vec = fields.document_order();
                fields_vec.sort_unstable_by_key(|field| {
                    let field_offset_string = eval_xpath_string(&xpath_context, &bit_offset_string_xpath, *field);
                    let field_offset = u32_from_str(&field_offset_string)
                        .expect("failed to parse field offset");
                    field_offset
                });

                let mut current_bit = 0;
                let mut field_reserved_index = 0;
                for field in fields_vec {
                    let field_name = eval_xpath_string(&xpath_context, &name_string_xpath, field);
                    let field_descr = eval_xpath_string(&xpath_context, &description_string_xpath, field);
                    let field_offset_string = eval_xpath_string(&xpath_context, &bit_offset_string_xpath, field);
                    let field_width_string = eval_xpath_string(&xpath_context, &bit_width_string_xpath, field);
                    let field_offset = u32_from_str(&field_offset_string)
                        .expect("failed to parse field offset");
                    let field_width = u32_from_str(&field_width_string)
                        .expect("failed to parse field width");

                    if current_bit < field_offset {
                        print_start(&mut header, &args);
                        writeln!(&mut header, "uint{}_t reserved{} : {};", reg_size, field_reserved_index, field_offset - current_bit).unwrap();
                        field_reserved_index += 1;
                    } else if current_bit > field_offset {
                        panic!("out of order field! we are at {}, offset is {}", current_bit, field_offset);
                    }

                    writeln!(&mut header, "  /** {} */", field_descr).unwrap();
                    print_start(&mut header, &args);
                    writeln!(&mut header, "uint{}_t {} : {};", reg_size, field_name, field_width).unwrap();
                    current_bit = field_offset + field_width;
                }

                if current_bit < reg_size {
                    print_start(&mut header, &args);
                    writeln!(&mut header, "uint{}_t reserved_tail : {};", reg_size, reg_size - current_bit).unwrap();
                }

                writeln!(&mut header, "}} {}_{};", peri_name, reg_name).unwrap();

                if reg_dim == 1 {
                    register_info.push(Register::Singular(reg_name.clone()));
                } else {
                    register_info.push(Register::Array(reg_name.clone(), reg_dim));
                }
            }

            writeln!(&mut header).unwrap();
            writeln!(&mut header, "/** {} */", peri_descr).unwrap();
            write!(&mut header, "typedef struct").unwrap();
            if args.name_structs {
                write!(&mut header, " {}_s", peri_name).unwrap();
            }
            writeln!(&mut header, " {{").unwrap();
            let mut reserved_counter = 0;
            for register in register_info {
                match register {
                    Register::Reserved(mut byte_count) => {
                        if byte_count >= 4 {
                            print_start(&mut header, &args);
                            writeln!(&mut header, "uint32_t reserved{}[{}];", reserved_counter, byte_count / 4).unwrap();
                            reserved_counter += 1;
                            byte_count %= 4;
                        }
                        if byte_count >= 2 {
                            print_start(&mut header, &args);
                            writeln!(&mut header, "uint16_t reserved{}[{}];", reserved_counter, byte_count / 2).unwrap();
                            reserved_counter += 1;
                            byte_count %= 2;
                        }
                        if byte_count >= 1 {
                            print_start(&mut header, &args);
                            writeln!(&mut header, "uint8_t reserved{}[{}];", reserved_counter, byte_count).unwrap();
                            reserved_counter += 1;
                        }
                    },
                    Register::Array(register_name, dimensions) => {
                        print_start(&mut header, &args);
                        writeln!(&mut header, "{}_{} {}[{}];", peri_name, register_name, register_name, dimensions).unwrap();
                    },
                    Register::Singular(register_name) => {
                        print_start(&mut header, &args);
                        writeln!(&mut header, "{}_{} {};", peri_name, register_name, register_name).unwrap();
                    },
                }
            }
            writeln!(&mut header, "}} {};", peri_name).unwrap();
        }

        writeln!(&mut header).unwrap();
        writeln!(&mut header, "#pragma pack(pop)").unwrap();
    }

    // now, the address space JSON
    let peripheral_address_space = AddressSpace {
        base_address: min_peripheral_address,
        length: max_peripheral_address - min_peripheral_address,
    };
    let spaces = AddressSpaces {
        cpu_core: cpu_core_name,
        peripheral_address_space,
        registers: register_defs,
        interrupts,
    };

    {
        let aspace = File::create(&args.addr_space_json)
            .expect("failed to open address space file");
        serde_json::to_writer_pretty(aspace, &spaces)
            .expect("failed to write address space file");
    }
}
