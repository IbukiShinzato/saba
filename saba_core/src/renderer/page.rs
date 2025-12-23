use crate::browser::Browser;
use crate::display_item::DisplayItem;
use crate::http::HttpResponse;
use crate::renderer::css::cssom::StyleSheet;
use crate::renderer::dom::node::Window;
use crate::renderer::html::parser::HtmlParser;
use crate::renderer::html::token::HtmlTokenizer;
use crate::renderer::layout::layout_view::LayoutView;
use alloc::rc::Rc;
use alloc::rc::Weak;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;

#[derive(Debug, Clone)]
pub struct Page {
    browser: Weak<RefCell<Browser>>,
    frame: Option<Rc<RefCell<Window>>>,
    style: Option<StyleSheet>,       // CSSOM
    layout_view: Option<LayoutView>, // レイアウトツリー
    display_items: Vec<DisplayItem>, // これ何？？
}

// 今開いているタブの中身を管理
impl Page {
    // コンストラクタ
    pub fn new() -> Self {
        Self {
            browser: Weak::new(),
            frame: None,
            style: None,
            layout_view: None,
            display_items: Vec::new(),
        }
    }

    // ブラウザ全体を管理する
    pub fn set_browser(&mut self, browser: Weak<RefCell<Browser>>) {
        self.browser = browser;
    }

    pub fn receive_response(&mut self, response: HttpResponse) {
        // 文字列からDOMツリーを構築
        self.create_frame(response.body());

        // レイアウトツリーを構築

        self.set_layout_view();

        // 描画ツリーを構築
        self.paint_tree();
    }

    // htmlファイルを入力して、parseして、DOMツリー構築
    fn create_frame(&mut self, html: String) {
        let html_tokenizer = HtmlTokenizer::new(html);
        let frame = HtmlParser::new(html_tokenizer).construct_tree();
        self.frame = Some(frame);
    }

    // domツリーを取得、cssomを引数にレイアウトツリーを構築
    fn set_layout_view(&mut self) {
        let dom = match &self.frame {
            Some(frame) => frame.borrow().document(),
            None => return,
        };

        let style = match self.style.clone() {
            Some(style) => style,
            None => return,
        };

        let layout_view = LayoutView::new(dom, &style);

        self.layout_view = Some(layout_view);
    }

    // display_itemに描画できるような形に変換した後代入
    fn paint_tree(&mut self) {
        if let Some(layout_view) = &self.layout_view {
            self.display_items = layout_view.paint();
        }
    }

    // 描画に必要な状態に変換
    pub fn display_items(&self) -> Vec<DisplayItem> {
        self.display_items.clone()
    }

    // display_itemsをクリア
    pub fn clear_display_items(&mut self) {
        self.display_items = Vec::new();
    }
}

impl Default for Page {
    fn default() -> Self {
        Self::new()
    }
}
