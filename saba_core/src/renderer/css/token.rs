use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq)]
pub enum CssToken {
    // ハッシュトークン
    HashToken(String),
    // 区切り　',' '.'　など
    Delim(char),
    // 数字トークン
    Number(f64),
    // コロン　':'
    Colon,
    // セミコロン　';'
    SemiColon,
    // 丸括弧（開き）　'('
    OpenParenthesis,
    // 丸括弧（閉じ）　')'
    CloseParenthesis,
    // 波括弧（開き）　'{'
    OpenCurly,
    // 波括弧（閉じ）　'}'
    CloseCurly,
    // 識別子トークン
    Ident(String),
    // 文字列トークン
    StringToken(String),
    // アットキーワードトークン
    AtKeyword(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CssTokenizer {
    pos: usize,
    input: Vec<char>,
}

impl CssTokenizer {
    // コンストラクタ
    pub fn new(css: String) -> Self {
        Self {
            pos: 0,
            input: css.chars().collect(),
        }
    }

    // もう一度"や'が出てくるまで、文字列を消費
    fn consume_string_token(&mut self) -> String {
        let mut s = String::new();

        loop {
            if self.pos >= self.input.len() {
                return s;
            }

            self.pos += 1;
            let c = self.input[self.pos];
            match c {
                '"' | '\'' => break,
                _ => {
                    s.push(c);
                }
            }
        }

        s
    }

    // 数字の文字列をf64で返す
    fn consume_numeric_token(&mut self) -> f64 {
        let mut num = 0f64;
        let mut floating = false;
        let mut floating_digit = 1f64;

        loop {
            if self.pos >= self.input.len() {
                return num;
            }

            let c = self.input[self.pos];
            match c {
                '0'..='9' => {
                    // 小数なら
                    if floating {
                        floating_digit *= 1f64 / 10f64; // 0.1
                        num += (c.to_digit(10).unwrap() as f64) * floating_digit
                    } else {
                        // 末尾の桁を一つ左にずらす　xx1 => xx1x
                        num = num * 10.0 + (c.to_digit(10).unwrap() as f64);
                    }
                    self.pos += 1;
                }

                '.' => {
                    floating = true;
                    self.pos += 1;
                }

                _ => {
                    break;
                }
            }
        }
        num
    }

    // 文字、数字、ハイフン(-)、アンダースコア(_)が出続けている間は識別子として扱う
    fn consume_ident_token(&mut self) -> String {
        let mut s = String::new();
        s.push(self.input[self.pos]);

        loop {
            self.pos += 1;
            let c = self.input[self.pos];
            match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => {
                    s.push(c);
                }
                _ => {
                    break;
                }
            }
        }
        s
    }
}

impl Iterator for CssTokenizer {
    type Item = CssToken;

    // 入力のCSS文字列の1文字ずつ見ていく
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.pos >= self.input.len() {
                return None;
            }

            // 1文字とる
            let c = self.input[self.pos];

            let token = match c {
                '(' => CssToken::OpenParenthesis,
                ')' => CssToken::CloseParenthesis,
                ',' => CssToken::Delim(','),
                '.' => CssToken::Delim('.'),
                ':' => CssToken::Colon,
                ';' => CssToken::SemiColon,
                '{' => CssToken::OpenCurly,
                '}' => CssToken::CloseCurly,
                ' ' | '\n' => {
                    // 今は一時的に空文字と改行をスキップ
                    self.pos += 1;
                    continue;
                }
                '"' | '\'' => {
                    // 次のクォーテーションが来るまで入力を文字として解釈
                    let value = self.consume_string_token();
                    CssToken::StringToken(value)
                }
                '0'..='9' => {
                    let t = CssToken::Number(self.consume_numeric_token());
                    // 数字の次の文字まで進んでいるので1つ戻す
                    self.pos -= 1;
                    t
                }
                '#' => {
                    let value = self.consume_ident_token();
                    self.pos -= 1;
                    CssToken::HashToken(value)
                }
                '-' => {
                    // 負の数は取り扱わないので、識別子の一つとして扱う
                    let t = CssToken::Ident(self.consume_ident_token());
                    self.pos -= 1;
                    t
                }
                '@' => {
                    // 次の3文字が識別子として有効な文字の場合、at-keyword-tokenトークンを作成して返す
                    // @media, @import, @font-faceなど
                    if self.pos + 3 < self.input.len()
                        && self.input[self.pos + 1].is_ascii_alphabetic()
                        && self.input[self.pos + 2].is_ascii_alphabetic()
                        && self.input[self.pos + 3].is_ascii_alphabetic()
                    {
                        self.pos += 1;
                        let t = CssToken::AtKeyword(self.consume_ident_token());
                        self.pos -= 1;
                        t
                    } else {
                        CssToken::Delim('@')
                    }
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let t = CssToken::Ident(self.consume_ident_token());
                    self.pos -= 1;
                    t
                }
                _ => {
                    unimplemented!("char {} is not implemented yet", c);
                }
            };

            // 次の文字へ移動
            self.pos += 1;
            return Some(token);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    // 空文字のテスト
    #[test]
    fn test_empty() {
        let style = "".to_string();
        let mut t = CssTokenizer::new(style);
        assert!(t.next().is_none());
    }

    // １つのルールのテスト
    // 今回はpタグ
    #[test]
    fn test_one_rule() {
        let style = "p { color: red; }".to_string();
        let mut t = CssTokenizer::new(style);
        let expected = [
            CssToken::Ident("p".to_string()),
            CssToken::OpenCurly,
            CssToken::Ident("color".to_string()),
            CssToken::Colon,
            CssToken::Ident("red".to_string()),
            CssToken::SemiColon,
            CssToken::CloseCurly,
        ];

        for e in expected {
            assert_eq!(Some(e), t.next());
        }
    }

    // IDセレクタを持つルールのテスト
    #[test]
    fn test_id_selector() {
        let style = "#id { color: red; }".to_string();
        let mut t = CssTokenizer::new(style);
        let expected = [
            CssToken::HashToken("#id".to_string()),
            CssToken::OpenCurly,
            CssToken::Ident("color".to_string()),
            CssToken::Colon,
            CssToken::Ident("red".to_string()),
            CssToken::SemiColon,
            CssToken::CloseCurly,
        ];

        for e in expected {
            assert_eq!(Some(e), t.next());
        }
    }

    // クラスセレクタを持つルールのテスト
    #[test]
    fn test_class_selector() {
        let style = ".class { color: red; }".to_string();
        let mut t = CssTokenizer::new(style);
        let expected = [
            CssToken::Delim('.'),
            CssToken::Ident("class".to_string()),
            CssToken::OpenCurly,
            CssToken::Ident("color".to_string()),
            CssToken::Colon,
            CssToken::Ident("red".to_string()),
            CssToken::SemiColon,
            CssToken::CloseCurly,
        ];

        for e in expected {
            assert_eq!(Some(e), t.next());
        }
    }

    // 複数のルールのテスト
    #[test]
    fn test_multiple_rules() {
        let style = "p { content: \"Hey\"; } h1 { font-size: 40; color: blue; }".to_string();
        let mut t = CssTokenizer::new(style);
        let expected = [
            CssToken::Ident("p".to_string()),
            CssToken::OpenCurly,
            CssToken::Ident("content".to_string()),
            CssToken::Colon,
            CssToken::StringToken("Hey".to_string()),
            CssToken::SemiColon,
            CssToken::CloseCurly,
            CssToken::Ident("h1".to_string()),
            CssToken::OpenCurly,
            CssToken::Ident("font-size".to_string()),
            CssToken::Colon,
            CssToken::Number(40.0),
            CssToken::SemiColon,
            CssToken::Ident("color".to_string()),
            CssToken::Colon,
            CssToken::Ident("blue".to_string()),
            CssToken::SemiColon,
            CssToken::CloseCurly,
        ];

        for e in expected {
            assert_eq!(Some(e), t.next());
        }

        assert!(t.next().is_none());
    }
}
