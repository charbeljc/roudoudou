mod common;
use log::{debug, error, info};
use roudoudou::{DBService, OString, OdooApi, OdooClient, OdooRpc};
use serde_json::json;

use pretty_assertions::assert_eq;

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
    assert_eq!(
        dblist,
        vec![
            "charbel",
            "ota",
            "ota2",
            "ota8",
            "ota_old",
            "prod",
            "prod2",
            "prod_snapshot_2020_12_29",
            "produpgrade",
            "suppliers",
            "tec-528",
            "test",
            "traca",
            "usgaap",
        ]
    );
}

macro_rules! assert_attr_eq {
    ($obj:expr, $name:ident, None) => {
        assert_eq!(($obj).attr(stringify!($name)), None)
    };
    ($obj:expr, $name:ident, $expr:tt) => {
        assert_eq!(($obj).attr(stringify!($name)), Some(&json!($expr)))
    };
}
#[test]
fn ota_update_kit_1000_os_version() {
    common::setup();
    let mut cli = OdooClient::new();

    match cli.login("ota8", "admin", "admin") {
        Err(err) => {
            error!("could not login: {:#?}", err)
        }
        Ok(_) => {
            //debug!("session: {:#?}", session);
            let stock_label = cli.get_model("stock.label").unwrap();
            let domain = json!([("name", "=", "1000")]);
            debug!("search domain: {:?}", domain);
            match stock_label.search_browse(domain) {
                Err(err) => {
                    error!("search error: {:#?}", err);
                }
                Ok(labels) => {
                    assert_attr_eq!(labels, name, "1000");
                    // assert_attr_eq!(labels, os_version, "OPM7.DBLG.012");
                    // assert_attr_eq!(labels, app_version, "1.3.1.9-dblg1-full-commercial");
                    // assert_attr_eq!(labels, updater_version, "1.4.0.28");
                    // assert_attr_eq!(labels, supervisor_version, "0.0.1");
                    // assert_attr_eq!(labels, pin_reset_version, "0.0.1");

                    // assert_attr_eq!(labels, system, "G1");
                    // assert_attr_eq!(labels, pump_type, "insight");
                    // assert_attr_eq!(labels, transmitter_type, "dexcom G6");
                    // assert_attr_eq!(labels, measure, "mg/dl");
                    // assert_attr_eq!(labels, market, "commercial");
                    // assert_attr_eq!(labels, country, "DE");
                    // assert_attr_eq!(labels, lang, "de");

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
                    match stock_label.browse(&labels.ids) {
                        Err(err) => {
                            error!("could not browse records: {}", err)
                        }
                        Ok(updated) => {
                            assert_attr_eq!(updated, name, "1000");
                            // assert_attr_eq!(updated, os_version, "0.0.1");
                            // assert_attr_eq!(updated, app_version, "1.3.1.9-dblg1-full-commercial");
                            // assert_attr_eq!(updated, updater_version, "1.4.0.28");
                            // assert_attr_eq!(updated, supervisor_version, "0.0.1");
                            // assert_attr_eq!(updated, pin_reset_version, "0.0.1");
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
    // cli.logout().unwrap(); // FIXME
}

#[test]
fn ota_update_demo0050_os_version() {
    common::setup();
    let mut cli = OdooClient::new();

    match cli.login("ota8", "admin", "admin") {
        Err(err) => {
            error!("could not login: {:#?}", err)
        }
        Ok(_) => {
            //debug!("session: {:#?}", session);
            let stock_label = cli.get_model("stock.label").unwrap();
            let domain = json!([("name", "=", "DEMO0050")]);
            debug!("search domain: {:?}", domain);
            match stock_label.search_browse(domain) {
                Err(err) => {
                    error!("search error: {:#?}", err);
                }
                Ok(labels) => {
                    assert_attr_eq!(labels, name, "DEMO0050");
                    // assert_attr_eq!(labels, os_version, "DBLG1.PROD.RELEASE.023");
                    // assert_attr_eq!(
                    //     labels,
                    //     app_version,
                    //     "1.8.1.28-dblg1-full-commercial-insight-release"
                    // );
                    // assert_attr_eq!(labels, updater_version, "1.4.0.28");
                    // assert_attr_eq!(labels, supervisor_version, "1.1.0.28");
                    // assert_attr_eq!(labels, pin_reset_version, "1.1.0.28");

                    // assert_attr_eq!(labels, system, "G1");
                    // assert_attr_eq!(labels, pump_type, "insight");
                    // assert_attr_eq!(labels, transmitter_type, "dexcom G6");
                    // assert_attr_eq!(labels, measure, "mg/dl");
                    // assert_attr_eq!(labels, market, "commercial");
                    // assert_attr_eq!(labels, country, "IT");
                    // assert_attr_eq!(labels, lang, "it");

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
                    match stock_label.browse(&labels.ids) {
                        Err(err) => {
                            error!("could not browse records: {}", err)
                        }
                        Ok(updated) => {
                            assert_attr_eq!(updated, name, "DEMO0050");
                            // assert_attr_eq!(updated, os_version, "0.0.1");
                            // assert_attr_eq!(
                            //     updated,
                            //     app_version,
                            //     "1.8.1.28-dblg1-full-commercial-insight-release"
                            // );
                            // assert_attr_eq!(updated, updater_version, "1.4.0.28");
                            // assert_attr_eq!(updated, supervisor_version, "1.1.0.28");
                            // assert_attr_eq!(updated, pin_reset_version, "1.1.0.28");
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
    // cli.logout().unwrap(); // FIXME
}
