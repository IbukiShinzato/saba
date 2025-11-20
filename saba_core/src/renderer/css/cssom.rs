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

    // トークン列からCSSOMを構築する
    pub fn parse_stylesheet(&mut self) -> StyleSheet {
        // StyleSheet（ルートノード）構造体のインスタンスを作成する
        let mut sheet = StyleSheet::new();

        // トークン列からルールのリストを作成し、StyleSheetのフィールドに設定する
        sheet.set_rules(self.consume_list_of_rules());
        sheet
    }

    // 複数のルールの解釈
    fn consume_list_of_rules(&mut self) -> Vec<QualifiedRule> {
        // 空のベクタを作成
        let mut rules = Vec::new();

        loop {
            let token = match self.t.peek() {
                Some(t) => t,
                None => return rules,
            };

            match token {
                // AtKeywordトークンが出てきた場合、他のCSSをimportする@import、メディアクエリを表す@mediaなどのルールが始まることを表す
                CssToken::AtKeyword(_keyword) => {
                    let _rule = self.consume_qualified_rule();
                }
                _ => {
                    // １つのルールを解釈し、ベクタに追加する
                    let rule = self.consume_qualified_rule();
                    match rule {
                        Some(r) => rules.push(r),
                        None => return rules,
                    }
                }
            }
        }
    }

    // 一つのルールの解釈
    fn consume_qualified_rule(&mut self) -> Option<QualifiedRule> {
        let mut rule = QualifiedRule::new();

        loop {
            let token = match self.t.peek() {
                Some(t) => t,
                None => return None,
            };

            match token {
                // 開き波括弧（{）の時
                CssToken::OpenCurly => {
                    assert_eq!(self.t.next(), Some(CssToken::OpenCurly));
                    rule.set_declarations(self.consume_list_of_declarations());
                    return Some(rule);
                }
                // それ以外はセレクタ
                _ => {
                    rule.set_selector(self.consume_selector());
                }
            }
        }
    }

    // セレクタの解釈
    fn consume_selector(&mut self) -> Selector {
        let token = match self.t.next() {
            Some(t) => t,
            None => panic!("should have a token but got None"),
        };

        match token {
            // #に続くものはID名でIDセレクタと呼ぶ
            // #は省略する #id => id
            // IDセレクタを作成
            CssToken::HashToken(value) => Selector::IdSelector(value[1..].to_string()),
            // .ならクラスセレクタを返す
            CssToken::Delim(delim) => {
                if delim == '.' {
                    return Selector::ClassSelector(self.consume_ident());
                }
                panic!("Parse error: {:?} is an unexpected token.", token);
            }
            // a:hover（マウスを置くと反応）のようなセレクタはタイプセレクタとしても扱うため、もしコロンが出てきた場合は宣言ブロックの開始直前までトークンを進める
            // a:active（クリックすると反応）などもある
            CssToken::Ident(ident) => {
                if self.t.peek() == Some(&CssToken::Colon) {
                    while self.t.peek() != Some(&CssToken::OpenCurly) {
                        self.t.next();
                    }
                }
                Selector::TypeSelector(ident.to_string())
            }
            CssToken::AtKeyword(_keyword) => {
                // @から始まるルールを無視するために、宣言ブロックの開始直前までトークンを進める
                while self.t.peek() != Some(&CssToken::OpenCurly) {
                    self.t.next();
                }
                Selector::UnknownSelector
            }
            _ => {
                self.t.next();
                Selector::UnknownSelector
            }
        }
    }

    // 複数の宣言の解釈
    fn consume_list_of_declarations(&mut self) -> Vec<Declaration> {
        // 宣言のベクタを初期化
        let mut declarations = Vec::new();

        loop {
            let token = match self.t.peek() {
                Some(t) => t,
                None => return declarations,
            };

            match token {
                CssToken::CloseCurly => {
                    assert_eq!(self.t.next(), Some(CssToken::CloseCurly));
                    return declarations;
                }
                CssToken::SemiColon => {
                    assert_eq!(self.t.next(), Some(CssToken::SemiColon));
                    // 一つの宣言が終了したので何もしない
                }
                // 識別子トークンの時（"p", "color", "red"など）
                CssToken::Ident(ref _ident) => {
                    if let Some(declaration) = self.consume_declaration() {
                        declarations.push(declaration);
                    }
                }
                _ => {
                    self.t.next();
                }
            }
        }
    }

    // 1つの宣言（プロパティと値のセット）の解釈
    fn consume_declaration(&mut self) -> Option<Declaration> {
        self.t.peek()?;

        // Declaration構造体を初期化する
        let mut declaration = Declaration::new();
        // Declaration構造体のプロパティに識別子を設定
        declaration.set_property(self.consume_ident());

        // もし次のトークンがコロンでない場合、パースエラーなので、Noneを返す
        // 想定だと property: value
        match self.t.next() {
            Some(CssToken::Colon) => {}
            _ => return None,
        }

        // Declaration構造体の値にコンポーネント値を設定する
        declaration.set_value(self.consume_component_value());

        Some(declaration)
    }

    // 識別子の解釈
    fn consume_ident(&mut self) -> String {
        let token = match self.t.next() {
            Some(t) => t,
            None => panic!("should have a token but got None"),
        };

        match token {
            CssToken::Ident(ref ident) => ident.to_string(),
            _ => {
                panic!("Parse error: {:?} is an unexpected token.", token);
            }
        }
    }

    // コンポーネント値の解釈
    // コンポーネント値はCSSのトークンと同等なので、存在をすることを確認
    fn consume_component_value(&mut self) -> ComponentValue {
        self.t
            .next()
            .expect("should have a token in consume_component_value")
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
    pub declarations: Vec<Declaration>,
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

    pub fn set_declarations(&mut self, declarations: Vec<Declaration>) {
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
pub struct Declaration {
    pub property: String,
    pub value: ComponentValue,
}

impl Default for Declaration {
    fn default() -> Self {
        Self::new()
    }
}

impl Declaration {
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

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    // 空文字のテスト
    #[test]
    fn test_empty() {
        // Task: 下3行を共通化する、引数にstyleの文字列を入れる
        let style = "".to_string();
        let t = CssTokenizer::new(style);
        let cssom = CssParser::new(t).parse_stylesheet();

        assert_eq!(cssom.rules.len(), 0);
    }

    // 1つのルールのテスト
    #[test]
    fn test_one_rule() {
        let style = "p { color: red; }".to_string();
        let t = CssTokenizer::new(style);
        let cssom = CssParser::new(t).parse_stylesheet();

        let mut rule = QualifiedRule::new();
        rule.set_selector(Selector::TypeSelector("p".to_string()));
        let mut declaration = Declaration::new();
        declaration.set_property("color".to_string());
        declaration.set_value(ComponentValue::Ident("red".to_string()));
        rule.set_declarations(vec![declaration]);

        let expected = [rule];
        assert_eq!(cssom.rules.len(), 1);

        // なるべくindex参照は避けたいのでzipを使用
        for (exp, rule) in expected.into_iter().zip(cssom.rules) {
            assert_eq!(exp, rule);
        }
    }

    // IDセレクタのテスト
    #[test]
    fn test_id_selector() {
        let style = "#id { color: blue }".to_string();
        let t = CssTokenizer::new(style);
        let cssom = CssParser::new(t).parse_stylesheet();

        let mut rule = QualifiedRule::new();
        rule.set_selector(Selector::IdSelector("id".to_string()));
        let mut declaration = Declaration::new();
        declaration.set_property("color".to_string());
        declaration.set_value(ComponentValue::Ident("blue".to_string()));
        rule.set_declarations(vec![declaration]);

        let expected = [rule];
        assert_eq!(cssom.rules.len(), expected.len());

        for (exp, rule) in expected.into_iter().zip(cssom.rules) {
            assert_eq!(exp, rule)
        }
    }

    // クラスセレクタのテスト
    #[test]
    fn test_class_selector() {
        let style = ".class { color: green }".to_string();
        let t = CssTokenizer::new(style);
        let cssom = CssParser::new(t).parse_stylesheet();

        let mut rule = QualifiedRule::new();
        rule.set_selector(Selector::ClassSelector("class".to_string()));
        let mut declaration = Declaration::new();
        declaration.set_property("color".to_string());
        declaration.set_value(ComponentValue::Ident("green".to_string()));
        rule.set_declarations(vec![declaration]);

        let expected = [rule];
        assert_eq!(cssom.rules.len(), expected.len());

        for (exp, rule) in expected.into_iter().zip(cssom.rules.into_iter()) {
            assert_eq!(exp, rule);
        }
    }

    // 複数のルールのテスト
    #[test]
    fn test_multiple_rules() {
        let style = "p { content: \"Hey\"; } h1 { font-size: 40; color: yellow; }".to_string();
        let t = CssTokenizer::new(style);
        let cssom = CssParser::new(t).parse_stylesheet();

        let mut rule1 = QualifiedRule::new();
        rule1.set_selector(Selector::TypeSelector("p".to_string()));
        let mut declaration1 = Declaration::new();
        declaration1.set_property("content".to_string());
        declaration1.set_value(ComponentValue::StringToken("Hey".to_string()));
        rule1.set_declarations(vec![declaration1]);

        // 宣言が２つ
        let mut rule2 = QualifiedRule::new();
        rule2.set_selector(Selector::TypeSelector("h1".to_string()));
        let mut declaration2 = Declaration::new();
        declaration2.set_property("font-size".to_string());
        declaration2.set_value(ComponentValue::Number(40.0));
        let mut declaration3 = Declaration::new();
        declaration3.set_property("color".to_string());
        declaration3.set_value(ComponentValue::Ident("yellow".to_string()));
        rule2.set_declarations(vec![declaration2, declaration3]);

        let expected = [rule1, rule2];
        assert_eq!(cssom.rules.len(), expected.len());

        for (exp, rule) in expected.into_iter().zip(cssom.rules.into_iter()) {
            assert_eq!(exp, rule);
        }
    }
}
