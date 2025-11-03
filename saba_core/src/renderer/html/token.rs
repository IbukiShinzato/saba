use crate::renderer::html::attribute::Attribute;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlTokenizer {
    state: State,
    pos: usize,
    reconsume: bool,
    latest_token: Option<HtmlToken>,
    input: Vec<char>,
    buf: String,
}

impl HtmlTokenizer {
    pub fn new(html: String) -> Self {
        Self {
            state: State::Data,
            pos: 0,
            reconsume: false,
            latest_token: None,
            input: html.chars().collect(),
            buf: String::new(),
        }
    }

    fn is_eof(&self) -> bool {
        self.pos > self.input.len()
    }

    fn reconsume_input(&mut self) -> char {
        self.reconsume = false;
        self.input[self.pos - 1]
    }

    fn consume_next_input(&mut self) -> char {
        let c = self.input[self.pos];
        self.pos += 1;
        c
    }

    // StartTagまたはEndTagトークンを作成し、latest_tokenフィールドにセット
    fn create_tag(&mut self, start_tag_token: bool) {
        if start_tag_token {
            self.latest_token = Some(HtmlToken::StartTag {
                tag: String::new(),
                self_closing: false,
                attributes: Vec::new(),
            });
        } else {
            self.latest_token = Some(HtmlToken::EndTag { tag: String::new() });
        }
    }

    // create_tagメソッドによって作られた最後のトークン(latest_token)に対して、1文字をそのトークンのタグの名前として追加する
    fn append_tag_name(&mut self, c: char) {
        // latest_tokenがSomeの値を持っているか
        assert!(self.latest_token.is_some());

        // latest_tokenの可変参照を取得
        if let Some(t) = self.latest_token.as_mut() {
            match t {
                HtmlToken::StartTag {
                    ref mut tag,
                    self_closing: _,
                    attributes: _,
                }
                | HtmlToken::EndTag { ref mut tag } => tag.push(c),
                _ => panic!("`latest_token` should be either StartTag or EndTag"),
            }
        }
    }

    // create_tagメソッドによって作られた最後のトークン(latest_token)を返す
    fn take_latest_token(&mut self) -> Option<HtmlToken> {
        assert!(self.latest_token.is_some());

        let t = self.latest_token.as_ref().cloned();
        self.latest_token = None;
        assert!(self.latest_token.is_none());

        t
    }

    // create_tagメソッドによって作られた最後のトークン(latest_token)に属性を追加する
    fn start_new_attribute(&mut self) {
        assert!(self.latest_token.is_some());

        if let Some(t) = self.latest_token.as_mut() {
            match t {
                HtmlToken::StartTag {
                    tag: _,
                    self_closing: _,
                    ref mut attributes,
                } => {
                    attributes.push(Attribute::new());
                }
                _ => panic!("`latest_token` should be either StartTag"),
            }
        }
    }

    // create_tagメソッドによって作られた最後のトークン(latest_token)に属性の文字を追加する
    fn append_attribute(&mut self, c: char, is_name: bool) {
        assert!(self.latest_token.is_some());

        if let Some(t) = self.latest_token.as_mut() {
            match t {
                HtmlToken::StartTag {
                    tag: _,
                    self_closing: _,
                    ref mut attributes,
                } => {
                    let len = attributes.len();
                    assert!(len > 0);

                    attributes[len - 1].add_char(c, is_name);
                }

                _ => panic!("`latest_token` should be either StartTag"),
            }
        }
    }

    // create_tagメソッドによって作られた最後のトークン(latest_token)が開始タグの場合、フラグをtrueにする
    fn set_self_closing_flag(&mut self) {
        assert!(self.latest_token.is_some());

        if let Some(t) = self.latest_token.as_mut() {
            match t {
                HtmlToken::StartTag {
                    tag: _,
                    ref mut self_closing,
                    attributes: _,
                } => *self_closing = true,
                _ => panic!("`latest_token` should be either StartTag"),
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HtmlToken {
    // 開始タグ
    StartTag {
        tag: String,
        self_closing: bool,
        attributes: Vec<Attribute>,
    },

    // 終了タグ
    EndTag {
        tag: String,
    },

    // 文字
    Char(char),

    // ファイルの終了(Enf Of File)
    Eof,
}

// ここは後から状態を追加できそう
// https://html.spec.whatwg.org/multipage/parsing.html

// ステートマシンはデータ状態(Data state)から開始して、ファイルの終端が現れるまで処理が行われる
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    Data,
    TagOpen,
    EndTagOpen,
    TagName,
    BeforeAttributeName,
    AttributeName,
    AfterAttributeName,
    BeforeAttributeValue,
    AttributeValueDoubleQuoted,
    AttributeValueSingleQuoted,
    AttributeValueUnquoted,
    AfterAttributeValueQuoted,
    SelfClosingStartTag,
    ScriptData,
    ScriptDataLessThanSign,
    ScriptDataEndTagOpen,
    ScriptDataEndTagName,
    TemporaryBuffer,
}

// Iteratorの実装をすることによってnextが使えるようになり状態遷移で役に立つ
impl Iterator for HtmlTokenizer {
    // 型エイリアス
    type Item = HtmlToken;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.input.len() {
            return None;
        }

        loop {
            // reconsume_input
            // inputの文字列から前の位置(pos - 1)の文字を1文字返す
            // 読み込んだ後もposはそのまま
            // 文字を消費する

            // consume_next_input
            // inputの文字列から現在の位置(pos)の文字を1文字返す
            // 読み込んだ後にはposの位置を１つ進める
            // 文字を消費する

            // reconsumeによって現在の位置の文字を取得するか一つ前の位置の文字を取得するかで分岐
            let c = match self.reconsume {
                true => self.reconsume_input(),
                false => self.consume_next_input(),
            };

            match self.state {
                // 1つの文字を消費し、その文字の種類によって次の行動を決める
                State::Data => {
                    if c == '<' {
                        self.state = State::TagOpen;
                        continue;
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    return Some(HtmlToken::Char(c));
                }

                // タグを開始している時の状態
                State::TagOpen => {
                    if c == '/' {
                        self.state = State::EndTagOpen;
                        continue;
                    }

                    // cがアルファベットかどうかの判定
                    if c.is_ascii_alphabetic() {
                        // 現在の文字を再度取り扱うためにreconsumeをtrue
                        self.reconsume = true;
                        self.state = State::TagName;
                        // start_tag_tokenがtrueなのでStartTag状態にする
                        self.create_tag(true);
                        continue;
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    self.reconsume = true;
                    self.state = State::Data;
                }

                // 終了タグを取り扱うための状態。
                State::EndTagOpen => {
                    // 入力である文字列が最後に到達した場合
                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    if c.is_ascii_alphabetic() {
                        self.reconsume = true;
                        self.state = State::TagName;

                        // start_tag_tokenがfalseなのでEndTag状態にする
                        self.create_tag(false);
                        continue;
                    }
                }

                // タグの名前を扱うための状態
                State::TagName => {
                    if c == ' ' {
                        self.state = State::BeforeAttributeName;
                        continue;
                    }

                    if c == '/' {
                        self.state = State::SelfClosingStartTag;
                        continue;
                    }

                    if c == '>' {
                        self.state = State::Data;
                        return self.take_latest_token();
                    }

                    if c.is_ascii_uppercase() {
                        // 小文字をtagに入れる
                        self.append_tag_name(c.to_ascii_lowercase());
                        continue;
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    self.append_tag_name(c);
                }

                // タグ属性の名前を処理する前の状態
                State::BeforeAttributeName => {
                    if c == '/' || c == '>' || self.is_eof() {
                        self.reconsume = true;
                        self.state = State::AfterAttributeName;
                        continue;
                    }

                    self.reconsume = true;
                    self.state = State::AttributeName;
                    self.start_new_attribute();
                }

                // <p class="hello">こんにちは</p>
                // ここでいうclassが属性
                // タグ属性を扱うための状態
                State::AttributeName => {
                    if c == ' ' || c == '/' || c == '>' || self.is_eof() {
                        self.reconsume = true;
                        self.state = State::AfterAttributeName;
                        continue;
                    }

                    if c == '=' {
                        self.state = State::BeforeAttributeValue;
                        continue;
                    }

                    if c.is_ascii_uppercase() {
                        self.append_attribute(c.to_ascii_lowercase(), true);
                        continue;
                    }

                    self.append_attribute(c, true);
                }

                // タグ属性の名前を処理している状態
                State::AfterAttributeName => {
                    if c == ' ' {
                        // 空白文字は無視
                        continue;
                    }

                    if c == '/' {
                        self.state = State::SelfClosingStartTag;
                        continue;
                    }

                    if c == '=' {
                        self.state = State::BeforeAttributeValue;
                        continue;
                    }

                    if c == '>' {
                        self.state = State::Data;
                        return self.take_latest_token();
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    self.reconsume = true;
                    self.state = State::AttributeName;
                    self.start_new_attribute();
                }

                // タグの属性の値を処理する前の状態
                State::BeforeAttributeValue => {
                    if c == ' ' {
                        // 空白文字は無視
                        continue;
                    }

                    if c == '"' {
                        self.state = State::AttributeValueDoubleQuoted;
                        continue;
                    }

                    if c == '\'' {
                        self.state = State::AttributeValueSingleQuoted;
                        continue;
                    }

                    self.reconsume = true;
                    self.state = State::AttributeValueUnquoted;
                }

                // ダブルクオートで囲まれたタグの属性の値を処理する状態
                State::AttributeValueDoubleQuoted => {
                    // ダブルクオートで閉じた時
                    if c == '"' {
                        self.state = State::AfterAttributeValueQuoted;
                        continue;
                    }

                    // ファイル終了時
                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    // 属性に文字を追加
                    self.append_attribute(c, false);
                }

                // シングルクオートで囲まれたタグの属性の値を処理する状態
                State::AttributeValueSingleQuoted => {
                    // シングルクオートで閉じた時
                    if c == '\'' {
                        self.state = State::AfterAttributeValueQuoted;
                        continue;
                    }

                    // ファイル終了時
                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    // 属性に文字を追加
                    self.append_attribute(c, false);
                }

                // シングルクオートで囲まれたタグの属性の値を処理する状態
                State::AttributeValueUnquoted => {
                    // 次の文字が空白の時
                    if c == ' ' {
                        self.state = State::BeforeAttributeName;
                        continue;
                    }

                    // 属性の名前が終了時
                    if c == '>' {
                        self.state = State::Data;
                        // create_tagメソッドによって作られた最後のトークンを返す
                        return self.take_latest_token();
                    }

                    // ファイル終了時
                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    // 属性に文字を追加
                    self.append_attribute(c, false);
                }

                // 属性の値を処理した後の状態
                State::AfterAttributeValueQuoted => {
                    // 次の文字が空白の時
                    if c == ' ' {
                        self.state = State::BeforeAttributeName;
                        continue;
                    }

                    // 次の文字がスラッシュの時
                    if c == '/' {
                        self.state = State::SelfClosingStartTag;
                        continue;
                    }

                    // create_tagメソッドによって作られた最後のトークンを返す
                    if c == '>' {
                        self.state = State::Data;
                        return self.take_latest_token();
                    }

                    // ファイル終了時
                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    self.reconsume = true;
                    self.state = State::BeforeAttributeValue;
                }

                // 自己終了タグ(<br />など)を処理する状態
                State::SelfClosingStartTag => {
                    if c == '>' {
                        self.set_self_closing_flag();
                        self.state = State::Data;
                        return self.take_latest_token();
                    }

                    if self.is_eof() {
                        // invalid parse error.
                        return Some(HtmlToken::Eof);
                    }
                }

                // <script>タグの中に書かれているJavaScriptを処理する状態
                State::ScriptData => {
                    // 開始
                    if c == '<' {
                        self.state = State::ScriptDataLessThanSign;
                        continue;
                    }

                    // ファイル終了時
                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    // 文字トークンを返す
                    return Some(HtmlToken::Char(c));
                }

                // <script>開始タグの中で小なり記号(<)が出てきた時の状態
                // 次の文字がタグ終了(</script>)を示すのか、それとも単なる文字リテラルなのかを判断
                State::ScriptDataLessThanSign => {
                    if c == '/' {
                        // 一時的なバッファを空文字でリセットする
                        self.buf = String::new();
                        self.state = State::ScriptDataEndTagOpen;
                        continue;
                    }

                    self.reconsume = true;
                    self.state = State::ScriptData;
                    return Some(HtmlToken::Char('<'));
                }

                // JavaScriptの終了を表す</script>終了タグを処理する前の状態
                State::ScriptDataEndTagOpen => {
                    // 次の文字がアルファベットの時
                    if c.is_ascii_alphabetic() {
                        self.reconsume = true;
                        self.state = State::ScriptDataEndTagName;
                        // create_tagメソッドを呼んで終了タグを作成
                        self.create_tag(false);
                        continue;
                    }

                    self.reconsume = true;
                    self.state = State::ScriptData;
                    // 使用では、"<"と"/"の2つの文字トークンを返すとなっているが、
                    // 私たちの実装ではnextメソッドからは一つのトークンしか返せないため、
                    // "<"のトークンのみを返す
                    return Some(HtmlToken::Char('<'));
                }

                // JavaScriptの終了を表す</script>終了タグのタグ名部分(script)を解析している状態
                State::ScriptDataEndTagName => {
                    // create_tagメソッドによって作成したlatest_tokenを返す
                    if c == '>' {
                        self.state = State::Data;
                        return self.take_latest_token();
                    }

                    // 次の文字がアルファベットの時、append_tag_nameメソッドを呼んで文字をトークンに追加する
                    if c.is_ascii_alphabetic() {
                        self.buf.push(c);
                        self.append_tag_name(c.to_ascii_lowercase());
                        continue;
                    }

                    self.state = State::TemporaryBuffer;
                    self.buf = String::from("</");
                    self.buf.push(c);
                    continue;
                }

                // 一時的にデータを蓄えるための状態
                State::TemporaryBuffer => {
                    self.reconsume = true;

                    // 文字数カウント
                    if self.buf.chars().count() == 0 {
                        self.state = State::ScriptData;
                        continue;
                    }

                    // 最初の1文字を削除する
                    // nth(index)は指定したindexの値をOption型で返す
                    // 最初の1文字を取得
                    let c = self.buf.chars().nth(0).expect("self.buf have least 1 char");
                    self.buf.remove(0);
                    return Some(HtmlToken::Char(c));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alloc::string::ToString;
    use alloc::vec;

    // 空文字のテスト
    // 何もない文字列が入力だった場合のテスト
    #[test]
    fn test_empty() {
        let html = "".to_string();
        let mut tokenizer = HtmlTokenizer::new(html);

        // if self.pos >= self.input.len() {
        //     return None;
        // }

        assert!(tokenizer.next().is_none())
    }

    // 開始タグと終了タグのテスト
    // HTMLの文字列が<body>の開始タグと終了タグだった場合のテスト
    #[test]
    fn test_start_and_end_tag() {
        let html = "<body></body>".to_string();
        let mut tokenizer = HtmlTokenizer::new(html);
        let expected = [
            HtmlToken::StartTag {
                tag: "body".to_string(),
                self_closing: false,
                attributes: Vec::new(),
            },
            HtmlToken::EndTag {
                tag: "body".to_string(),
            },
        ];

        for e in expected {
            // iteratorで解析
            assert_eq!(Some(e), tokenizer.next());
        }
    }

    // 属性のテスト
    // HTMLの文字列が<p>の開始タグと終了タグだった場合
    // <p class=\"A\" id='B' foo=bar></p>
    #[test]
    fn test_attributes() {
        let html = "<p class=\"A\" id='B' foo=bar></p>".to_string();
        let mut tokenizer = HtmlTokenizer::new(html);

        // class="A"
        let mut attr1 = Attribute::new();

        // name
        attr1.add_char('c', true);
        attr1.add_char('l', true);
        attr1.add_char('a', true);
        attr1.add_char('s', true);
        attr1.add_char('s', true);

        // value
        attr1.add_char('A', false);

        // id='B'
        let mut attr2 = Attribute::new();

        // name
        attr2.add_char('i', true);
        attr2.add_char('d', true);

        // value
        attr2.add_char('B', false);

        // foo=bar
        let mut attr3 = Attribute::new();

        // name
        attr3.add_char('f', true);
        attr3.add_char('o', true);
        attr3.add_char('o', true);

        // value
        attr3.add_char('b', false);
        attr3.add_char('a', false);
        attr3.add_char('r', false);

        let expected = [
            HtmlToken::StartTag {
                tag: "p".to_string(),
                self_closing: false,
                attributes: vec![attr1, attr2, attr3],
            },
            HtmlToken::EndTag {
                tag: "p".to_string(),
            },
        ];

        for e in expected {
            assert_eq!(Some(e), tokenizer.next());
        }
    }

    // 空要素タグのテスト
    // コンテンツを何も持たないから要素のテスト
    // 開始タグのself_closing（自己終了タグ）がtrueであるかのチェック
    #[test]
    fn test_self_closing_tag() {
        let html = "<img />".to_string();
        let mut tokenizer = HtmlTokenizer::new(html);
        let expected = [HtmlToken::StartTag {
            tag: "img".to_string(),
            self_closing: true,
            attributes: Vec::new(),
        }];

        for e in expected {
            assert_eq!(Some(e), tokenizer.next());
        }
    }

    // スクリプトタグのテスト
    // <script>タグとそのコンテンツである擬似的なJavaScriptのコードのテスト
    // <script>タグて囲まれたコンテンツは文字トークン(HtmlToken::Char)になる
    #[test]
    fn test_script_tag() {
        let html = "<script>js code;</script>".to_string();
        let mut tokenizer = HtmlTokenizer::new(html);
        let expected = [
            HtmlToken::StartTag {
                tag: "script".to_string(),
                self_closing: false,
                attributes: Vec::new(),
            },
            HtmlToken::Char('j'),
            HtmlToken::Char('s'),
            HtmlToken::Char(' '),
            HtmlToken::Char('c'),
            HtmlToken::Char('o'),
            HtmlToken::Char('d'),
            HtmlToken::Char('e'),
            HtmlToken::Char(';'),
            HtmlToken::EndTag {
                tag: "script".to_string(),
            },
        ];

        for e in expected {
            assert_eq!(Some(e), tokenizer.next());
        }
    }
}
