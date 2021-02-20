use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use std::env;
use std::path::Path;
use url::{ParseError, Url};

use jsonrpc_client_core::{call_method, Error};
use jsonrpc_client_http::HttpHandle;
use jsonrpc_client_http::HttpTransport;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufWriter;
use std::io::Cursor;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserContext {
    current_week: String,
    current_week2: String,
    pub lang: String,
    pub tz: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionInfo {
    company_id: i32,
    db: String,
    partner_id: i32,
    registered_contract: String,
    session_id: String,
    uid: i32,
    user_context: UserContext,
    username: String,
}

#[derive(Debug, Serialize)]
struct RpcRequest<'a> {
    jsonrpc: &'a str,
    method: &'a str,
    id: u32,
    params: Value,
}

const ODOO_SERVER_VERSION: &str = "/web/webclient/version_info";
const ODOO_LOGIN: &str = "/web/session/authenticate";
const ODOO_LOGOUT: &str = "/web/session/destroy";
const ODOO_JSONRPC: &str = "/jsonrpc";

const JSONRPC_20: &str = "2.0";

pub struct OdooClient {
    pub base_url: Url,
    http: reqwest::blocking::Client,
}

impl OdooClient {
    pub fn new() -> Self {
        Self {
            base_url: odoo_url_from_env().unwrap(),
            http: reqwest::blocking::Client::new(),
        }
    }
    fn encode_call<'a>(&self, method: &'a str, params: Value) -> RpcRequest<'a> {
        RpcRequest {
            jsonrpc: JSONRPC_20,
            method: method,
            id: 1,
            params: params,
        }
    }
    fn send_call(&self, endpoint: &str, payload: RpcRequest) -> reqwest::Result<reqwest::blocking::Response> {
        // -> reqwest::Result<reqwest::Response> {
        let j = serde_json::to_string(&payload).unwrap();
        println!("to string: {:?}", j);
        let res = self.http.post(endpoint).body(j).send()?;

        Ok(res)
    }
    fn decode_response(&self, response: reqwest::Response) -> Value {
        println!("decode_response: {:?}", response);
        json!({})
    }
    fn download_base64(&self, response: reqwest::Response, path: &Path) {
        println!("download file: {:?} to {:?}", response, path);
    }
}

pub struct OdooApi {
    login_url: Url,
    version_handle: HttpHandle,
    login_handle: HttpHandle,
    logout_handle: HttpHandle,
    jsonrpc_handle: HttpHandle,
    cli: OdooClient,
}

impl OdooApi {
    pub fn new(cli: OdooClient) -> Self {
        let transport = HttpTransport::new().standalone().unwrap();
        let version_url = cli.base_url.join(ODOO_SERVER_VERSION).unwrap();
        let login_url = cli.base_url.join(ODOO_LOGIN).unwrap();
        let logout_url = cli.base_url.join(ODOO_LOGOUT).unwrap();
        let jsonrpc_url = cli.base_url.join(ODOO_JSONRPC).unwrap();

        OdooApi {
            cli: cli,
            login_url: login_url.clone(),
            version_handle: transport.handle(version_url.as_str()).unwrap(),
            login_handle: transport.handle(login_url.as_str()).unwrap(),
            logout_handle: transport.handle(logout_url.as_str()).unwrap(),
            jsonrpc_handle: transport.handle(jsonrpc_url.as_str()).unwrap(),
        }
    }
    pub fn login(&mut self, db: &str, login: &str, password: &str) -> Result<SessionInfo, Error> {
        let params = json!({"db": db, "login": login, "password": password});
        let payload = self.cli.encode_call("call", params.clone());
        println!("login payload: {:?}", payload);
        println!("raw: {}", serde_json::to_string_pretty(&payload).unwrap());

        let result = call_method(&mut self.login_handle, "call".to_owned(), params).call();
        let session = match result {
            Ok(value) => Ok(serde_json::from_value::<SessionInfo>(value).unwrap()),
            Err(err) => Err(err),
        };
        println!("session: {:?}", session);
        let alt = self.cli.send_call(self.login_url.as_str(), payload);
        println!("alt: {:?}", alt.unwrap());
        session
    }
    pub fn logout(&mut self) -> Result<Value, Error> {
        call_method(&mut self.logout_handle, "call".to_owned(), json!({})).call()
    }
    pub fn db_list(&mut self) -> Result<Value, Error> {
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
    }
    pub fn db_dump(
        &mut self,
        master_password: &str,
        db: &str,
        path: &str,
    ) -> Result<(), io::Error> {
        let result = call_method(
            &mut self.jsonrpc_handle,
            "call".to_owned(),
            json!({
                "service": "db",
                "method": "dump",
                "args": [master_password, db, "zip"]
            }),
        )
        .call();
        match result {
            Ok(Value::String(val)) => {
                // println!("decoding data:\n{:#?}", &val[0..1000]);
                let f = File::create("dump.zip").unwrap();
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
                Ok(())
            }
            Ok(_) => {
                println!("Huu");
                Err(io::Error::new(io::ErrorKind::Other, "Huu ..."))
            }
            Err(err) => {
                println!("error: {:#?}", err);
                Err(io::Error::new(io::ErrorKind::Other, err.to_string()))
            }
        }
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
    pub fn object_fields_get(
        &mut self,
        db: &str,
        uid: u32,
        login: &str,
        object: &str,
    ) -> Result<Value, Error> {
        call_method(
            &mut self.jsonrpc_handle,
            "call".to_owned(),
            json!({
                "service": "object",
                "method": "execute",
                "args": [db, uid, login, object, "fields_get"]
            }),
        )
        .call()
    }
    pub fn object_search(
        &mut self,
        db: &str,
        uid: u32,
        login: &str,
        object: &str,
        domain: Value,
    ) -> Result<Value, Error> {
        call_method(
            &mut self.jsonrpc_handle,
            "call".to_owned(),
            json!({
                "service": "object",
                "method": "execute_kw",
                "args": [db, uid, login, object, "search", (domain,),
                 {
                    "context": {"lang": "en_US",
                    "current_week": "2107",
                    "tz": "Europe/Paris",
                    "uid": 1,
                    "current_week2": "2018"
                }}]
            }),
        )
        .call()
    }
    pub fn object_read(
        &mut self,
        db: &str,
        uid: u32,
        login: &str,
        object: &str,
        ids: Value,
        fields: Value,
    ) -> Result<Value, Error> {
        call_method(
            &mut self.jsonrpc_handle,
            "call".to_owned(),
            json!({
                "service": "object",
                "method": "execute_kw",
                "args": [db, uid, login, object, "read", (ids, fields),
                 {
                    "context": {"lang": "en_US",
                    "current_week": "2107",
                    "tz": "Europe/Paris",
                    "uid": 1,
                    "current_week2": "2018"
                }}]
            }),
        )
        .call()
    }
}

/// Obtain Odoo Server URL from environment variables
///
/// You can use ODOO_URL or ODOO_HOST and ODOO_PORT.
/// ODOO_URL takes precedence.
pub fn odoo_url_from_env() -> Result<Url, ParseError> {
    match env::var("ODOO_URL") {
        Ok(url) => Url::parse(&url),
        Err(_) => {
            let odoo_host = match env::var("ODOO_HOST") {
                Ok(val) => val,
                Err(_) => "localhost".to_owned(),
            };
            let odoo_port = match env::var("ODOO_PORT") {
                Ok(val) => val.parse().map_err(|_e| ParseError::InvalidPort),
                Err(_) => Ok(8069),
            }?;
            let scheme = if odoo_port == 443 { "https" } else { "http" };
            Url::parse(&format!(
                "{}:{}{}",
                scheme,
                odoo_host,
                match odoo_port {
                    443 | 80 => "".to_owned(),
                    port => format!(":{}", port),
                }
            ))
        }
    }
}

#[cfg(test)]
#[macro_use]
extern crate lazy_static;

#[cfg(test)]
mod tests {
    use crate::odoo_url_from_env;
    use std::env;
    use std::sync::{Arc, Mutex};
    use url::Url;
    lazy_static! {
        static ref LOCK: Arc<Mutex<u32>> = { Arc::new(Mutex::new(0)) };
    }

    #[test]
    fn test_odoo_default_url() {
        let lock = Arc::clone(&LOCK);
        let mut _data = lock.lock().unwrap();

        env::remove_var("ODOO_URL");
        env::remove_var("ODOO_HOST");
        env::remove_var("ODOO_PORT");

        assert_eq!(odoo_url_from_env(), Url::parse("http://localhost:8069"));
    }
    #[test]
    fn test_odoo_url_precedence() {
        let lock = Arc::clone(&LOCK);
        let mut _data = lock.lock().unwrap();
        env::set_var("ODOO_URL", "http://example.com");
        env::set_var("ODOO_HOST", "localhost");
        env::set_var("ODOO_PORT", "8069");

        assert_eq!(odoo_url_from_env(), Url::parse("http://example.com"));
    }

    #[test]
    fn test_odoo_host_port() {
        let lock = Arc::clone(&LOCK);
        let mut _data = lock.lock().unwrap();

        env::remove_var("ODOO_URL");
        env::set_var("ODOO_HOST", "example.com");
        env::set_var("ODOO_PORT", "8068");

        assert_eq!(odoo_url_from_env(), Url::parse("http://example.com:8068"));
    }

    #[test]
    fn test_port_80() {
        let lock = Arc::clone(&LOCK);
        let mut _data = lock.lock().unwrap();

        env::remove_var("ODOO_URL");
        env::set_var("ODOO_HOST", "example.com");
        env::set_var("ODOO_PORT", "80");

        assert_eq!(odoo_url_from_env(), Url::parse("http://example.com"));
    }

    #[test]
    fn test_port_443() {
        let lock = Arc::clone(&LOCK);
        let mut _data = lock.lock().unwrap();

        env::remove_var("ODOO_URL");
        env::set_var("ODOO_HOST", "example.com");
        env::set_var("ODOO_PORT", "443");

        assert_eq!(odoo_url_from_env(), Url::parse("https://example.com"));
    }

    #[test]
    fn test_invalid_url() {
        let lock = Arc::clone(&LOCK);
        let mut _data = lock.lock().unwrap();

        env::set_var("ODOO_URL", "http://foooobar:zorgl");

        assert!(odoo_url_from_env().is_err());
    }

    #[test]
    fn test_invalid_port() {
        let lock = Arc::clone(&LOCK);
        let mut _data = lock.lock().unwrap();

        env::remove_var("ODOO_URL");
        env::set_var("ODOO_HOST", "foobar");
        env::set_var("ODOO_PORT", "foobar");

        assert!(odoo_url_from_env().is_err());
    }
}
