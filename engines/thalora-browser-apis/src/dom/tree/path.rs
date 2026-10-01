//! Selector paths for layout lookups.

use super::{DomTree, NodeId};

/// The same path `css_path_for_scraper_element` (`dom/document.rs`) builds
/// for a scraper element, so keys into the layout rect map match exactly.
///
/// Note the `:nth-child(n)` label carries the element's position among
/// same-tag siblings; both functions must keep agreeing on that.
pub(super) fn css_path(tree: &DomTree, id: NodeId) -> Option<String> {
    tree.tag(id)?;
    let mut parts: Vec<String> = Vec::new();
    let mut current = Some(id);

    while let Some(el) = current {
        let tag = tree.tag(el)?.to_lowercase();
        let parent = tree.parent(el);

        // Position among same-tag siblings
        let nth = match parent {
            Some(parent) => {
                let mut count = 0u32;
                for &sibling in tree.children(parent) {
                    if tree
                        .tag(sibling)
                        .is_some_and(|t| t.eq_ignore_ascii_case(&tag))
                    {
                        count += 1;
                        if sibling == el {
                            break;
                        }
                    }
                }
                count
            }
            None => 1,
        };

        // The root element gets no position
        let parent_element = parent.filter(|&p| tree.is_element(p));
        if parent_element.is_none() {
            parts.push(tag);
        } else {
            parts.push(format!("{}:nth-child({})", tag, nth));
        }
        current = parent_element;
    }

    parts.reverse();
    Some(parts.join(">"))
}
