use askama::Template;


#[derive(Template)]
#[template(path = "unpack.rs.askama", escape = "none", syntax = "asplike")]
pub(crate) struct Eeps {
    pub rorgs: Vec<Rorg>,
}

pub(crate) struct Rorg {
    pub name: String,
    pub code: u8,
    pub funcs: Vec<Func>,
}

pub(crate) struct Func {
    pub name: String,
    pub code: u8,
    pub types: Vec<Type>,
}

pub(crate) struct Type {
    pub name: String,
    pub code: u8,
    pub properties: Vec<Property>,
}

pub(crate) enum Property {
    Numeric(NumericProperty),
    Enumerated(EnumeratedProperty),
}
impl Property {
    pub fn name(&self) -> &str {
        match self {
            Self::Numeric(n) => n.name.as_str(),
            Self::Enumerated(e) => e.name.as_str(),
        }
    }

    pub fn raw_primitive_type(&self) -> &str {
        match self {
            Self::Numeric(n) => n.raw_primitive_type,
            Self::Enumerated(e) => e.raw_primitive_type,
        }
    }

    pub fn lowest_bit_index(&self) -> usize {
        match self {
            Self::Numeric(n) => n.lowest_bit_index,
            Self::Enumerated(e) => e.lowest_bit_index,
        }
    }

    pub fn bit_count(&self) -> usize {
        match self {
            Self::Numeric(n) => n.bit_count,
            Self::Enumerated(e) => e.bit_count,
        }
    }

    pub fn unit(&self) -> Option<&String> {
        match self {
            Self::Numeric(n) => n.unit.as_ref(),
            Self::Enumerated(e) => e.unit.as_ref(),
        }
    }
}

pub(crate) struct NumericProperty {
    pub name: String,
    pub raw_primitive_type: &'static str,
    pub lowest_bit_index: usize,
    pub bit_count: usize,
    pub unit: Option<String>,
    pub min_range: f64,
    pub max_range: f64,
    pub min_scale: f64,
    pub max_scale: f64,
}

pub(crate) struct EnumeratedProperty {
    pub name: String,
    pub raw_primitive_type: &'static str,
    pub lowest_bit_index: usize,
    pub bit_count: usize,
    pub unit: Option<String>,
    pub values: Vec<EnumValue>,
}

pub(crate) struct EnumValue {
    pub name: String,
    pub value: String,
}

pub(crate) mod filters {
    pub fn pascal_case(value: &str) -> askama::Result<String> {
        let mut ret = String::new();
        for word in value.split(' ') {
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

    pub fn snake_case(value: &str) -> askama::Result<String> {
        let mut ret = String::new();
        for word in value.split(' ') {
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
}
