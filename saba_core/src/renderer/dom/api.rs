use crate::renderer::dom::node::Element;
use crate::renderer::dom::node::ElementKind;
use crate::renderer::dom::node::Node;
use crate::renderer::dom::node::NodeKind;
use alloc::rc::Rc;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::cell::RefCell;

pub fn get_target_element_node(
    node: Option<Rc<RefCell<Node>>>,
    element_kind: ElementKind,
) -> Option<Rc<RefCell<Node>>> {
    match node {
        Some(n) => {
            // impl<T: Display + ?Sized> ToString for T
            // ElementKind は Display を実装していないため .to_string() は Debug 表現 (例: "Html") を返す点に注意
            if n.borrow().kind()
                == NodeKind::Element(Element::new(&element_kind.to_string(), Vec::new()))
            {
                return Some(n.clone());
            }

            // 子供のノードに対して探索
            let result1 = get_target_element_node(n.borrow().first_child(), element_kind);
            // 兄弟のノードに対して探索
            let result2 = get_target_element_node(n.borrow().next_sibling(), element_kind);

            if result1.is_none() && result2.is_none() {
                return None;
            }
            if result1.is_none() {
                return result2;
            }
            result1
        }
        None => None,
    }
}
