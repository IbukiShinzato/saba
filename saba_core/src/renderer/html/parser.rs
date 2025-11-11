use crate::renderer::dom::node::Element;
use crate::renderer::dom::node::ElementKind;
use crate::renderer::dom::node::Node;
use crate::renderer::dom::node::NodeKind;
use crate::renderer::dom::node::Window;
use crate::renderer::html::attribute::Attribute;
use crate::renderer::html::token::HtmlToken;
use crate::renderer::html::token::HtmlTokenizer;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InsertionMode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    AfterHead,
    InBody,
    Text,
    AfterBody,
    AfterAfterBody,
}

#[derive(Debug, Clone)]
pub struct HtmlParser {
    // DOMツリーのルートノードを持つWindowオブジェクトを格納するフィールド
    window: Rc<RefCell<Window>>,
    // 状態遷移で使用される挿入モード
    mode: InsertionMode,
    // とある状態に遷移したときに、以前の挿入モードを保存するために使用するフィールド
    original_insertion_mode: InsertionMode,
    // HTMLの構文解析中にブラウザが使用するスタック
    stack_of_open_elements: Vec<Rc<RefCell<Node>>>,
    // HtmlTokenizerの構造体　次のトークンはt.next()で取得
    t: HtmlTokenizer,
}

// -- HTMLファイルの中身 --

// <html>
// <head>
//     <style>
//         body { background-color: khaki; }
//     </style>
// </head>
// <body>
//     <h1 id="title">H1 title</h1>
//     <h2 class="class">H2 title</h2>
//     <p>Test text.</p>
//     <p>
//         <a href="example.com">Link 1</a>
//         <a href="example.com">Link 2</a>
//     </p>
// </body>
// </html>

impl HtmlParser {
    pub fn new(t: HtmlTokenizer) -> Self {
        Self {
            window: Rc::new(RefCell::new(Window::new())),
            mode: InsertionMode::Initial,
            original_insertion_mode: InsertionMode::Initial,
            stack_of_open_elements: Vec::new(),
            t,
        }
    }

    fn create_element(&self, tag: &str, attributes: Vec<Attribute>) -> Node {
        Node::new(NodeKind::Element(Element::new(tag, attributes)))
    }

    fn insert_element(&mut self, tag: &str, attributes: Vec<Attribute>) {
        let window = self.window.borrow();

        // 現在の開いている要素スタックの最後のノードを取得
        // スタックがからの場合は、ルート要素が現在参照しているノード
        let current = match self.stack_of_open_elements.last() {
            Some(n) => n.clone(),
            None => window.document(),
        };

        // 新しい要素ノードを作成
        let node = Rc::new(RefCell::new(self.create_element(tag, attributes)));

        // 現在参照しているノードにすでに子要素がある場合
        if current.borrow().first_child().is_some() {
            // 最後の兄弟ノードを探索
            let mut last_sibiling = current.borrow().first_child();
            loop {
                last_sibiling = match last_sibiling {
                    Some(ref node) => {
                        if node.borrow().next_sibling().is_some() {
                            node.borrow().next_sibling()
                        } else {
                            break;
                        }
                    }
                    None => unimplemented!("last_sibiling should be Some"),
                }
            }

            // 新しいノードを最後の直後に挿入
            last_sibiling
                .unwrap()
                .borrow_mut()
                .set_next_sibling(Some(node.clone()));

            node.borrow_mut().set_previous_sibling(Rc::downgrade(
                &current
                    .borrow()
                    .first_child()
                    .expect("failed to get a first child"),
            ));
        } else {
            // 兄弟ノードが存在しない場合は新しいノードを現在参照しているノードの最初の子要素に設定する
            current.borrow_mut().set_first_child(Some(node.clone()));
        }

        // 現在参照しているノードの最後の子ノードを新しいノードに設定する
        current.borrow_mut().set_last_child(Rc::downgrade(&node));
        // 新しいノードの親を現在参照しているノードに設定
        node.borrow_mut().set_parent(Rc::downgrade(&current));

        // 新しいノードを開いている要素スタックに追加
        self.stack_of_open_elements.push(node);
    }

    // 開いている要素のスタック管理
    fn pop_current_node(&mut self, element_kind: ElementKind) -> bool {
        let current = match self.stack_of_open_elements.last() {
            Some(n) => n,
            None => return false,
        };

        if current.borrow().element_kind() == Some(element_kind) {
            self.stack_of_open_elements.pop();
            return true;
        }

        false
    }

    // stack_of_open_elementスタックから特定の種類の要素が現れるまでノードを取り出し続ける
    fn pop_until(&mut self, element_kind: ElementKind) {
        assert!(
            self.contain_in_stack(element_kind),
            "stack doesn't have an element {:?}",
            element_kind
        );

        loop {
            let current = match self.stack_of_open_elements.pop() {
                Some(n) => n,
                None => return,
            };

            if current.borrow().element_kind() == Some(element_kind) {
                return;
            }
        }
    }

    // stack_of_open_elements スタックに存在する全ての要素を確認して、特定の種類の要素がある場合にtrueを返す
    fn contain_in_stack(&mut self, element_kind: ElementKind) -> bool {
        let contain = self
            .stack_of_open_elements
            .iter()
            .any(|element| element.borrow().element_kind() == Some(element_kind));

        contain
    }

    // 新しいテキストノードを作成
    fn create_char(&self, c: char) -> Node {
        let s = String::from(c);
        Node::new(NodeKind::Text(s))
    }

    fn insert_char(&mut self, c: char) {
        let current = match self.stack_of_open_elements.last() {
            Some(n) => n.clone(),
            None => return,
        };

        // 現在参照しているノードがテキストノードの場合、そのノードに文字を追加する
        if let NodeKind::Text(ref mut s) = current.borrow_mut().kind {
            s.push(c);
            return;
        }

        // 改行文字や空白文字の時はテキストノードを追加しない
        if c == '\n' || c == ' ' {
            return;
        }

        // 新しいテキストノードの作成
        let node = Rc::new(RefCell::new(self.create_char(c)));

        // すでに子要素がある場合
        if current.borrow().first_child().is_some() {
            // 新しいテキストノードをその直後に挿入
            current
                .borrow()
                .first_child()
                .unwrap()
                .borrow_mut()
                .set_next_sibling(Some(node.clone()));
            node.borrow_mut().set_previous_sibling(Rc::downgrade(
                &current
                    .borrow()
                    .first_child()
                    .expect("failed to get a first child"),
            ));
        } else {
            // 新しいテキストノードを現在参照しているノードの最初の子要素として設定する
            current.borrow_mut().set_first_child(Some(node.clone()));
        }

        // 現在参照しているノードの最後の子ノードを新しいノードに設定
        current.borrow_mut().set_last_child(Rc::downgrade(&node));
        // 新しいノードの親を現在参照しているノードに設定
        node.borrow_mut().set_parent(Rc::downgrade(&current));

        // 新しいノードを開いている要素スタックに追加
        self.stack_of_open_elements.push(node);
    }

    pub fn construct_tree(&mut self) -> Rc<RefCell<Window>> {
        let mut token = self.t.next();

        // tokenを取得できる限りloop
        while token.is_some() {
            match self.mode {
                // DOCTYPEトークンを扱う
                InsertionMode::Initial => {
                    // 本書では、DOCTYPEトークンをサポートしていないため、
                    // <!doctype html>のようなトークンは文字トークンとして
                    // 表せられる。
                    // 文字トークンは無視する
                    if let Some(HtmlToken::Char(_)) = token {
                        token = self.t.next();
                        continue;
                    }
                }

                // <html>の開始タグを取り扱う
                InsertionMode::BeforeHtml => {
                    match token {
                        Some(HtmlToken::Char(c)) => {
                            // 次のトークンが空白文字や改行ならスキップ
                            if c == ' ' || c == '\n' {
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::StartTag {
                            ref tag,
                            self_closing: _,
                            ref attributes,
                        }) => {
                            // タグの名前が<html>だったとき
                            if tag == "html" {
                                // 新しいノードを追加する
                                self.insert_element(tag, attributes.to_vec());
                                self.mode = InsertionMode::BeforeHead;
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::Eof) | None => return self.window.clone(),
                        _ => {}
                    }

                    // 自動的にHTMLの要素をDOMツリーに追加
                    self.insert_element("html", Vec::new());
                    self.mode = InsertionMode::BeforeHead;
                    continue;
                }

                // <head>の開始タグを取り扱う
                InsertionMode::BeforeHead => {
                    match token {
                        Some(HtmlToken::Char(c)) => {
                            // 次のトークンが空白文字や改行ならスキップ
                            if c == ' ' || c == '\n' {
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::StartTag {
                            ref tag,
                            self_closing: _,
                            ref attributes,
                        }) => {
                            // タグの名前が<head>だったとき
                            if tag == "head" {
                                self.insert_element(tag, attributes.to_vec());
                                self.mode = InsertionMode::InHead;
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::Eof) | None => return self.window.clone(),
                        _ => {}
                    }

                    self.insert_element("head", Vec::new());
                    self.mode = InsertionMode::InHead;
                    continue;
                }

                // <head>の終了タグ、<style>開始タグ、<script>開始タグを取り扱う
                InsertionMode::InHead => {
                    match token {
                        Some(HtmlToken::Char(c)) => {
                            // 次のトークンが空白文字や改行ならスキップ
                            if c == ' ' || c == '\n' {
                                self.insert_char(c);
                                token = self.t.next();
                                continue;
                            }
                        }
                        // <style>または<script>だったとき
                        Some(HtmlToken::StartTag {
                            ref tag,
                            self_closing: _,
                            ref attributes,
                        }) => {
                            if tag == "style" || tag == "script" {
                                // DOMツリーに新しいノードを追加して、Text状態に遷移
                                self.insert_element(tag, attributes.to_vec());
                                self.original_insertion_mode = self.mode;
                                self.mode = InsertionMode::Text;
                                token = self.t.next();
                                continue;
                            }

                            // 仕様書には定められていないが、このブラウザは仕様を
                            // 全て実装しているわけではないので、<head>が省略
                            // されているHTML文書を扱うために必要。これがないと
                            // <head>が省略されているHTML文書で無限ループが発生
                            if tag == "body" {
                                self.pop_until(ElementKind::Head);
                                self.mode = InsertionMode::AfterHead;
                                continue;
                            }

                            if let Ok(_element_kind) = ElementKind::from_str(tag) {
                                // スタックに保存されているノードを取り出しAfterHead状態に遷移
                                self.pop_until(ElementKind::Head);
                                self.mode = InsertionMode::AfterHead;
                                continue;
                            }
                        }
                        Some(HtmlToken::EndTag { ref tag }) => {
                            if tag == "head" {
                                self.mode = InsertionMode::AfterHead;
                                token = self.t.next();
                                self.pop_until(ElementKind::Head);
                                continue;
                            }
                        }
                        Some(HtmlToken::Eof) | None => return self.window.clone(),
                    }
                    // <meta>や<title>などのサポートしていないタグは無視
                    token = self.t.next();
                }

                // <body>開始タグを取り扱う
                InsertionMode::AfterHead => {
                    match token {
                        Some(HtmlToken::Char(c)) => {
                            if c == ' ' || c == '\n' {
                                self.insert_char(c);
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::StartTag {
                            ref tag,
                            self_closing: _,
                            ref attributes,
                        }) => {
                            if tag == "body" {
                                self.insert_element(tag, attributes.to_vec());
                                token = self.t.next();
                                self.mode = InsertionMode::InBody;
                                continue;
                            }
                        }
                        Some(HtmlToken::Eof) | None => return self.window.clone(),
                        _ => {}
                    }
                    self.insert_element("body", Vec::new());
                    self.mode = InsertionMode::InBody;
                    continue;
                }

                // <body>タグのコンテンツを扱う
                // 具体的には<div>タグ、<h1>タグ、<p>タグなど
                // Task: タグを増やす
                InsertionMode::InBody => {
                    match token {
                        Some(HtmlToken::StartTag {
                            ref tag,
                            self_closing: _,
                            ref attributes,
                        }) => match tag.as_str() {
                            // Task: ここ共通化できそう
                            "p" => {
                                // Elementノードを作成してDOMツリーに追加
                                self.insert_element(tag, attributes.to_vec());
                                token = self.t.next();
                                continue;
                            }
                            "h1" | "h2" => {
                                // Elementノードを作成してDOMツリーに追加
                                self.insert_element(tag, attributes.to_vec());
                                token = self.t.next();
                                continue;
                            }
                            "a" => {
                                // Elementノードを作成してDOMツリーに追加
                                self.insert_element(tag, attributes.to_vec());
                                token = self.t.next();
                                continue;
                            }
                            _ => {
                                token = self.t.next();
                            }
                        },
                        Some(HtmlToken::EndTag { ref tag }) => {
                            match tag.as_str() {
                                "body" => {
                                    self.mode = InsertionMode::AfterBody;
                                    token = self.t.next();

                                    if !self.contain_in_stack(ElementKind::Body) {
                                        // パースの失敗。トークンを無視する
                                        continue;
                                    }

                                    self.pop_until(ElementKind::Body);
                                    continue;
                                }
                                "html" => {
                                    if self.pop_current_node(ElementKind::Body) {
                                        self.mode = InsertionMode::AfterBody;
                                        assert!(self.pop_current_node(ElementKind::Html));
                                    } else {
                                        token = self.t.next();
                                    }
                                    continue;
                                }
                                // Task: ここ共通化できそう
                                "p" => {
                                    let element_kind = ElementKind::from_str(tag)
                                        .expect("failed to convert string to ElementKind");
                                    token = self.t.next();
                                    self.pop_until(element_kind);
                                    continue;
                                }
                                "h1" | "h2" => {
                                    let element_kind = ElementKind::from_str(tag)
                                        .expect("failed to convert string to ElementKind");
                                    token = self.t.next();
                                    self.pop_until(element_kind);
                                    continue;
                                }
                                "a" => {
                                    let element_kind = ElementKind::from_str(tag)
                                        .expect("failed to convert string to ElementKind");
                                    token = self.t.next();
                                    self.pop_until(element_kind);
                                    continue;
                                }
                                _ => {
                                    token = self.t.next();
                                }
                            }
                        }
                        Some(HtmlToken::Char(c)) => {
                            // テキストノードをDOMツリーに追加
                            self.insert_char(c);
                            token = self.t.next();
                            continue;
                        }
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        }
                    }
                }

                // <style>タグと<script>タグが開始した後の状態
                // 終了タグが出てくるまで、文字をテキストノードとしてDOMツリーに追加
                InsertionMode::Text => {
                    match token {
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        }
                        Some(HtmlToken::EndTag { ref tag }) => {
                            // ここ二つの処理を関数かできそう
                            if tag == "style" {
                                self.pop_until(ElementKind::Style);
                                self.mode = self.original_insertion_mode;
                                token = self.t.next();
                                continue;
                            }

                            if tag == "script" {
                                self.pop_until(ElementKind::Script);
                                self.mode = self.original_insertion_mode;
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::Char(c)) => {
                            self.insert_char(c);
                            token = self.t.next();
                            continue;
                        }
                        _ => {}
                    }

                    self.mode = self.original_insertion_mode;
                }

                // <html>終了タグを取り扱う
                InsertionMode::AfterBody => {
                    match token {
                        Some(HtmlToken::Char(_c)) => {
                            token = self.t.next();
                            continue;
                        }
                        Some(HtmlToken::EndTag { ref tag }) => {
                            if tag == "html" {
                                self.mode = InsertionMode::AfterAfterBody;
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        }
                        _ => {}
                    }

                    self.mode = InsertionMode::InBody;
                }

                // トークンが終了することを確認し、パースを終了する
                InsertionMode::AfterAfterBody => {
                    match token {
                        Some(HtmlToken::Char(_c)) => {
                            token = self.t.next();
                            continue;
                        }
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        }
                        _ => {}
                    }

                    // ブラウザでは間違ったHTML文書でもできるだけ解釈して表示しようとするので、すぐに実行中断されることはない
                    // パースの失敗
                    self.mode = InsertionMode::InBody;
                }
            }
        }

        self.window.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alloc::string::ToString;

    // 空文字のテスト
    // DOMツリーのルートにNodeKind::Documentを持つ
    #[test]
    fn test_empty() {
        let html = "".to_string();
        let t = HtmlTokenizer::new(html);
        let window = HtmlParser::new(t).construct_tree();
        let expected = Rc::new(RefCell::new(Node::new(NodeKind::Document)));

        assert_eq!(expected, window.borrow().document());
    }

    // bodyノードのテスト
    // <thml>タグ、<head>タグ、<body>タグを含む文字列のテスト
    // NodeKind::Document -> NodeKind::Element
    #[test]
    fn test_body() {
        let html = "<html><head></head><body></body></html>".to_string();
        let t = HtmlTokenizer::new(html);
        let window = HtmlParser::new(t).construct_tree();

        // ルートノードがNodeKind::Documentであるか
        let document = window.borrow().document();
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Document))),
            document
        );

        // その子供のノードはhtmlのNodeKind::Elementであるか
        let html = document
            .borrow()
            .first_child()
            .expect("failed to get a first child of document");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Element(Element::new(
                "html",
                Vec::new()
            ))))),
            html
        );

        // さらにその子供のノードはheadのNodeKind::Elementであるか
        let head = html
            .borrow()
            .first_child()
            .expect("failed to get a first child of html");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Element(Element::new(
                "head",
                Vec::new()
            ))))),
            head
        );

        // そしてheadノードの兄弟ノードはbodyのNodeKind::Elementであるか
        let body = head
            .borrow()
            .next_sibling()
            .expect("failed to get a next sibling of head");
        assert!(
            Rc::new(RefCell::new(Node::new(NodeKind::Element(Element::new(
                "body",
                Vec::new()
            ))))),
            body
        );
    }
}
