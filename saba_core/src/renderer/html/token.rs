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

                State::TagName => todo!(),
                State::BeforeAttributeName => todo!(),
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
