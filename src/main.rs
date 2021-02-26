// extern crate base64;
// extern crate dotenv;
// extern crate jsonrpc_client_core;
// extern crate jsonrpc_client_http;
// extern crate serde_json;
// extern crate url;
// extern crate reqwest;

use dotenv::dotenv;
use log::{debug, error};
use roudoudou::{DBService, Error, OdooClient};
pub use serde_json::json;
pub use serde_json::{Map, Number, Value};
use std::env;

macro_rules! oo_test {
    () => {
        json!([])
    };
    ($left:ident == $right:expr) => {
        json!([stringify!($left), '=', $right])
    };
}
fn main() -> Result<(), Error> {
    dotenv().ok();

    let mut cli = OdooClient::new();

    let version = match cli.api.version_info() {
        Ok(version) => version,
        Err(err) => {
            eprintln!("Could not get version info from server: {:#?}", err);
            return Err(err);
        }
    };
    println!("version: {:#?}", version);
    let cli = cli.login("ota3", "admin", "admin").unwrap();
    let db = DBService::new(cli);
    println!("logged in");
    println!("calling db list ...");
    let dblist = db.list().unwrap();
    println!("db.list: {:#?}", dblist);
    println!("field get ...");
    let stock_label = cli.get_model("stock.label").unwrap();
    let empty_dom = oo_test!();
    let basic_eq = oo_test!(foo == true);

    assert_eq!(empty_dom, json!([]));
    assert_eq!(basic_eq, json![("foo", "=", true)]);

    match stock_label.search(json!([
        ("is_terminal", "=", true),
        ("id", ">=", 1000),
        ("id", "<=", 1010)
    ])) {
        Ok(ids) => {
            match stock_label.read(
                &ids,
                &vec![
                    "name",
                    "product_id",
                    "product_tag_ids",
                    "is_terminal",
                    "location_id",
                    "state",
                ],
            ) {
                Err(err) => {
                    error!("search error: {}", err);
                }
                Ok(objs) => {
                    debug!("objects: {:#?}", objs);
                    for item in objs {
                        println!("item: {:?}", item);
                    }
                }
            }
        }
        Err(err) => {
            println!("search error: {}", err);
        }
    };

    let model = cli.get_model("res.users").unwrap();
    let users = match model.search(json!([("id", ">", 0), ("id", "<", 10)])) {
        Ok(a) => model.browse(&a),
        _ => unreachable!(),
    }
    .unwrap();
    println!("XXX: users: {:?}", users);

    let model = cli.get_model("ir.module.module").unwrap();
    let ids = model.search(json!([("name", "=", "mrp_fixes")])).unwrap();
    let module = model.browse(&ids).unwrap();
    println!("module: {:?}", module);
    //println!("module data: {:#?}", module.data);
    println!("module name: {:?}", module.attr("name"));

    let model = cli.get_model("stock.label").unwrap();
    let methods = model.get_methods();
    println!("methods: {:#?}", methods);


    let ids = model
        .search(json!([("name", "=", "352719110488433")]))
        .unwrap();
    let term = model.browse(&ids).unwrap();
    println!("terminal: {:?}", term);
    println!(
        "name: {:?} is_terminal: {:?} is_diabeloop: {:?}",
        term.attr("name"),
        term.attr("is_terminal"),
        term.attr("is_diabeloop")
    );
    let update = json!({"os": {"version": "0.1.0", "apk": "foobar.apk"}});

    let card = model
        .browse(
            &model
                .search(json!([("name", "=", "89351060000893708683")]))
                .unwrap(),
        )
        .unwrap();

    let args = Some(json!(update.to_string()));
    let res = card.call("servicing_ota_update", args, None);
    println!("servicing_ota result: {:?}", res);

    let res = term.call(
        "foobar",
        Some(json!((1, 2, 3))),
        Some(json!({"say": "Viva l'Algérie !"})),
    );
    println!("foobar result: {:#?}", res);

    match env::var("DB_PASSWORD") {
        Ok(password) => {
            println!("calling db dump ...");
            let res = db.dump(&password, "tec-528", "dump.zip");
            println!("res: {:?}", res);
            println!("db drop ...");
            let res = db.drop("diabeloop", "test2");
            match res {
                Ok(val) => {
                    println!("drop: {:#?}", val);
                }
                Err(err) => {
                    println!("err: {:#?}", err);
                }
            };

            println!("db create ...");
            let res = db.create(&password, "test2", false, "fr_FR", "admin");
            match res {
                Ok(val) => {
                    println!("create: {:#?}", val);
                }
                Err(err) => {
                    println!("error: {:#?}", err);
                }
            };
            println!("db drop ...");
            let res = db.drop(&password, "test2");
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

    match cli.api.logout() {
        Ok(res) => println!("ok, logged out: {}", res),
        Err(err) => println!("ouch, could not logout: {}", err),
    }

    Ok(())
}
