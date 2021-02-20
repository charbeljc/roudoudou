extern crate base64;
extern crate dotenv;
extern crate jsonrpc_client_core;
extern crate jsonrpc_client_http;
extern crate serde_json;
extern crate url;
extern crate reqwest;

use dotenv::dotenv;
use std::env;
// use std::fs::File;
use std::io;
// use std::io::prelude::*;
// use std::io::BufWriter;
// use std::io::Cursor;

// use serde::{Serialize,Deserialize};
pub use serde_json::json;
pub use serde_json::{Map, Number, Value};

use jsonrpc_client_core::{call_method, Error};
use jsonrpc_client_http::HttpHandle;
use jsonrpc_client_http::HttpTransport;

use url::Url;
use rudodoo::{odoo_url_from_env, SessionInfo, OdooClient, OdooApi};

fn main() -> io::Result<()> {
    dotenv().ok();

    let client = OdooClient::new();

    let mut api = OdooApi::new(client);
    let res: SessionInfo = api.login("tec-528", "admin", "admin").unwrap();
    println!("login: res: {:#?}", res);
    println!("calling db list ...");
    let dblist: Value = api.db_list().unwrap();
    println!("db_list: {:#?}", dblist);
    // println!("calling db dump ...");
    // let res = client.db_dump("diabeloop", "tec-528", "zip");
    // match res {
    //     Ok(Value::String(val)) => {
    //         // println!("decoding data:\n{:#?}", &val[0..1000]);
    //         let f = File::create("dump.zip")?;
    //         let mut writer = BufWriter::new(f);
    //         let wrapped_reader = Cursor::new(val);
    //         println!("save file ...");
    //         for line in wrapped_reader.lines() {
    //             //let data = base64::decode(line.as_bytes()).unwrap();
    //             match line {
    //                 Ok(val) => {
    //                     let data = base64::decode(val).unwrap();
    //                     writer.write(&data)?;
    //                 }
    //                 Err(err) => {
    //                     println!("err: {:#?}", err);
    //                 }
    //             };
    //         }
    //         //let data = base64::decode(val).unwrap();
    //         println!("done.");
    //     }
    //     Ok(_) => {
    //         println!("Huu");
    //     }
    //     Err(err) => {
    //         println!("error: {:#?}", err);
    //     }
    // }
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
