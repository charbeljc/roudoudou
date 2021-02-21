// extern crate base64;
// extern crate dotenv;
// extern crate jsonrpc_client_core;
// extern crate jsonrpc_client_http;
// extern crate serde_json;
// extern crate url;
// extern crate reqwest;

use dotenv::dotenv;
pub use serde_json::json;
pub use serde_json::{Map, Number, Value};
use std::env;
use std::io;

use rudodoo::{OdooApi, OdooRpc, SessionInfo};

fn main() -> io::Result<()> {
    dotenv().ok();

    let rpc = OdooRpc::new();

    let mut api = OdooApi::new(rpc);

    let version = api.version_info().unwrap();
    println!("version: {:#?}", version);
    let res: SessionInfo = api.login("tec-528", "admin", "admin").unwrap();
    println!("login: res: {:#?}", res);
    println!("calling db list ...");
    let dblist = api.db_list().unwrap();
    println!("db_list: {:#?}", dblist);
    println!("field get ...");
    let res = api.object_fields_get("tec-528", 1, "admin", "res.users");
    match res {
        Ok(obj) => {
            println!("Object: {}", obj.name);
            for (attr, desc) in obj.get_searchable_fields() {
                println!("search: {} = {}", attr, desc.type_);
            }
            for (attr, desc) in obj.get_required_fields() {
                println!("required: {} = {}", attr, desc.type_);
                println!("{:#?}", desc);
            }
            for (attr, desc) in obj.get_relational_fields() {
                println!("relational: {} = {}", attr, desc.type_);
                println!("{:#?}", desc);

            }
        }
        Err(why) => {
            println!("err: {:#?}", why);
        }
    };

    let ids = match api.object_search(
        "tec-528",
        1,
        "admin",
        "stock.label",
        json!([
            ("is_terminal", "=", true),
            ("id", ">=", 1000),
            ("id", "<=", 1100)
        ]),
    ) {
        Ok(val) => val,
        Err(err) => {
            println!("search error: {}", err);
            vec![]
        }
    };
    let res = api.object_read(
        "tec-528",
        1,
        "admin",
        "stock.label",
        ids, // json!([1024, 1025, 1026])
        [
            "name",
            "product_tag_ids",
            "is_terminal",
            "location_id",
            "state",
        ]
        .to_vec(),
    );
    match res {
        Ok(Value::Array(val)) => {
            for item in val {
               // println!("item: {:?}", item);
            }
        }
        _ => {
            println!("uhh?");
        }
    }
    match env::var("DB_PASSWORD") {
        Ok(password) => {
            println!("calling db dump ...");
            let res = api.db_dump(&password, "tec-528", "dump.zip");
            println!("res: {:?}", res);
            println!("db drop ...");
            let res = api.db_drop("diabeloop", "test2");
            match res {
                Ok(val) => {
                    println!("drop: {:#?}", val);
                }
                Err(err) => {
                    println!("err: {:#?}", err);
                }
            };

            println!("db create ...");
            let res = api.db_create(&password, "test2", false, "fr_FR", "admin");
            match res {
                Ok(val) => {
                    println!("create: {:#?}", val);
                }
                Err(err) => {
                    println!("error: {:#?}", err);
                }
            };
            println!("db drop ...");
            let res = api.db_drop(&password, "test2");
            match res {
                Ok(val) => {
                    println!("drop: {:#?}", val);
                }
                Err(err) => {
                    println!("err: {:#?}", err);
                }
            };
        }
        Err(_) => {
            println!("master password not set (use env variable DB_PASSWORD)");
        }
    }


    match api.logout() {
        Ok(res) => println!("ok, logged out: {}", res),
        Err(err) => println!("ouch, could not logout: {}", err),
    }

    Ok(())
}
