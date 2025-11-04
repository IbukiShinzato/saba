use crate::renderer::dom::node::Node;
use crate::renderer::dom::node::Window;
use crate::renderer::html::token::HtmlToken;
use crate::renderer::html::token::HtmlTokenizer;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;

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

impl HtmlParser {
    pub fn construct_tree(&mut self) -> Rc<RefCell<Window>> {
        let mut token = self.t.next();

        // tokenを取得できる限りloop
        while token.is_some() {
            match self.mode {
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
                            if c == ' ' || c == '\n' {
                                
                            }
                        }
                    }
                },
                InsertionMode::BeforeHead => todo!(),
                InsertionMode::InHead => todo!(),
                InsertionMode::AfterHead => todo!(),
                InsertionMode::InBody => todo!(),
                InsertionMode::Text => todo!(),
                InsertionMode::AfterBody => todo!(),
                InsertionMode::AfterAfterBody => todo!(),
            }
        }

        self.window.clone()
    }
}
