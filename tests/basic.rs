mod common;
use roudoudou::{OdooApi, OdooRpc};
use serde_json::json;
use log::{debug, info, warn};

#[test]
fn test_version_info() {
    common::setup();
    let rpc = OdooRpc::new();
    let api = OdooApi::new(rpc);

    let version = api.version_info().unwrap();
    assert_eq!(version.protocol_version, 1);
    assert_eq!(version.server_serial, "9.0");
    assert_eq!(version.server_version, "9.0c");

    println!("version: {:#?}", version);
}

#[test]
fn test_dblist() {
    common::setup();
    let rpc = OdooRpc::new();
    let api = OdooApi::new(rpc);

    let dblist = api.db_list().unwrap();
    
    println!("dblist: {:#?}", dblist);
    assert!(dblist.iter().any(|x| x == "test"));
}

#[test]
fn test_login_logout() {
    common::setup();
    let rpc = OdooRpc::new();
    let api = OdooApi::new(rpc);

    let session = api.login("tec-528", "admin", "admin").unwrap();
    println!("session: {:#?}", session);

    let status = api.logout();
    println!("status: {:#?}", status)
}

