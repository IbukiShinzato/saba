use crate::renderer::html::attribute::Attribute;
use alloc::format;
use alloc::rc::Rc;
use alloc::rc::Weak;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::str::FromStr;

#[derive(Debug, Clone)]
pub struct Window {
    document: Rc<RefCell<Node>>,
}

impl Window {
    pub fn new() -> Self {
        let window = Self {
            document: Rc::new(RefCell::new(Node::new(NodeKind::Document))),
        };

        window
            .document
            .borrow_mut()
            .set_window(Rc::downgrade(&Rc::new(RefCell::new(window.clone()))));

        window
    }

    pub fn document(&self) -> Rc<RefCell<Node>> {
        self.document.clone()
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    // ノードの種類
    pub kind: NodeKind,
    // DOMツリーを持つウィンドウ(弱い参照)
    window: Weak<RefCell<Window>>,
    // ノードの親ノード（弱い参照）
    parent: Weak<RefCell<Node>>,
    // ノードの1番初めの子ノード
    first_child: Option<Rc<RefCell<Node>>>,
    // ノードの最後の子ノード（弱い参照）
    last_child: Weak<RefCell<Node>>,
    // ノードの前の兄弟ノード（弱い参照）
    previous_sibling: Weak<RefCell<Node>>,
    // ノードの次の兄弟ノード
    next_sibling: Option<Rc<RefCell<Node>>>,
}

// ノードの種類で比較
impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

// 双方向リンクを持つ木構造ノード
impl Node {
    // コンストラクタ
    pub fn new(kind: NodeKind) -> Self {
        Self {
            kind,
            window: Weak::new(),
            parent: Weak::new(),
            first_child: None,
            last_child: Weak::new(),
            previous_sibling: Weak::new(),
            next_sibling: None,
        }
    }

    // セッター
    pub fn set_window(&mut self, window: Weak<RefCell<Window>>) {
        self.window = window;
    }

    // セッター
    pub fn set_parent(&mut self, parent: Weak<RefCell<Node>>) {
        self.parent = parent;
    }

    // ゲッター
    pub fn parent(&self) -> Weak<RefCell<Node>> {
        self.parent.clone()
    }

    // セッター
    pub fn set_first_child(&mut self, first_child: Option<Rc<RefCell<Node>>>) {
        self.first_child = first_child;
    }

    // ゲッター
    pub fn first_child(&self) -> Option<Rc<RefCell<Node>>> {
        self.first_child.as_ref().cloned()
    }

    // セッター
    pub fn set_last_child(&mut self, last_child: Weak<RefCell<Node>>) {
        self.last_child = last_child;
    }

    // ゲッター
    pub fn last_child(&self) -> Weak<RefCell<Node>> {
        self.last_child.clone()
    }

    // セッター
    pub fn set_previous_sibling(&mut self, previous_sibling: Weak<RefCell<Node>>) {
        self.previous_sibling = previous_sibling;
    }

    // ゲッター
    pub fn previous_sibling(&self) -> Weak<RefCell<Node>> {
        self.previous_sibling.clone()
    }

    // セッター
    pub fn set_next_sibling(&mut self, next_sibling: Option<Rc<RefCell<Node>>>) {
        self.next_sibling = next_sibling;
    }

    // ゲッター
    pub fn next_sibling(&self) -> Option<Rc<RefCell<Node>>> {
        self.next_sibling.as_ref().cloned()
    }

    // ゲッター
    pub fn kind(&self) -> NodeKind {
        self.kind.clone()
    }

    // 要素（Element）の取得
    pub fn get_element(&self) -> Option<Element> {
        match self.kind {
            NodeKind::Document | NodeKind::Text(_) => None,
            NodeKind::Element(ref e) => Some(e.clone()),
        }
    }

    // 要素（Element）の種類
    pub fn element_kind(&self) -> Option<ElementKind> {
        match self.kind {
            NodeKind::Document | NodeKind::Text(_) => None,
            NodeKind::Element(ref e) => Some(e.kind()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum NodeKind {
    // DOMツリーのルート要素
    Document,
    // DOMツリー内の要素ノード
    Element(Element),
    // 要素内のテキストコンテンツ
    Text(String),
}

impl PartialEq for NodeKind {
    fn eq(&self, other: &Self) -> bool {
        match &self {
            // matches!マクロを使うことによって指定されたパターンのいずれかに一致するかを見ることができる
            NodeKind::Document => matches!(other, NodeKind::Document),
            NodeKind::Element(e1) => match &other {
                NodeKind::Element(e2) => e1.kind == e2.kind,
                _ => false,
            },
            NodeKind::Text(_) => matches!(other, NodeKind::Text(_)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    kind: ElementKind,
    attributes: Vec<Attribute>,
}

impl Element {
    pub fn new(element_name: &str, attributes: Vec<Attribute>) -> Self {
        Self {
            kind: ElementKind::from_str(element_name)
                .expect("failed to convert string to ElementKind"),
            attributes,
        }
    }

    pub fn kind(&self) -> ElementKind {
        self.kind
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ElementKind {
    // <html>
    Html,
    // <head>
    Head,
    // <style>
    Style,
    // <script>
    Script,
    // <body>
    Body,
    // <p>
    P,
    // <h1>
    H1,
    // <h2>
    H2,
    // <a>
    A,
}

impl FromStr for ElementKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "html" => Ok(ElementKind::Html),
            "head" => Ok(ElementKind::Head),
            "style" => Ok(ElementKind::Style),
            "script" => Ok(ElementKind::Script),
            "body" => Ok(ElementKind::Body),
            "p" => Ok(ElementKind::P),
            "h1" => Ok(ElementKind::H1),
            "h2" => Ok(ElementKind::H2),
            "a" => Ok(ElementKind::A),
            _ => Err(format!("unimplemented element name {:?}", s)),
        }
    }
}
