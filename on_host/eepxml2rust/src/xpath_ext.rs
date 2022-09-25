use sxd_xpath::{Context, Factory, Value, XPath};
use sxd_xpath::nodeset::{Node, Nodeset};


pub(crate) trait FactoryExt {
    fn build_strict(&self, xpath: &str) -> XPath;
}
impl FactoryExt for Factory {
    fn build_strict(&self, xpath: &str) -> XPath {
        match self.build(xpath) {
            Err(e) => panic!("failed to parse XPath {:?}: {}", xpath, e),
            Ok(None) => panic!("parsing XPath {:?} returned None", xpath),
            Ok(Some(xp)) => xp,
        }
    }
}

pub(crate) trait XPathExt {
    fn eval_strict<'d, N: Into<Node<'d>>>(&self, context: &Context<'d>, node: N) -> Value<'d>;

    fn eval_strict_nodeset<'d, N: Into<Node<'d>>>(&self, context: &Context<'d>, node: N) -> Vec<Node<'d>> {
        self.eval_strict(context, node).into_nodeset_strict().document_order()
    }
    fn eval_strict_node_exists<'d, N: Into<Node<'d>>>(&self, context: &Context<'d>, node: N) -> bool {
        self.eval_strict(context, node).into_nodeset_strict().size() > 0
    }
    fn eval_strict_string<'d, N: Into<Node<'d>>>(&self, context: &Context<'d>, node: N) -> String {
        // be a bit more liberal here; we might be getting a text node nodeset
        self.eval_strict(context, node).into_string()
    }
    fn eval_strict_stru32<'d, N: Into<Node<'d>>>(&self, context: &Context<'d>, node: N) -> u32 {
        let string = self.eval_strict_string(context, node);
        let trimmed_string = string.trim();
        let val_res = if let Some(rest_str) = trimmed_string.strip_prefix("0x") {
            u32::from_str_radix(rest_str, 16)
        } else if let Some(rest_str) = trimmed_string.strip_prefix("0b") {
            u32::from_str_radix(rest_str, 2)
        } else if let Some(rest_str) = trimmed_string.strip_prefix("0o") {
            u32::from_str_radix(rest_str, 8)
        } else {
            u32::from_str_radix(&trimmed_string, 10)
        };
        match val_res {
            Ok(v) => v,
            Err(e) => panic!("failed to parse {:?} as u32: {}", trimmed_string, e),
        }
    }
    fn eval_strict_stru8<'d, N: Into<Node<'d>>>(&self, context: &Context<'d>, node: N) -> u8 {
        self.eval_strict_stru32(context, node)
            .try_into().unwrap()
    }
    fn eval_strict_strf64<'d, N: Into<Node<'d>>>(&self, context: &Context<'d>, node: N) -> f64 {
        let string = self.eval_strict_string(context, node);
        match string.trim().parse() {
            Ok(v) => v,
            Err(e) => panic!("failed to parse {:?} as f64: {}", string, e),
        }
    }
}
impl XPathExt for XPath {
    fn eval_strict<'d, N: Into<Node<'d>>>(&self, context: &Context<'d>, node: N) -> Value<'d> {
        self.evaluate(context, node)
            .expect("failed to evaluate XPath")
    }
}

pub(crate) trait ValueExt<'d> {
    fn into_nodeset_strict(self) -> Nodeset<'d>;
    fn into_string_strict(self) -> String;
}
impl<'d> ValueExt<'d> for Value<'d> {
    fn into_nodeset_strict(self) -> Nodeset<'d> {
        match self {
            Self::Nodeset(ns) => ns,
            _ => panic!("{:?} is not a nodeset", self),
        }
    }
    fn into_string_strict(self) -> String {
        match self {
            Self::String(s) => s,
            _ => panic!("{:?} is not a string", self),
        }
    }
}
