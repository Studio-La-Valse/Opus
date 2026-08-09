use roxmltree::Node;
use std::str::FromStr;

/// Extension trait for parsing values out of string slices and XML nodes.
pub trait ReqParse {
    /// Parses the content into type `T`, panicking with context if parsing fails.
    fn req_parse<T>(&self) -> T
    where
        T: FromStr,
        <T as FromStr>::Err: std::fmt::Debug;
}

impl ReqParse for str {
    fn req_parse<T>(&self) -> T
    where
        T: FromStr,
        <T as FromStr>::Err: std::fmt::Debug,
    {
        self.trim()
            .parse::<T>()
            .unwrap_or_else(|err| panic!("Failed to parse '{self}' into target type: {err:?}"))
    }
}

impl<'a, 'input> ReqParse for Node<'a, 'input> {
    fn req_parse<T>(&self) -> T
    where
        T: FromStr,
        <T as FromStr>::Err: std::fmt::Debug,
    {
        self.req_text().req_parse()
    }
}

/// Helper utilities for non-mutating `roxmltree::Node` navigation and retrieval.
pub trait NodeUtils<'a, 'input> {
    fn req_attribute(&self, name: &str) -> &'input str;
    fn get_attribute(&self, name: &str) -> Option<&'input str>;
    fn req_child(&self, name: &str) -> Node<'a, 'input>;
    fn get_child(&self, name: &str) -> Option<Node<'a, 'input>>;
    fn get_children(&self, name: &str) -> Vec<Node<'a, 'input>>;
    fn req_text(&self) -> &'input str;
    fn has_tag(&self, name: &str) -> bool;
    fn has_child(&self, name: &str) -> bool;
}

impl<'a, 'input> NodeUtils<'a, 'input> for Node<'a, 'input>
where
    'a: 'input, // <--- Add this bound here
{
    fn req_attribute(&self, name: &str) -> &'input str {
        match self.attribute(name) {
            Some(attr) => attr,
            None => panic!(
                "Attribute '{name}' missing on element <{}>",
                self.tag_name().name()
            ),
        }
    }

    fn get_attribute(&self, name: &str) -> Option<&'input str> {
        self.attribute(name)
    }

    fn req_child(&self, name: &str) -> Node<'a, 'input> {
        let parent_name = self.tag_name().name();
        self.get_child(name)
            .unwrap_or_else(|| panic!("Child element <{name}> missing under <{parent_name}>"))
    }

    fn get_child(&self, name: &str) -> Option<Node<'a, 'input>> {
        self.children()
            .find(|n| n.is_element() && n.has_tag_name(name))
    }

    fn get_children(&self, name: &str) -> Vec<Node<'a, 'input>> {
        self.children()
            .filter(|n| n.is_element() && n.has_tag_name(name))
            .collect()
    }

    fn req_text(&self) -> &'input str {
        match self.text() {
            Some(text) => text,
            None => panic!(
                "Text content missing in element <{}>",
                self.tag_name().name()
            ),
        }
    }

    fn has_tag(&self, name: &str) -> bool {
        self.has_tag_name(name)
    }
    fn has_child(&self, name: &str) -> bool {
        self.get_child(name).is_some()
    }
}
