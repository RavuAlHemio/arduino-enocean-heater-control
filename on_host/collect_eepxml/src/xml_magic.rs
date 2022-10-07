use sxd_document::QName;
use sxd_document::dom::{ChildOfElement, Document, Element};


pub trait DocumentExt<'d> {
    fn root_element(&self) -> Option<Element<'d>>;
}
impl<'d> DocumentExt<'d> for Document<'d> {
    fn root_element(&self) -> Option<Element<'d>> {
        self.root()
            .children().iter()
            .filter_map(|c| c.element())
            .nth(0)
    }
}

pub trait ElementExt<'d> {
    fn first_element_child_with_tag_name<'n, N: Into<QName<'n>>>(&self, name: N) -> Option<Element<'d>>;
    fn first_text_child(&self) -> Option<&'d str>;
}
impl<'d> ElementExt<'d> for Element<'d> {
    fn first_element_child_with_tag_name<'n, N: Into<QName<'n>>>(&self, name: N) -> Option<Element<'d>> {
        let qname = name.into();
        self.children().iter()
            .filter_map(|c| c.element())
            .filter(|e| e.name() == qname)
            .nth(0)
    }

    fn first_text_child(&self) -> Option<&'d str> {
        self.children().iter()
            .filter_map(|c| c.text())
            .nth(0)
            .map(|t| t.text())
    }
}


pub fn copy_element_children(source_element: &Element, target_element: &Element) {
    for child in source_element.children() {
        match child {
            ChildOfElement::Comment(comment) => {
                let new_comment = target_element.document().create_comment(comment.text());
                target_element.append_child(new_comment);
            },
            ChildOfElement::ProcessingInstruction(proc_ins) => {
                let new_proc_ins = target_element.document().create_processing_instruction(proc_ins.target(), proc_ins.value());
                target_element.append_child(new_proc_ins);
            },
            ChildOfElement::Text(txt) => {
                let new_text = target_element.document().create_text(txt.text());
                target_element.append_child(new_text);
            },
            ChildOfElement::Element(elem) => {
                // this is all the fun
                let new_elem = target_element.document().create_element(elem.name());
                for attrib in elem.attributes() {
                    new_elem.set_attribute_value(attrib.name(), attrib.value());
                }
                copy_element_children(&elem, &new_elem);
                target_element.append_child(new_elem);
            },
        }
    }
}
