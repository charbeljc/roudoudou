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
use rudodoo::{odoo_url_from_env, SessionInfo, OdooClient, OdooApi, Odoo};

// #[derive(Debug, Serialize)]
// pub struct RpcRequest<'a> {
//     jsonrpc: &'a str,
//     method: &'a str,
//     id: u32,
//     params: Value,
// }
// #[derive(Debug, Serialize, Deserialize)]
// pub struct UserContext {
//     current_week: String,
//     current_week2: String,
//     lang: String,
//     tz: String,
// }
// #[derive(Debug, Serialize, Deserialize)]
// pub struct SessionInfo {
//     company_id: i32,
//     db: String,
//     partner_id: i32,
//     registered_contract: String,
//     session_id: String,
//     uid: i32,
//     user_context: UserContext,
//     username: String,
// }

// const ODOO_SERVER_VERSION: &str = "/web/webclient/version_info";
// const ODOO_LOGIN: &str = "/web/session/authenticate";
// const ODOO_LOGOUT: &str = "/web/session/destroy";
// const ODOO_JSONRPC: &str = "/jsonrpc";

// const JSONRPC_20: &str = "2.0";

// pub struct OdooClient {
//     base_url: Url,
//     http: reqwest::Client,
// }

// impl OdooClient {
//     pub fn new() -> Self {
//         Self { base_url: odoo_url_from_env().unwrap(),
//         http: reqwest::Client::new() }
//     }
// }

// trait OdooRpc {
//     fn encode_call<'a>(&self, method: &'a str, params: Value) -> RpcRequest<'a> {
//         RpcRequest {
//             jsonrpc: JSONRPC_20,
//             method: method,
//             id: 1,
//             params: params
//         }
//     }
//     fn send_call(&self, endpoint: &str, payload: RpcRequest); // -> reqwest::Result<reqwest::Response>;
//     fn decode_response(&self, response: reqwest::Response) -> Value;
//     fn download_base64(&self, response: reqwest::Response, path: &Path);
// }

// impl OdooRpc for OdooClient {
//     fn send_call(&self, endpoint: &str, payload: RpcRequest) { // -> reqwest::Result<reqwest::Response> {
//         let j = serde_json::to_string(&payload).unwrap();
//         println!("to string: {:?}", j);
//     }
//     fn decode_response(&self, response: reqwest::Response) -> Value {
//         println!("decode_response: {:?}", response);
//         json!({})
//     }
//     fn download_base64(&self, response: reqwest::Response, path: &Path) {
//         println!("download file: {:?} to {:?}", response, path);
//     }
// }

// #[derive(Default, Debug)]
// struct Odoo {
//     host: String,
//     port: u16,
//     protocol: String,
//     version: String,
// }

// pub struct OdooApi {
//     version_handle: HttpHandle,
//     login_handle: HttpHandle,
//     logout_handle: HttpHandle,
//     jsonrpc_handle: HttpHandle,
//     cli: OdooClient,
// }

// impl OdooApi {
//     pub fn new(cli: OdooClient) -> Self {


//         let transport = HttpTransport::new().standalone().unwrap();
//         let version_url = cli.base_url.join(ODOO_SERVER_VERSION).unwrap();
//         let login_url = cli.base_url.join(ODOO_LOGIN).unwrap();
//         let logout_url = cli.base_url.join(ODOO_LOGOUT).unwrap();
//         let jsonrpc_url = cli.base_url.join(ODOO_JSONRPC).unwrap();

//         OdooApi {
//             cli: cli,
//             version_handle: transport.handle(version_url.as_str()).unwrap(),
//             login_handle: transport.handle(login_url.as_str()).unwrap(),
//             logout_handle: transport.handle(logout_url.as_str()).unwrap(),
//             jsonrpc_handle: transport.handle(jsonrpc_url.as_str()).unwrap(),
//         }
//     }
//     pub fn login(&mut self, db: &str, login: &str, password: &str) -> Result<SessionInfo, Error> {
//         let params = json!({"db": db, "login": login, "password": password});
        
//         let payload = self.cli.encode_call("call", params.clone());
//         println!("login payload: {:?}", payload);
//         println!("raw: {}", serde_json::to_string_pretty(&payload).unwrap());

//         let result = call_method(
//             &mut self.login_handle,
//             "call".to_owned(),
//             params
//         )
//         .call();
//         let session = match result {

//             Ok(value) => Ok(serde_json::from_value::<SessionInfo>(value).unwrap()),
//             Err(err) => Err(err)
//         };
//         println!("session: {:?}", session);
//         session
//     }
//     pub fn logout(&mut self) -> Result<Value, Error> {
//         call_method(&mut self.logout_handle, "call".to_owned(), json!({})).call()
//     }
//     pub fn db_list(&mut self) -> Result<Value, Error> {
//         call_method(
//             &mut self.jsonrpc_handle,
//             "call".to_owned(),
//             json!({
//                 "service": "db",
//                 "method": "list",
//                 "args": []
//             }),
//         )
//         .call()
//     }
//     pub fn db_dump(
//         &mut self,
//         master_password: &str,
//         db: &str,
//         format: &str,
//     ) -> Result<Value, Error> {
//         call_method(
//             &mut self.jsonrpc_handle,
//             "call".to_owned(),
//             json!({
//                 "service": "db",
//                 "method": "dump",
//                 "args": [master_password, db, format]
//             }),
//         )
//         .call()
//     }
//     pub fn db_create(
//         &mut self,
//         master_password: &str,
//         db: &str,
//         demo: bool,
//         lang: &str,
//         admin_password: &str,
//     ) -> Result<Value, Error> {
//         call_method(
//             &mut self.jsonrpc_handle,
//             "call".to_owned(),
//             json!({
//                 "service": "db",
//                 "method": "create_database",
//                 "args": [master_password, db, demo, lang, admin_password]
//             }),
//         )
//         .call()
//     }
//     pub fn db_drop(&mut self, master_password: &str, db: &str) -> Result<Value, Error> {
//         call_method(
//             &mut self.jsonrpc_handle,
//             "call".to_owned(),
//             json!({
//                 "service": "db",
//                 "method": "drop",
//                 "args": [master_password, db]
//             }),
//         )
//         .call()
//     }
//     pub fn object_fields_get(
//         &mut self,
//         db: &str,
//         uid: u32,
//         login: &str,
//         object: &str,
//     ) -> Result<Value, Error> {
//         call_method(
//             &mut self.jsonrpc_handle,
//             "call".to_owned(),
//             json!({
//                 "service": "object",
//                 "method": "execute",
//                 "args": [db, uid, login, object, "fields_get"]
//             }),
//         )
//         .call()
//     }
//     pub fn object_search(
//         &mut self,
//         db: &str,
//         uid: u32,
//         login: &str,
//         object: &str,
//         domain: Value,
//     ) -> Result<Value, Error> {
//         call_method(
//             &mut self.jsonrpc_handle,
//             "call".to_owned(),
//             json!({
//                 "service": "object",
//                 "method": "execute_kw",
//                 "args": [db, uid, login, object, "search", (domain,),
//                  {
//                     "context": {"lang": "en_US",
//                     "current_week": "2107",
//                     "tz": "Europe/Paris",
//                     "uid": 1,
//                     "current_week2": "2018"
//                 }}]
//             }),
//         )
//         .call()
//     }
//     pub fn object_read(
//         &mut self,
//         db: &str,
//         uid: u32,
//         login: &str,
//         object: &str,
//         ids: Value,
//         fields: Value,
//     ) -> Result<Value, Error> {
//         call_method(
//             &mut self.jsonrpc_handle,
//             "call".to_owned(),
//             json!({
//                 "service": "object",
//                 "method": "execute_kw",
//                 "args": [db, uid, login, object, "read", (ids, fields),
//                  {
//                     "context": {"lang": "en_US",
//                     "current_week": "2107",
//                     "tz": "Europe/Paris",
//                     "uid": 1,
//                     "current_week2": "2018"
//                 }}]
//             }),
//         )
//         .call()
//     }
// }

// impl Odoo {
//     fn login(&self, db: String, user: String, password: String) {
//         println!("login: db: {}, u: {}, pwd: {}", db, user, password);
//     }
// }
fn main() -> io::Result<()> {
    dotenv().ok();

    let client = OdooClient::new();

    let odoo = Odoo {
        host: match client.base_url.host_str() {
            Some(host) => host.to_owned(),
            None => panic!("no host")
        },
        port: match client.base_url.port() {
            Some(port) => port,
            None => panic!("no port")
        },
        protocol: match client.base_url.scheme() {
            "https" => "jsonrpc+ssl".to_owned(),
            "http" => "jsonrpc".to_owned(),
            other => panic!("unsupported scheme: {}", other)
        },
        version: "0.9".to_owned(),
    };
    println!("Odoo: {:?}", odoo);
    odoo.login("tec-528".to_owned(), "admin".to_owned(), "admin".to_owned());

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
