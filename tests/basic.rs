mod common;
use roudoudou::{OdooApi, OdooRpc};
use serde_json::json;
use log::{debug, info, warn};

#[test]
fn test_rpc() {
    common::setup();
    eprintln!("TEST");
    let rpc = OdooRpc::new();
    eprintln!("BASE_URL: {:#?}", rpc);
    let query = rpc.encode_query("call", json!([1, 2, 3]));
    eprintln!("QUERY: {:#?}", query)
}