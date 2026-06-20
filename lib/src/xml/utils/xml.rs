use roxmltree::Node;

pub trait N<'a> {
    fn req_attribute(&self, name: &str) -> &str;
    fn req_element(&self, name: &str) -> Node<'a, 'a>;
    fn req_text(&self) -> &str;
    fn req_child(&self, name: &str) -> Node<'a, 'a>;
    fn req_f32(&self) -> f32;
    fn req_i32(&self) -> i32;
    fn req_u32(&self) -> u32;

    fn get_child(&self, name: &str) -> Option<Node<'a, 'a>>;
    fn get_attribute(&self, name: &str) -> Option<&str>;

    fn has_tag(&self, name: &str) -> bool;
}

impl<'a> N<'a> for Node<'a, 'a> {
    fn req_attribute(&self, name: &str) -> &str {
        self.attribute(name).unwrap_or_else(|| {
            panic!(
                "attribute '{}' not found under '{}'",
                name,
                self.tag_name().name()
            )
        })
    }

    fn req_element(&self, name: &str) -> Node<'a, 'a> {
        self.children()
            .find(|n| n.is_element() && n.has_tag_name(name))
            .unwrap_or_else(|| {
                panic!(
                    "cannot find element '{}' under <{}>",
                    name,
                    self.tag_name().name()
                )
            })
    }

    fn req_text(&self) -> &str {
        self.text()
            .unwrap_or_else(|| panic!("No text found under element"))
    }

    fn req_child(&self, name: &str) -> Node<'a, 'a> {
        self.children()
            .find(|n| n.has_tag(name))
            .unwrap_or_else(|| panic!("Missing required element <{}>", name))
    }

    fn req_f32(&self) -> f32 {
        self.text()
            .unwrap_or_else(|| panic!("Missing text value"))
            .trim()
            .parse::<f32>()
            .unwrap_or_else(|_| panic!("Invalid f64 value"))
    }

    fn req_i32(&self) -> i32 {
        self.text()
            .unwrap_or_else(|| panic!("Missing text value"))
            .trim()
            .parse::<i32>()
            .unwrap_or_else(|_| panic!("Invalid f64 value"))
    }

    fn req_u32(&self) -> u32 {
        self.text()
            .unwrap_or_else(|| panic!("Missing text value"))
            .trim()
            .parse::<u32>()
            .unwrap_or_else(|_| panic!("Invalid f64 value"))
    }

    fn get_child(&self, name: &str) -> Option<Node<'a, 'a>> {
        self.children()
            .find(|n| n.is_element() && n.has_tag_name(name))
    }

    fn get_attribute(&self, name: &str) -> Option<&str> {
        self.attribute(name)
    }

    fn has_tag(&self, name: &str) -> bool {
        self.tag_name().name() == name
    }
}
