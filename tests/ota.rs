mod common;
use log::{debug, error, info, trace, warn};
use roudoudou::{OString, OdooApi, OdooClient, OdooRpc};
use serde_json::json;

use pretty_assertions::{assert_eq, assert_ne};

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
    assert_eq!(dblist, vec![
        "charbel",
         "ota",
         "ota2",
         "prod",
         "prod2",
         "prod_snapshot_2020_12_29",
         "produpgrade",
         "suppliers",
         "tec-528",
         "test",
         "traca",
         "usgaap",
    ]);
}

#[test]
fn ota_update_a0014_os_version() {
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
                    assert_eq!(
                        labels.attr("app_version"),
                        Some(&json!("1.3.1.9-dblg1-full-commercial"))
                    );
                    assert_eq!(labels.attr("updater_version"), Some(&json!("1.4.0.28")));
                    assert_eq!(labels.attr("supervisor_version"), Some(&json!("0.0.1")));
                    assert_eq!(labels.attr("pin_reset_version"), Some(&json!("0.0.1")));

                    match labels.call("servicing_ota_query", None, None) {
                        Ok(value) => {
                            info!("success calling method: {:#?}", value);
                        }
                        Err(err) => {
                            error!("calling method: {}", err);
                        }
                    }
                    let empty: Option<String> = None;
                    match labels.call(
                        "servicing_ota_update",
                        Some(json!(json!({
                                "os": {
                                    "version": "0.0.1",
                                    "apk:": empty
                                }
                        })
                        .to_string())),
                        None,
                    ) {
                        Ok(value) => {
                            info!("success calling method: {:#?}", value);
                        }
                        Err(err) => {
                            error!("calling method: {}", err);
                        }
                    }
                    match labels.call("servicing_ota_query", None, None) {
                        Ok(value) => {
                            info!("success calling method: {:#?}", value);
                        }
                        Err(err) => {
                            error!("calling method: {}", err);
                        }
                    }
                    match StockLabel.browse(&labels.ids) {
                        Err(err) => {
                            error!("could not browse records: {}", err)
                        }
                        Ok(updated) => {
                            assert_eq!(updated.attr("name"), Some(&json!("A0014")));
                            assert_eq!(updated.attr("os_version"), Some(&json!("0.0.1")));
                            assert_eq!(
                                updated.attr("app_version"),
                                Some(&json!("1.3.1.9-dblg1-full-commercial"))
                            );
                            assert_eq!(updated.attr("updater_version"), Some(&json!("1.4.0.28")));
                            assert_eq!(updated.attr("supervisor_version"), Some(&json!("0.0.1")));
                            assert_eq!(updated.attr("pin_reset_version"), Some(&json!("0.0.1")));
                        }
                    }

                    match labels.call("servicing_ota_query", None, None) {
                        Ok(value) => {
                            info!("success calling method: {:#?}", value);
                        }
                        Err(err) => {
                            error!("calling method: {}", err);
                        }
                    }
                }
            }
        }
    };
    cli.logout().unwrap();
}
