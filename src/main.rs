// extern crate base64;
// extern crate dotenv;
// extern crate jsonrpc_client_core;
// extern crate jsonrpc_client_http;
// extern crate serde_json;
// extern crate url;
// extern crate reqwest;

use dotenv::dotenv;
use std::env;
use std::io;
pub use serde_json::json;
pub use serde_json::{Map, Number, Value};

use rudodoo::{SessionInfo, OdooClient, OdooApi};

fn main() -> io::Result<()> {
    dotenv().ok();

    let client = OdooClient::new();

    let mut api = OdooApi::new(client);

    let version = api.version_info().unwrap();
    println!("version: {:#?}", version);
    let res: SessionInfo = api.login("tec-528", "admin", "admin").unwrap();
    println!("login: res: {:#?}", res);
    return Ok(());
    println!("calling db list ...");
    let dblist: Value = api.db_list().unwrap();
    println!("db_list: {:#?}", dblist);
    println!("calling db dump ...");
    match env::var("DB_PASSWORD") {
        Ok(password) => {
            let res = api.db_dump(&password, "tec-528", "dump.zip");
            println!("res: {:?}", res);
        },
        Err(_) => {
            println!("master password not set");
        } 

    };
    // println!("db create ...");
    // let res = client.db_create("diabeloop", "test2", false, "fr_FR", "admin");
    // match res {
    //     Ok(val) => {
    //         println!("create: {:#?}", val);
    //     }
    //     Err(err) => {
    //         println!("error: {:#?}", err);
    //     }
    // };
    // println!("db drop ...");
    // let res = client.db_drop("diabeloop", "test2");
    // match res {
    //     Ok(val) => {
    //         println!("drop: {:#?}", val);
    //     }
    //     Err(err) => {
    //         println!("err: {:#?}", err);
    //     }
    // };
    println!("field get ...");
    let res = api.object_fields_get("tec-528", 1, "admin", "stock.label");
    match res {
        Ok(Value::Object(val)) => {
            for (key, _value) in val {
                println!("attr: {:#?}", key);
            }
        }
        Ok(_) => {
            println!("uuhh");
        }
        Err(why) => {
            println!("err: {:#?}", why);
        }
    };

    let res = api.object_search(
        "tec-528",
        1,
        "admin",
        "stock.label",
        json!([
            ("is_terminal", "=", true),
            ("id", ">=", 1000),
            ("id", "<=", 1100)
        ]),
    ).unwrap();
    // match res {
    //     Ok(Value::Array(val)) => {
    //         println!("res: {:#?}", val);
    //     }
    //     _ => {
    //         println!("uuhh?");
    //     }
    // }
    let res = api.object_read(
        "tec-528",
        1,
        "admin",
        "stock.label",
        res,                              // json!([1024, 1025, 1026])
        json!([
            "name",
            "product_tag_ids",
            "is_terminal",
            "location_id",
            "state"
        ]),
    );
    match res {
        Ok(Value::Array(val)) => {
            for item in val {
                println!("item: {:#?}", item);
            }
        }
        _ => {
            println!("uhh?");
        }
    }

    Ok(())
}
