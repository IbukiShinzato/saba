use alloc::string::{String, ToString};
use alloc::vec::Vec;

// http://<host>:<port>/<path>?<searchpart>

#[derive(Debug, Clone, PartialEq)]
pub struct Url {
    url: String,
    host: String,
    port: String,
    path: String,
    searchpart: String,
}

impl Url {
    // コンストラクタ　インスタンスを作成する関数
    pub fn new(url: String) -> Self {
        Self {
            url,
            host: "".to_string(),
            port: "".to_string(),
            path: "".to_string(),
            searchpart: "".to_string(),
        }
    }

    // 以降はメソッド

    // Resultの右側がparse成功時の値、左側が失敗時のエラーメッセージ
    // &mut selfは可変参照で、インスタンス自体を変更することができる
    pub fn parse(&mut self) -> Result<Self, String> {
        if !self.is_http() {
            return Err("Only HTTP scheme is supported.".to_string());
        }

        self.host = self.extract_host();
        self.port = self.extract_port();
        self.path = self.extract_path();
        self.searchpart = self.extract_searchpart();

        Ok(self.clone())
    }

    // httpスキーマを省略していないかの判定
    fn is_http(&mut self) -> bool {
        self.url.contains("http://")
    }

    // host部分を抽出するメソッド
    fn extract_host(&self) -> String {
        let url_parts: Vec<&str> = self
            .url
            .trim_start_matches("http://")
            .splitn(2, '/')
            .collect();

        // port番号が指定されてたら、:の前までをhostとして抽出
        if let Some(index) = url_parts[0].find(':') {
            url_parts[0][..index].to_string()
        } else {
            url_parts[0].to_string()
        }
    }

    // port番号を抽出するメソッド
    fn extract_port(&self) -> String {
        let url_parts: Vec<&str> = self
            .url
            .trim_start_matches("http://")
            .splitn(2, '/')
            .collect();

        // port番号が指定されてたら、:の後ろまでをportとして抽出
        if let Some(index) = url_parts[0].find(':') {
            url_parts[0][index + 1..].to_string()
        } else {
            // httpのデフォルトポートは80番
            "80".to_string()
        }
    }

    // path部分を抽出するメソッド
    fn extract_path(&self) -> String {
        let url_parts: Vec<&str> = self
            .url
            .trim_start_matches("http://")
            .splitn(2, '/')
            .collect();

        // pathが存在しない場合は空の文字列を返す
        if url_parts.len() < 2 {
            return "".to_string();
        }

        // 左側がpath、右側がsearchpart
        let path_and_searchpart: Vec<&str> = url_parts[1].splitn(2, '?').collect();
        path_and_searchpart[0].to_string()
    }

    // searchpart部分を抽出するメソッド
    fn extract_searchpart(&self) -> String {
        let url_parts: Vec<&str> = self
            .url
            .trim_start_matches("http://")
            .splitn(2, '/')
            .collect();

        // pathが存在しない場合は空の文字列を返す
        if url_parts.len() < 2 {
            return "".to_string();
        }

        // 左側がpath、右側がsearchpart
        let path_and_searchpart: Vec<&str> = url_parts[1].splitn(2, '?').collect();

        // queryパラメータが存在しない場合はからの文字列を返す
        if path_and_searchpart.len() < 2 {
            "".to_string()
        } else {
            path_and_searchpart[1].to_string()
        }
    }

    // rustの構造体はデフォルトでプライベートなので、外部からアクセスできるようにゲッターを定義する
    // 以降はゲッターメソッド

    pub fn host(&self) -> String {
        self.host.clone()
    }

    pub fn port(&self) -> String {
        self.port.clone()
    }

    pub fn path(&self) -> String {
        self.path.clone()
    }

    pub fn searchpart(&self) -> String {
        self.searchpart.clone()
    }
}

// ユニットテスト（実装したコードの一部が正しい挙動をしているかの確認）による動作確認
#[cfg(test)]
mod tests {
    use super::*;

    // 成功ケース

    // host名が記入されている
    #[test]
    fn test_url_host() {
        let url = "http://example.com".to_string();
        let expected = Ok(Url {
            url: url.clone(),
            host: "example.com".to_string(),
            port: "80".to_string(),
            path: "".to_string(),
            searchpart: "".to_string(),
        });

        assert_eq!(expected, Url::new(url).parse());
    }

    // host名とport番号が記入されている
    #[test]
    fn test_url_host_port() {
        let url = "http://exapmle.com:8888".to_string();
        let expected = Ok(Url {
            url: url.clone(),
            host: "exapmle.com".to_string(),
            port: "8888".to_string(),
            path: "".to_string(),
            searchpart: "".to_string(),
        });

        assert_eq!(expected, Url::new(url).parse());
    }

    // host名とport番号、pathが記入されている
    #[test]
    fn test_url_host_port_path() {
        let url = "http://example.com:8888/index.html".to_string();
        let expected = Ok(Url {
            url: url.clone(),
            host: "example.com".to_string(),
            port: "8888".to_string(),
            path: "index.html".to_string(),
            searchpart: "".to_string(),
        });

        assert_eq!(expected, Url::new(url).parse());
    }

    // host名とpathが記入されている
    #[test]
    fn test_url_host_path() {
        let url = "http://example.com/index.html".to_string();
        let expected = Ok(Url {
            url: url.clone(),
            host: "example.com".to_string(),
            port: "80".to_string(),
            path: "index.html".to_string(),
            searchpart: "".to_string(),
        });

        assert_eq!(expected, Url::new(url).parse());
    }

    // host名、port番号、path、searchpartが記入されている
    #[test]
    fn test_url_host_port_path_searchpart() {
        let url = "http://example.com:8888/index.html?a=123&b=456".to_string();
        let expected = Ok(Url {
            url: url.clone(),
            host: "example.com".to_string(),
            port: "8888".to_string(),
            path: "index.html".to_string(),
            searchpart: "a=123&b=456".to_string(),
        });

        assert_eq!(expected, Url::new(url).parse());
    }

    // 失敗ケース

    // scheme(HTTP)が記入されていない
    #[test]
    fn test_no_scheme() {
        let url = "example.com".to_string();
        let expected = Err("Only HTTP scheme is supported.".to_string());

        assert_eq!(expected, Url::new(url).parse());
    }

    // HTTP以外のschemeを記入している
    #[test]
    fn test_unsupported_scheme() {
        let url = "https://example.com:8888/index.html?a=123&b=456".to_string();
        let expected = Err("Only HTTP scheme is supported.".to_string());

        assert_eq!(expected, Url::new(url).parse());
    }
}
