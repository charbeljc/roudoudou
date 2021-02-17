extern crate dotenv;
extern crate serde_json;
extern crate jsonrpc_client_core;
extern crate jsonrpc_client_http;
extern crate base64;
use dotenv::dotenv;
use std::env;
use std::io;
use std::io::prelude::*;
use std::io::BufWriter;
use std::fs::File;

pub use serde_json::{Value, Map, Number};
pub use serde_json::json;

use jsonrpc_client_core::{call_method, Error};
use std::io::Read;
use std::io::Cursor;
use jsonrpc_client_http::HttpHandle;
use jsonrpc_client_http::HttpTransport;

#[derive(Default, Debug)]
struct Odoo {
    host: String,
    port: u32,
    protocol: String,
    version: String
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
        call_method(&mut self.login_handle, "call".to_owned(), json!({"db": db, "login": login, "password": password})).call().unwrap()
    }
    pub fn logout(&mut self) -> Value {
        call_method(&mut self.logout_handle, "call".to_owned(), json!({})).call().unwrap()
    }
    pub fn db_list(&mut self) -> Value {
        call_method(&mut self.jsonrpc_handle, "call".to_owned(), json!({
            "service": "db",
            "method": "list",
            "args": []
        })).call().unwrap()
    }
    pub fn db_dump(&mut self, master_password: &str, db: &str, format: &str) -> Result<Value, Error> {
        call_method(&mut self.jsonrpc_handle, "call".to_owned(), json!({
            "service": "db",
            "method": "dump",
            "args": [master_password, db, format]
        })).call()
    }
}

impl Odoo {
    fn login(&self, db: String, user: String, password: String) {
        println!("login: db: {}, u: {}, pwd: {}", db, user, password);
    }
}
fn main () -> io::Result<()> {
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
            _ => "jsonrpc".to_owned()
        },
        version: "0.9".to_owned()
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
            println!("decoding data: {:#?}", &val[0..1000]);
            let mut wrapped_reader = Cursor::new(val);
            let mut decoder = base64::read::DecoderReader::new(
                &mut wrapped_reader, base64::STANDARD
            );
            let mut result = Vec::new();
            decoder.read_to_end(&mut result).unwrap();
            println!("save file ...");
            let f = File::create("dump.zip")?;
            {
                let mut writer = BufWriter::new(f);
                writer.write(&result)?;
            }
            Ok(())
        },
        Ok(_) => {
            println!("Huu");
            Err(std::io::Error::new(std::io::ErrorKind::Other, "oh no!"))
        },
        Err(err) => {
            println!("error: {:#?}", err);
            Err(std::io::Error::new(std::io::ErrorKind::Other, "oh no no no!!!"))
        },
    }
}
