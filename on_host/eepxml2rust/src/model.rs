use askama::Template;


#[derive(Template)]
#[template(path = "unpack.rs.askama", escape = "none", syntax = "asplike")]
pub(crate) struct Eeps {
    pub rorgs: Vec<Rorg>,
}
impl Eeps {
    pub fn max_cases_per_type(&self) -> usize {
        self.rorgs
            .iter()
            .map(|r| r.max_cases_per_type())
            .max()
            .unwrap_or(0)
    }
}

pub(crate) struct Rorg {
    pub name: String,
    pub code: u8,
    pub funcs: Vec<Func>,
}
impl Rorg {
    pub fn max_cases_per_type(&self) -> usize {
        self.funcs
            .iter()
            .map(|t| t.max_cases_per_type())
            .max()
            .unwrap_or(0)
    }
}

pub(crate) struct Func {
    pub name: String,
    pub code: u8,
    pub types: Vec<Type>,
}
impl Func {
    pub fn max_cases_per_type(&self) -> usize {
        self.types
            .iter()
            .map(|t| t.cases.len())
            .max()
            .unwrap_or(0)
    }
}

pub(crate) struct Type {
    pub name: String,
    pub code: u8,
    pub cases: Vec<Case>,
}

pub(crate) struct Case {
    pub number: Option<usize>,
    pub properties: Vec<Property>,
}

pub(crate) enum Property {
    Numeric(NumericProperty),
    Enumerated(EnumeratedProperty),
    RawOnly(RawOnlyProperty),
}
impl Property {
    pub fn common(&self) -> &PropertyCommon {
        match self {
            Self::Numeric(n) => &n.common,
            Self::Enumerated(e) => &e.common,
            Self::RawOnly(ro) => &ro.common,
        }
    }
}

pub(crate) struct PropertyCommon {
    pub name: String,
    pub raw_primitive_type: &'static str,
    pub lowest_bit_index: usize,
    pub bit_count: usize,
    pub unit: Option<String>,
}

pub(crate) struct NumericProperty {
    pub common: PropertyCommon,
    pub min_range: f64,
    pub max_range: f64,
    pub min_scale: f64,
    pub max_scale: f64,
}

pub(crate) struct EnumeratedProperty {
    pub common: PropertyCommon,
    pub values: Vec<EnumValue>,
}

pub(crate) struct RawOnlyProperty {
    pub common: PropertyCommon,
}

pub(crate) enum EnumValue {
    Const(ConstEnumValue),
    Ranged(RangedEnumValue),
}

pub(crate) struct ConstEnumValue {
    pub name_pascal: String,
    pub value: String,
}

pub(crate) struct RangedEnumValue {
    pub name_pascal: String,
    pub min_range: f64,
    pub max_range: f64,
    pub min_scale: f64,
    pub max_scale: f64,
}

pub(crate) mod filters {
    fn remove_special_characters(value: &str) -> String {
        let mut ret = String::new();
        let mut previous_minus = false;
        for c in value.chars() {
            if previous_minus {
                if c >= '0' && c <= '9' {
                    ret.push_str("minus ");
                }
                previous_minus = false;
            }
            match c {
                'a'..='z' => ret.push(c),
                'A'..='Z' => ret.push(c),
                '0'..='9' => ret.push(c),
                '_' => ret.push(c),
                '\u{B2}' => ret.push('2'),
                '<' => ret.push_str(" less than "),
                '>' => ret.push_str(" more than "),
                '\u{3BC}'|'\u{B5}' => ret.push_str(" mu "),
                '\u{B0}' => ret.push_str(" degrees "),
                '-' => {
                    previous_minus = true;
                    ret.push(' ');
                },
                ' '|'\r'|'\n'|'\t'|'\u{A0}' => ret.push(' '),
                '.'|','|':'|';'|'\u{2013}'|'\u{2014}'|'/'|'*'|'+'|'='|'%'|'\u{2026}' => ret.push(' '),
                '\u{2264}'|'\u{2265}' => ret.push(' '),
                '\u{2019}'|'\u{201C}'|'\u{201D}' => ret.push(' '),
                '('|')'|'['|']' => ret.push(' '),
                other => {
                    eprintln!("unhandled special character {:?}!", other);
                },
            }
        }
        ret
    }

    pub fn pascal_case(value: &str) -> askama::Result<String> {
        let mut ret = String::new();
        let clean_value = remove_special_characters(value);
        for word in clean_value.split(' ') {
            let word_trimmed = word.trim();
            if word_trimmed.len() == 0 {
                continue;
            }
            for (i, c) in word_trimmed.char_indices() {
                if i == 0 {
                    // uppercase first letter
                    for upper_c in c.to_uppercase() {
                        ret.push(upper_c);
                    }
                } else {
                    // lowercase all others
                    for lower_c in c.to_lowercase() {
                        ret.push(lower_c);
                    }
                }
            }
        }
        Ok(ret)
    }

    pub fn pascal_case_word_start(value: &str) -> askama::Result<String> {
        let mut pc = pascal_case(value)?;

        // prepend underscore if we begin with a digit
        if pc.len() > 0 {
            let first_char = pc.chars().nth(0).unwrap();
            if first_char >= '0' && first_char <= '9' {
                pc.insert(0, '_');
            }
        }

        Ok(pc)
    }

    pub fn snake_case(value: &str) -> askama::Result<String> {
        let mut ret = String::new();
        let clean_value = remove_special_characters(value);
        for word in clean_value.split(' ') {
            let word_trimmed = word.trim();
            if word_trimmed.len() == 0 {
                continue;
            }
            if ret.len() > 0 {
                ret.push('_');
            }
            ret.push_str(&word_trimmed.to_lowercase());
        }
        Ok(ret)
    }

    pub fn hex(value: &u8) -> askama::Result<String> {
        Ok(format!("{:02X}", value))
    }

    pub fn pascal_fallback(name: &str, fallback_value: &str) -> askama::Result<String> {
        if name.len() > 0 {
            return Ok(name.to_owned());
        }

        pascal_case(&format!("value {}", fallback_value))
    }

    pub fn dec(value: &f64) -> askama::Result<String> {
        let mut string = value.to_string();
        if !string.contains(".") {
            string.push_str(".0");
        }
        Ok(string)
    }

    pub(crate) fn typecasename(tp: &&super::Type, cs: &&super::Case) -> askama::Result<String> {
        use std::fmt::Write;
        let mut case_name = format!("Type{}", hex(&tp.code)?);
        if let Some(csnum) = cs.number {
            write!(&mut case_name, "Case{}", csnum).unwrap();
        }
        Ok(case_name)
    }
}
