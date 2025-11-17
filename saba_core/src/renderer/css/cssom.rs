use crate::renderer::css::token::CssToken;
use crate::renderer::css::token::CssTokenizer;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::iter::Peekable;

// コンポーネント値ノード（Component value）
// CSSのトークンと同等
pub type ComponentValue = CssToken;

// CSSOMを構築するパーサー
#[derive(Debug, Clone)]
pub struct CssParser {
    t: Peekable<CssTokenizer>,
}

impl CssParser {
    pub fn new(t: CssTokenizer) -> Self {
        Self { t: t.peekable() }
    }
}

// ルートノード（StyleSheet）
// CSSOMの1番上のノード
#[derive(Debug, Clone, PartialEq)]
pub struct StyleSheet {
    pub rules: Vec<QualifiedRule>,
}

impl Default for StyleSheet {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleSheet {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn set_rules(&mut self, rules: Vec<QualifiedRule>) {
        self.rules = rules;
    }
}

// ルールノード（QualifiedRule）
// セレクタ（Selector）と宣言（Declaration）のベクタを持つ
#[derive(Debug, Clone, PartialEq)]
pub struct QualifiedRule {
    pub selector: Selector,
    pub declarations: Vec<Declarations>,
}

impl Default for QualifiedRule {
    fn default() -> Self {
        Self::new()
    }
}

impl QualifiedRule {
    pub fn new() -> Self {
        Self {
            selector: Selector::TypeSelector("".to_string()),
            declarations: Vec::new(),
        }
    }

    pub fn set_selector(&mut self, selector: Selector) {
        self.selector = selector;
    }

    pub fn set_declarations(&mut self, declarations: Vec<Declarations>) {
        self.declarations = declarations;
    }
}

// セレクタノード（Selector)
// タグ名で指定するTypeSelector、クラス名で指定するClassSelector、ID名で指定するIdSelector
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selector {
    TypeSelector(String),
    ClassSelector(String),
    IdSelector(String),
    UnknownSelector,
}

// 宣言ノード（Decralation）
// プロパティ（property）と値（value）のセット
#[derive(Debug, Clone, PartialEq)]
pub struct Declarations {
    pub property: String,
    pub value: ComponentValue,
}

impl Default for Declarations {
    fn default() -> Self {
        Self::new()
    }
}

impl Declarations {
    pub fn new() -> Self {
        Self {
            property: String::new(),
            value: ComponentValue::Ident(String::new()),
        }
    }

    pub fn set_property(&mut self, property: String) {
        self.property = property;
    }

    pub fn set_value(&mut self, value: ComponentValue) {
        self.value = value;
    }
}
