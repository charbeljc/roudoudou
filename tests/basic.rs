mod common;
use log::debug;
use roudoudou::{DBService, OString, OdooApi, OdooClient, OdooRpc};

#[test]
fn test_version_info() {
    common::setup();
    let rpc = OdooRpc::new();
    let api = OdooApi::new(rpc);

    let version = api.version_info().unwrap();
    assert_eq!(version.protocol_version, 1);
    assert_eq!(version.server_serial, OString::Filled("9.0".to_owned()));
    assert_eq!(version.server_version, OString::Filled("9.0c".to_owned()));

    debug!("version: {:#?}", version);
}

#[test]
fn test_dblist() {
    common::setup();
    let cli = OdooClient::new();
    let db = DBService::new(&cli);

    let dblist = db.list().unwrap();

    debug!("dblist: {:#?}", dblist);
    assert!(dblist.iter().any(|x| x == "test"));
}

#[test]
fn test_login_logout() {
    common::setup();
    let rpc = OdooRpc::new();
    let api = OdooApi::new(rpc);

    let session = api.login("tec-528", "admin", "admin").unwrap();
    debug!("session: {:#?}", session);

    let status = api.logout();
    debug!("status: {:#?}", status)
}
