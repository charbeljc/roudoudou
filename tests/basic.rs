mod common;
use roudoudou::{OdooApi, OdooRpc, RpcRequest};
use serde_json::json;
use log::{debug, info, warn};

#[test]
fn test_rpc() {
    common::setup();
    let rpc = OdooRpc::new();
    let query = rpc.encode_query("call", json!([1, 2, 3]));
    // assert_eq!(query, RpcRequest {
    //     id: 1,
    //     jsonrpc: "2.0",
    //     method: "call",
    //     params:  json!([1, 2, 3])
    // })
}
