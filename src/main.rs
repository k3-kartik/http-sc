use k3_wasm_macros::http_handler;
use k3_wasm_sdk::{
    data_sc,
    http::{self, Request, Response},
};

#[http_handler]
pub fn get(_req: Request<Vec<u8>>) -> Response<Vec<u8>> {
    let address = std::env::var("SC_ADDRESS").expect("SC_ADDRESS variable must be set");
    dbg!(&address);
    let data = data_sc::query(&address);
    dbg!(&data);
    Response::builder()
        .status(200)
        .body(data.as_bytes().to_vec())
        .unwrap()
}

#[derive(Debug, Clone)]
enum Selector {
    Key(String),
    Index(usize),
}

fn parse_selector(selector: &str) -> Vec<Selector> {
    let mut parts = vec![];

    let bytes = selector.as_bytes();
    let mut buffer = vec![];
    let mut buffer_is_key = true;
    let mut offset = 0usize;
    while offset < bytes.len() {
        match bytes[offset] {
            b'"' => {
                buffer_is_key = true;
                offset += 1;
                while bytes[offset] != b'"' {
                    buffer.push(bytes[offset]);
                    offset += 1;
                }
                offset += 1;
            }
            b'.' => {
                parts.push(if buffer_is_key {
                    Selector::Key(String::from_utf8(buffer.clone()).unwrap())
                } else {
                    Selector::Index(String::from_utf8(buffer.clone()).unwrap().parse().unwrap())
                });
                offset += 1;
                buffer.clear();
            }
            _ if bytes[offset].is_ascii_digit() => {
                buffer_is_key = false;
                while bytes[offset].is_ascii_digit() {
                    buffer.push(bytes[offset]);
                    offset += 1;
                }
            }
            _ => {
                panic!(
                    "Selector parser reached invalid character: '{}'",
                    bytes[offset] as char
                )
            }
        }
    }

    if !buffer.is_empty() {
        parts.push(if buffer_is_key {
            Selector::Key(String::from_utf8(buffer.clone()).unwrap())
        } else {
            Selector::Index(String::from_utf8(buffer.clone()).unwrap().parse().unwrap())
        });
    }

    parts
}

fn execute_selector(selector: &[Selector], json: serde_json::Value) -> Option<serde_json::Value> {
    let mut current = json;
    for selector in selector {
        match selector {
            Selector::Key(key) => {
                current = current.as_object()?.get(key)?.clone();
            }
            Selector::Index(idx) => {
                current = current.as_array()?.get(*idx)?.clone();
            }
        }
    }
    Some(current)
}

#[http_handler]
pub fn post(_req: Request<Vec<u8>>) -> Response<Vec<u8>> {
    let address = std::env::var("SC_ADDRESS").expect("SC_ADDRESS variable must be set");
    let url = std::env::var("URL").expect("URL variable must be set");
    let json_selector = std::env::var("JSON_SELECTOR").expect("JSON_SELECTOR variable must be set");
    dbg!(&url, &json_selector);
    let json_selector = parse_selector(&json_selector);
    let res = http::get(&url);
    if let Some(res) = res {
        let json = serde_json::from_slice::<serde_json::Value>(&res).unwrap();
        let new_data = execute_selector(&json_selector, json);
        if let Some(new_data) = new_data {
            let tx_hash = data_sc::update(
                &address,
                String::from_utf8(new_data.to_string().into()).unwrap(),
            );
            Response::builder()
                .status(200)
                .body(tx_hash.as_bytes().to_vec())
                .unwrap()
        } else {
            Response::builder()
                .status(500)
                .body(
                    "JSON selector was not valid against returned JSON"
                        .as_bytes()
                        .to_vec(),
                )
                .unwrap()
        }
    } else {
        Response::builder()
            .status(500)
            .body("Fetch to HTTP API failed".as_bytes().to_vec())
            .unwrap()
    }
}

k3_wasm_macros::init!();
