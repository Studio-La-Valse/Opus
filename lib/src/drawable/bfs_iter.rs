use crate::drawable::content::Content;
use crate::drawable::element::Element;
use std::collections::VecDeque;

pub struct BfsIter<'a> {
    queue: VecDeque<&'a dyn Content>,
}

impl<'a> Iterator for BfsIter<'a> {
    type Item = &'a dyn Content;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.queue.pop_front()?;
        for child in node.content() {
            self.queue.push_back(child);
        }
        Some(node)
    }
}

pub fn bfs<'a>(root: &'a dyn Content) -> BfsIter<'a> {
    let mut queue = VecDeque::new();
    queue.push_back(root);
    BfsIter { queue }
}

pub fn bfs_elements<'a>(root: &'a dyn Content) -> impl Iterator<Item = Element> + 'a {
    bfs(root).flat_map(|node| node.elements())
}
