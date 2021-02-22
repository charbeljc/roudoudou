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

use roudoudou::{OdooApi, OdooRpc, SessionInfo, Error};

fn main() -> Result<(), Error> {
    dotenv().ok();

    let rpc = OdooRpc::new();
    let api = OdooApi::new(rpc);

    let version = match api.version_info() {
    
        Ok(version) => version,
        Err(err) => {
            eprintln!("Could not get version info from server: {:#?}", err);
            return Err(err);
        }
    };
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
            // obj.show();
            println!("{}", obj.name);
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
            ("id", "<=", 1010)
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
            "name".to_owned(),
            "product_id".to_owned(),
            "product_tag_ids".to_owned(),
            "is_terminal".to_owned(),
            "location_id".to_owned(),
            "state".to_owned(),
        ]
        .to_vec(),
    );
    match res {
        Ok(Value::Array(val)) => {
            for item in val {
                println!("item: {:?}", item);
            }
        }
        _ => {
            println!("uhh?");
        }
    }

    let model = api.get_model("res.users").unwrap();
    let users = match model.search(json!([("id", ">", 0), ("id", "<", 10)])) {
        Ok(a) => model.browse(a),
        _ => unreachable!(),
    }
    .unwrap();
    println!("XXX: users: {:?}", users);

    let model = api.get_model("ir.module.module").unwrap();
    let ids = model.search(json!([("name", "=", "mrp_fixes")])).unwrap();
    let module = model.browse(ids).unwrap();
    println!("module: {:?}", module);
    //println!("module data: {:#?}", module.data);
    println!("module name: {:?}", module.attr("name"));

    let model = api.get_model("stock.label").unwrap();
    let ids = model
        .search(json!([("name", "=", "352719110488433")]))
        .unwrap();
    let term = model.browse(ids).unwrap();
    println!("terminal: {:?}", term);
    println!(
        "name: {:?} is_terminal: {:?} is_diabeloop: {:?}",
        term.attr("name"),
        term.attr("is_terminal"),
        term.attr("is_diabeloop")
    );
    let update = json!({"os": {"version": "0.1.0", "apk": "foobar.apk"}});

    let card = model.browse(
        model
            .search(json!([("name", "=", "89351060000893708683")]))
            .unwrap(),
    ).unwrap();

    let args = Some(json!(update.to_string()));
    let res = card.call("servicing_ota_update", args, None);
    println!("servicing_ota result: {:?}", res);

    let res = model.call("get_public_methods", None, None);
    println!("get_public_methods: {:#?}", res);

    let res = term.call(
        "foobar",
        Some(json!((1, 2, 3))),
        Some(json!({"say": "Viva l'Algérie !"})),
    );
    println!("foobar result: {:?}", res);

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
