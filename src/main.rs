#![no_std]
#![no_main]

extern crate alloc;

use crate::alloc::string::ToString;
use net_wasabi::http::HttpClient;
use noli::prelude::*;

fn main() -> u64 {
    let client = HttpClient::new();
    // localhostと接続
    // localhostに直接アクセスすることができないので、host.testでlocalhostにフォワーディングする
    match client.get("host.test".to_string(), 8000, "/test.html".to_string()) {
        Ok(res) => {
            print!("response:\n{:#?}", res);
        }
        Err(e) => {
            print!("error:\n{:#?}", e);
        }
    }

    0
}

entry_point!(main);
