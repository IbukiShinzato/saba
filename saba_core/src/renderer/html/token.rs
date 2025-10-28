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

                State::AttributeName => todo!(),
                State::AfterAttributeName => todo!(),
                State::BeforeAttributeValue => todo!(),
                State::AttributeValueDoubleQuoted => todo!(),
                State::AttributeValueSingleQuoted => todo!(),
                State::AttributeValueUnquoted => todo!(),
                State::AfterAttributeValueQuoted => todo!(),
                State::SelfClosingStartTag => todo!(),
                State::ScriptData => todo!(),
                State::ScriptDataLessThanSign => todo!(),
                State::ScriptDataEndTagOpen => todo!(),
                State::ScriptDataEndTagName => todo!(),
                State::TemporaryBuffer => todo!(),
            }
        }
    }
}
