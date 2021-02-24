mod common;
use roudoudou::{OdooRpc, OdooApi, OdooClient, OString};
use log::{trace, debug, info, warn, error};
use serde_json::json;

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
    let rpc = OdooRpc::new();
    let api = OdooApi::new(rpc);

    let dblist = api.db_list().unwrap();
    
    debug!("dblist: {:#?}", dblist);
    assert!(dblist.iter().any(|x| x == "test"));
}

#[test]
fn test_select_handsets() {
    common::setup();
    let mut cli = OdooClient::new();

    match cli.login("ota3", "admin", "admin") {
        Err(err) => {
            error!("could not login: {:#?}", err)
        }
        Ok(_) => {
            //debug!("session: {:#?}", session);
            let StockLabel = cli.get_model("stock.label").unwrap();
            let domain = json!([("name", "=", "A0014")]);
            debug!("search domain: {:?}", domain);
            match StockLabel.search_browse(domain) {
                Err(err) => {
                    error!("search error: {:#?}", err);
                }
                Ok(labels) => {
                    assert_eq!(labels.attr("name"), Some(&json!("A0014")));
                    assert_eq!(labels.attr("os_version"), Some(&json!("OPM7.DBLG.012")));
                    assert_eq!(labels.attr("app_version"), Some(&json!("1.3.1.9-dblg1-full-commercial")));
                    assert_eq!(labels.attr("updater_version"), Some(&json!("1.4.0.28")));
                    assert_eq!(labels.attr("supervisor_version"), Some(&json!("0.0.1")));
                    assert_eq!(labels.attr("pin_reset_version"), Some(&json!("0.0.1")));
                }
            }
        }
    };
    cli.logout().unwrap();
    info!("bye bye!!")
}

