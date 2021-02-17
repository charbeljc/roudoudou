extern crate base64;
extern crate dotenv;
extern crate jsonrpc_client_core;
extern crate jsonrpc_client_http;
extern crate serde_json;
use dotenv::dotenv;
use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufWriter;

pub use serde_json::json;
pub use serde_json::{Map, Number, Value};

use jsonrpc_client_core::{call_method, Error};
use jsonrpc_client_http::HttpHandle;
use jsonrpc_client_http::HttpTransport;
use std::io::Cursor;

#[derive(Default, Debug)]
struct Odoo {
    host: String,
    port: u32,
    protocol: String,
    version: String,
}

const ODOO_LOGIN: &str = "http://localhost:8069/web/session/authenticate";
const ODOO_LOGOUT: &str = "http://localhost:8069/web/session/destroy";
const ODOO_JSONRPC: &str = "http://localhost:8069/jsonrpc";

pub struct OdooApi {
    login_handle: HttpHandle,
    logout_handle: HttpHandle,
    jsonrpc_handle: HttpHandle,
}

impl OdooApi {
    pub fn new() -> Self {
        let login_transport = HttpTransport::new().standalone().unwrap();
        let logout_transport = HttpTransport::new().standalone().unwrap();
        let jsonrpc_transport = HttpTransport::new().standalone().unwrap();

        OdooApi {
            login_handle: login_transport.handle(ODOO_LOGIN).unwrap(),
            logout_handle: logout_transport.handle(ODOO_LOGOUT).unwrap(),
            jsonrpc_handle: jsonrpc_transport.handle(ODOO_JSONRPC).unwrap(),
        }
    }
    pub fn login(&mut self, db: &str, login: &str, password: &str) -> Value {
        call_method(
            &mut self.login_handle,
            "call".to_owned(),
            json!({"db": db, "login": login, "password": password}),
        )
        .call()
        .unwrap()
    }
    pub fn logout(&mut self) -> Value {
        call_method(&mut self.logout_handle, "call".to_owned(), json!({}))
            .call()
            .unwrap()
    }
    pub fn db_list(&mut self) -> Value {
        call_method(
            &mut self.jsonrpc_handle,
            "call".to_owned(),
            json!({
                "service": "db",
                "method": "list",
                "args": []
            }),
        )
        .call()
        .unwrap()
    }
    pub fn db_dump(
        &mut self,
        master_password: &str,
        db: &str,
        format: &str,
    ) -> Result<Value, Error> {
        call_method(
            &mut self.jsonrpc_handle,
            "call".to_owned(),
            json!({
                "service": "db",
                "method": "dump",
                "args": [master_password, db, format]
            }),
        )
        .call()
    }
    pub fn db_create(
        &mut self,
        master_password: &str,
        db: &str,
        demo: bool,
        lang: &str,
        admin_password: &str,
    ) -> Result<Value, Error> {
        call_method(
            &mut self.jsonrpc_handle,
            "call".to_owned(),
            json!({
                "service": "db",
                "method": "create_database",
                "args": [master_password, db, demo, lang, admin_password]
            }),
        )
        .call()
    }
    pub fn db_drop(&mut self, master_password: &str, db: &str) -> Result<Value, Error> {
        call_method(
            &mut self.jsonrpc_handle,
            "call".to_owned(),
            json!({
                "service": "db",
                "method": "drop",
                "args": [master_password, db]
            }),
        )
        .call()
    }
}

impl Odoo {
    fn login(&self, db: String, user: String, password: String) {
        println!("login: db: {}, u: {}, pwd: {}", db, user, password);
    }
}
fn main() -> io::Result<()> {
    dotenv().ok();
    for (key, env) in env::vars() {
        println!("{}: {}", key, env);
    }
    let port: u32 = match env::var("ODOO_PORT") {
        Ok(val) => val.parse().unwrap(),
        Err(_) => 8069,
    };

    let odoo = Odoo {
        host: match env::var("ODOO_HOST") {
            Ok(val) => val,
            Err(_) => "localhost".to_owned(),
        },
        port: port,
        protocol: match port {
            443 => "jsonrpc+ssl".to_owned(),
            _ => "jsonrpc".to_owned(),
        },
        version: "0.9".to_owned(),
    };
    println!("Odoo: {:?}", odoo);
    odoo.login("tec-528".to_owned(), "admin".to_owned(), "admin".to_owned());

    let mut client = OdooApi::new();
    let res: Value = client.login("tec-528", "admin", "admin");
    println!("login: res: {:#?}", res);
    println!("calling db list ...");
    let dblist: Value = client.db_list();
    println!("db_list: {:#?}", dblist);
    println!("calling db dump ...");
    // let res = client.db_dump("diabeloop", "tec-529", "zip");
    // match res {
    //     Ok(_val) => println!("string"),
    //     Err(err) => println!("error: {:#?}", err),
    // }
    let res = client.db_dump("diabeloop", "tec-528", "zip");
    match res {
        Ok(Value::String(val)) => {
            // println!("decoding data:\n{:#?}", &val[0..1000]);
            let f = File::create("dump.zip")?;
            let mut writer = BufWriter::new(f);
            let wrapped_reader = Cursor::new(val);
            println!("save file ...");
            for line in wrapped_reader.lines() {
                //let data = base64::decode(line.as_bytes()).unwrap();
                match line {
                    Ok(val) => {
                        let data = base64::decode(val).unwrap();
                        writer.write(&data)?;
                    }
                    Err(err) => {
                        println!("err: {:#?}", err);
                    }
                };
            }
            //let data = base64::decode(val).unwrap();
            println!("done.");
        }
        Ok(_) => {
            println!("Huu");
        }
        Err(err) => {
            println!("error: {:#?}", err);
        }
    }
    println!("db create ...");
    let res = client.db_create("diabeloop", "test2", false, "fr_FR", "admin");
    match res {
        Ok(val) => {
            println!("create: {:#?}", val);
        }
        Err(err) => {
            println!("error: {:#?}", err);
        }
    };
    println!("db drop ...");
    let res = client.db_drop("diabeloop", "test2");
    match res {
        Ok(val) => {
            println!("drop: {:#?}", val);
        }
        Err(err) => {
            println!("err: {:#?}", err);
        }
    };
    Ok(())
}
