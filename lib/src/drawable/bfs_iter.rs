use crate::drawable::drawable_content::DrawableContent;
use crate::drawable::drawable_element::DrawableElement;
use std::collections::VecDeque;

pub struct BfsIter<'a> {
    queue: VecDeque<&'a dyn DrawableContent>,
}

impl<'a> Iterator for BfsIter<'a> {
    type Item = &'a dyn DrawableContent;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.queue.pop_front()?;
        for child in node.content() {
            self.queue.push_back(child);
        }
        Some(node)
    }
}

pub fn bfs<'a>(root: &'a dyn DrawableContent) -> BfsIter<'a> {
    let mut queue = VecDeque::new();
    queue.push_back(root);
    BfsIter { queue }
}

pub fn bfs_elements<'a>(
    root: &'a dyn DrawableContent,
) -> impl Iterator<Item = DrawableElement> + 'a {
    bfs(root).flat_map(|node| node.elements())
}
