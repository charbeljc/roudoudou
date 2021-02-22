use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Map;
use serde_json::Value;
use std::{env, fmt};
use url::{ParseError, Url};

use std::default::Default;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::io::Cursor;

use std::collections::BTreeMap;

use log::debug;
use reqwest::blocking::Client;
use reqwest::blocking::Response;

#[macro_use]
extern crate error_chain;
error_chain! {
    errors {
        RpcError(t: Value) {
            description("blrub")
            display("blaaah: {}", t)
        }
        MyOtherError(t: String) {
            description("blrub")
            display("blaaah: {}", t)
        }
    }
    foreign_links {
        Parse(ParseError);
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct VersionInfo {
    pub protocol_version: u16,
    #[serde(rename = "server_serie")]
    pub server_serial: String,
    pub server_version: String,
    server_version_info: Option<(u16, u16, u16, String, u16, String)>,
}

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
/// raw Odoo field descriptor
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct FieldDescriptor {
    pub change_default: bool,
    pub company_dependent: bool,
    pub depends: Vec<String>,
    pub help: Value,
    pub manual: bool,
    pub readonly: bool,
    pub required: bool,
    pub searchable: bool,
    pub sortable: bool,
    pub store: bool,
    pub string: String,
    #[serde(rename = "type")]
    pub type_: String,
}
/// raw Odoo object descriptor
#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectDescriptor {
    /// object name
    pub name: String,
    /// object fields
    pub fields: BTreeMap<String, FieldDescriptor>,
}

impl ObjectDescriptor {
    pub fn get_searchable_fields(&self) -> Vec<(String, &FieldDescriptor)> {
        self.fields
            .iter()
            .filter(|(_, desc)| desc.searchable)
            .map(|(name, desc)| (name.clone(), desc))
            .collect()
    }
    pub fn get_required_fields(&self) -> Vec<(&String, &FieldDescriptor)> {
        self.fields
            .iter()
            .filter(|(_, desc)| desc.required)
            .map(|(name, desc)| (name, desc))
            .collect()
    }
    pub fn get_relational_fields(&self) -> Vec<(&String, &FieldDescriptor)> {
        self.fields
            .iter()
            .filter(|(_, desc)| match desc.type_.as_str() {
                "one2many" | "many2one" | "many2many" => true,
                _ => false,
            })
            .collect()
    }
    pub fn get_scalar_fields(&self) -> Vec<(&String, &FieldDescriptor)> {
        self.fields
            .iter()
            .filter(|(_, desc)| match desc.type_.as_str() {
                "one2many" | "many2one" | "many2many" => false,
                _ => true,
            })
            .collect()
    }

    pub fn show(&self) {
        debug!("object {}", self.name);
        debug!("attributes:");
        for (attr, desc) in self.get_scalar_fields() {
            debug!("scalar: {} = {:#?}\n", attr, desc);
        }
        debug!("relations:");
        for (attr, desc) in self.get_relational_fields() {
            debug!("relation: {} = {:#?}\n", attr, desc);
        }
    }
}
#[derive(Debug, Serialize, PartialEq)]
pub struct RpcRequest<'a> {
    jsonrpc: &'a str,
    method: &'a str,
    id: u32,
    params: Value,
}

#[derive(Debug, Deserialize)]
pub struct RpcResponse {
    jsonrpc: String,
    id: u32,
    result: Value,
}
#[derive(Debug, Deserialize)]
pub struct RpcError {
    jsonrpc: String,
    id: u32,
    error: Value,
}

const ODOO_SERVER_VERSION: &str = "/web/webclient/version_info";
const ODOO_LOGIN: &str = "/web/session/authenticate";
const ODOO_LOGOUT: &str = "/web/session/destroy";
const ODOO_JSONRPC: &str = "/jsonrpc";

const JSONRPC_20: &str = "2.0";

#[derive(Debug)]
pub struct OdooRpc {
    pub base_url: Url,
    http: Client,
}

impl OdooRpc {
    pub fn new() -> Self {
        OdooRpc {
            base_url: odoo_url_from_env().unwrap(),
            http: Client::builder().cookie_store(true).build().unwrap(),
        }
    }
    pub fn encode_query<'a>(&self, method: &'a str, params: Value) -> RpcRequest<'a> {
        RpcRequest {
            jsonrpc: JSONRPC_20,
            method: method,
            id: 1,
            params: params,
        }
    }
    pub fn send_payload(&self, endpoint: &str, payload: RpcRequest) -> Result<Response> {
        let j = serde_json::to_string(&payload).unwrap();
        let req = self
            .http
            .post(endpoint)
            .header("Content-Type", "application/json")
            .body(j);

        req.send().chain_err(|| "could not send payload")
    }

    pub fn decode_response<T: for<'de> Deserialize<'de>>(
        &self,
        resp: Result<Response>,
    ) -> Result<T> {
        match resp {
            Ok(resp) => {
                match resp.text().chain_err(|| "could not get response body") {
                    Ok(raw) => {
                
                        let j = serde_json::from_str::<Value>(&raw).unwrap();
                        debug!("raw response: {:#?}", j);
                        if let Some(_i) = j.get("result") {
                            let resp = serde_json::from_value::<RpcResponse>(j).unwrap();
                            let res: Value = resp.result;
                            // debug!("res: {:#?}", res);
                            let o: T = serde_json::from_value(res).unwrap();
                            Ok(o)
                        } else if let Some(_) = j.get("error") {
                            let rcp_err = serde_json::from_value::<RpcError>(j).unwrap();
                            let res = rcp_err.error;
                            Err(Error::from(ErrorKind::RpcError(res)))
                        } else {
                            Err(Error::from(ErrorKind::MyOtherError("zooo".to_owned())))
                        }
                    }
                    Err(err) => Err(err)
                }
            }
            Err(err) => Err(err)
        }
    }
}

pub struct OdooApi {
    cli: OdooRpc,
    version_url: Url,
    login_url: Url,
    logout_url: Url,
}
/// Odoo Model object
pub struct Model<'a> {
    desc: ObjectDescriptor,
    api: &'a OdooApi,
}
impl fmt::Debug for Model<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.debug_struct("Model")
            .field("name", &self.desc.name)
            .finish()
    }
}

impl Model<'_> {
    pub fn call(
        &self,
        method: &str,
        args: Option<Value>,
        kwargs: Option<Value>,
    ) -> Result<Value> {
        self.api.recordset_call(
            "tec-528",
            1,
            "admin",
            &self.desc.name,
            None,
            method,
            args,
            kwargs,
        )
    }
}
/// Odoo RecordSet
pub struct RecordSet<'a> {
    pub ids: Vec<u32>,
    pub model: &'a Model<'a>,
    pub data: Vec<Value>,
}

impl RecordSet<'_> {
    /// get attribute `name` for the first object of this record set
    pub fn attr(&self, name: &str) -> Option<&Value> {
        let head = &self.data[0];
        match head {
            Value::Object(obj) => obj.get(name),
            _ => {
                unreachable!()
            }
        }
    }
    /// call `method` on this `RecordSet`
    pub fn call(
        &self,
        method: &str,
        args: Option<Value>,
        kwargs: Option<Value>,
    ) -> Result<Value> {
        debug!("call {:?}::{}({:?})", self, method, args);
        self.model.api.recordset_call(
            "tec-528",
            1,
            "admin",
            &self.model.desc.name,
            Some(&self.ids),
            method,
            args,
            kwargs,
        )
    }
}
impl fmt::Debug for RecordSet<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.debug_struct("RecordSet")
            .field("name", &self.model.desc.name)
            .field("ids", &self.ids)
            .finish()
    }
}

impl Model<'_> {
    pub fn search(&self, domain: Value) -> Result<Vec<u32>> {
        self.api
            .object_search("tec-528", 1, "admin", &self.desc.name, domain)
    }

    pub fn browse(&self, ids: Vec<u32>) -> Result<RecordSet> {
        let names: Vec<String> = self
            .desc
            .fields
            .iter()
            .map(|(name, _)| name.clone())
            .collect();
        let data = self
            .api
            .object_read("tec-528", 1, "admin", &self.desc.name, ids.clone(), names);
        match data {
            Ok(data) => Ok(RecordSet {
                ids,
                model: self,
                data: serde_json::from_value::<Vec<Value>>(data).unwrap(),
            }),
            Err(err) => Err(err),
        }
    }
}

pub struct OdooService<'a> {
    pub name: &'a str,
    pub path: &'a str,
}

pub static LOGIN_SERVICE: OdooService = OdooService {
    name: "login",
    path: ODOO_LOGIN,
};
pub static LOGOUT_SERVICE: OdooService = OdooService {
    name: "logout",
    path: ODOO_LOGOUT,
};
pub static DB_SERVICE: OdooService = OdooService {
    name: "db",
    path: ODOO_JSONRPC,
};
pub static OBJECT_SERVICE: OdooService = OdooService {
    name: "object",
    path: ODOO_JSONRPC,
};

impl OdooApi {
    pub fn new(cli: OdooRpc) -> Self {
        let version_url = cli.base_url.join(ODOO_SERVER_VERSION).unwrap();
        let login_url = cli.base_url.join(ODOO_LOGIN).unwrap();
        let logout_url = cli.base_url.join(ODOO_LOGOUT).unwrap();

        let api: OdooApi = Self {
            cli: cli,
            version_url: version_url.clone(),
            login_url: login_url.clone(),
            logout_url: logout_url.clone(),
        };
        api
    }
    // fn decode_response<T>(&mut self, resp)

    pub fn version_info(&self) -> Result<VersionInfo> {
        let params = json!({});
        let payload = self.cli.encode_query("call", params);
        self.cli.decode_response::<VersionInfo>(
            self.cli.send_payload(self.version_url.as_str(), payload),
        )
    }

    pub fn login(
        &self,
        db: &str,
        login: &str,
        password: &str,
    ) -> Result<SessionInfo> {
        let params = json!({"db": db, "login": login, "password": password});
        let payload = self.cli.encode_query("call", params);
        self.cli
            .decode_response::<SessionInfo>(self.cli.send_payload(self.login_url.as_str(), payload))
    }

    pub fn logout(&self) -> Result<Value> {
        let params = json!({});
        let payload = self.cli.encode_query("call", params);
        let resp = self.cli.send_payload(self.logout_url.as_str(), payload);
        match resp {
            Ok(resp) => {
                let data = resp.text().unwrap();
                debug!("data: {}", data);
                Ok(serde_json::from_str(&data).unwrap())
            }
            Err(err) => Err(err),
        }
        //self.decode_response::<Value>(self.cli.send_payload(self.logout_url.as_str(), payload))
    }
    pub fn odoo_service_call(
        &self,
        service: &OdooService,
        method: &str,
        args: Value,
    ) -> Result<Response> {
        let params = json!({
            "service": service.name,
            "method": method,
            "args": args
        });
        let payload = self.cli.encode_query("call", params);
        let endpoint = self.cli.base_url.join(service.path).unwrap();
        let resp = self.cli.send_payload(endpoint.as_str(), payload);
        resp
    }
    pub fn db_list(&self) -> Result<Vec<String>> {
        let resp = self.odoo_service_call(&DB_SERVICE, "list", json!([]));
        self.cli.decode_response::<Vec<String>>(resp)
    }
    pub fn db_dump(
        &self,
        master_password: &str,
        db: &str,
        path: &str,
    ) -> Result<()> {
        let resp = self.odoo_service_call(&DB_SERVICE, "dump", json!([master_password, db, "zip"]));
        let data = self.cli.decode_response::<String>(resp); // FIXME: allocating a whole data dump is bad ...
        match data {
            Ok(data) => {
                let f = File::create(path).unwrap();
                let mut writer = BufWriter::new(f);
                let wrapped_reader = Cursor::new(data);
                debug!("save dump to {} ...", path);
                for line in wrapped_reader.lines() {
                    match line {
                        Ok(val) => {
                            let data = base64::decode(val).unwrap();
                            writer.write(&data).unwrap();
                        }
                        Err(err) => {
                            debug!("err: {:#?}", err);
                        }
                    }
                }
                Ok(())
            }
            Err(err) => Err(err),
        }
    }
    pub fn db_create(
        &self,
        master_password: &str,
        db: &str,
        demo: bool,
        lang: &str,
        admin_password: &str,
    ) -> Result<Value> {
        let resp = self.odoo_service_call(
            &DB_SERVICE,
            "create_database",
            json!([master_password, db, demo, lang, admin_password]),
        );
        let result = self.cli.decode_response::<Value>(resp);
        result
    }

    pub fn db_drop(&self, master_password: &str, db: &str) -> Result<Value> {
        let resp = self.odoo_service_call(&DB_SERVICE, "drop", json!([master_password, db]));
        let result = self.cli.decode_response::<Value>(resp);
        result
    }
    pub fn get_model(&self, name: &str) -> Result<Model> {
        let desc = self.object_fields_get("tec-528", 1, "admin", name);
        match desc {
            Ok(desc) => Ok(Model { desc, api: self }),
            Err(err) => Err(err),
        }
    }
    pub fn object_fields_get(
        &self,
        db: &str,
        uid: u32,
        login: &str,
        object: &str,
    ) -> Result<ObjectDescriptor> {
        let resp = self.odoo_service_call(
            &OBJECT_SERVICE,
            "execute",
            json!([db, uid, login, object, "fields_get"]),
        );
        //prointln!(r#"resp: {:#?}"#, resp);

        match self.cli.decode_response::<Map<String, Value>>(resp) {
            Ok(values) => {
                let mut fields = BTreeMap::<String, FieldDescriptor>::new();
                for (attr, obj) in values.iter() {
                    let desc = serde_json::from_value(obj.to_owned());
                    match desc {
                        Ok(desc) => {
                            fields.insert(attr.to_owned(), desc);
                        }
                        Err(err) => {
                            if let Some(ro) = obj.get("readonly") {
                                let ro: bool = match ro {
                                    Value::Number(n) => {
                                        if n.as_i64() == Some(0) {
                                            false
                                        } else {
                                            true
                                        }
                                    }
                                    _ => false,
                                };
                                let mut changed = obj.clone();
                                debug!("RO: {:?}", ro);
                                changed["readonly"] = json!(ro);
                                let desc = serde_json::from_value(changed).unwrap();
                                // debug!("{} = {:#?}\n", attr, desc);
                                fields.insert(attr.to_owned(), desc);
                            } else {
                                debug!("Could not get field descriptor for {}: {}", attr, err);
                                //debug!("{}\n\n", serde_json::to_string_pretty(value).unwrap());
                            }
                        }
                    }
                }
                Ok(ObjectDescriptor {
                    name: object.to_owned(),
                    fields,
                })
            }
            Err(err) => Err(err),
        }
    }

    pub fn object_search(
        &self,
        db: &str,
        uid: u32,
        login: &str,
        object: &str,
        domain: Value,
    ) -> Result<Vec<u32>> {
        let args = json!(
            [db, uid, login, object, "search", (domain,),
             {
                "context": {
                    "lang": "en_US",
                    "current_week": "2107",
                    "tz": "Europe/Paris",
                    "uid": 1,
                    "current_week2": "2018"
                }
            }]
        );
        let resp = self.odoo_service_call(&OBJECT_SERVICE, "execute_kw", args);
        self.cli.decode_response::<Vec<u32>>(resp)
    }
    pub fn object_read(
        &self,
        db: &str,
        uid: u32,
        login: &str,
        object: &str,
        ids: Vec<u32>,
        fields: Vec<String>,
    ) -> Result<Value> {
        let ids = json!(ids);
        let args = json!(
            [db, uid, login, object, "read", (ids, fields),
             {
                "context": {
                    "lang": "en_US",
                    "current_week": "2107",
                    "tz": "Europe/Paris",
                    "uid": 1,
                    "current_week2": "2018"
                }
            }]
        );
        let resp = self.odoo_service_call(&OBJECT_SERVICE, "execute_kw", args);
        self.cli.decode_response::<Value>(resp)
    }
    pub fn recordset_call(
        &self,
        db: &str,
        uid: u32,
        login: &str,
        object: &str,
        ids: Option<&Vec<u32>>,
        method: &str,
        args: Option<Value>,
        _kwargs: Option<Value>,
    ) -> Result<Value> {
        let args = match (ids, args) {
            (Some(ids), Some(args)) => {
                json!([
                    db, uid, login, object, method, (ids, args),
                    {
                        "context": {
                            "lang": "en_US",
                            "current_week": "2107",
                            "tz": "Europe/Paris",
                            "uid": 1,
                            "current_week2": "2018"
                        }
                    }
                ])
            }
            (Some(ids), None) => json!([
            db, uid, login, object, method, (ids,),
            {
                "context": {
                    "lang": "en_US",
                    "current_week": "2107",
                    "tz": "Europe/Paris",
                    "uid": 1,
                    "current_week2": "2018"
                }
            }
            ]),
            (None, Some(args)) => json!([
            db, uid, login, object, method, (args,),
            {
                "context": {
                    "lang": "en_US",
                    "current_week": "2107",
                    "tz": "Europe/Paris",
                    "uid": 1,
                    "current_week2": "2018"
                }
            }
            ]),
            (None, None) => json!([
            db, uid, login, object, method, [],
            {
                "context": {
                    "lang": "en_US",
                    "current_week": "2107",
                    "tz": "Europe/Paris",
                    "uid": 1,
                    "current_week2": "2018"
                }
            }

            ]),
        };
        let resp = self.odoo_service_call(&OBJECT_SERVICE, "execute_kw", args);
        self.cli.decode_response::<Value>(resp)
    }
}

/// Obtain Odoo Server URL from environment variables
///
/// You can use ODOO_URL or ODOO_HOST and ODOO_PORT.
/// ODOO_URL takes precedence.
pub fn odoo_url_from_env() -> Result<Url> {
    match env::var("ODOO_URL") {
        Ok(url) => Url::parse(&url).chain_err(|| "invalid url"),
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
            )).chain_err(|| "invalid url")
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
        static ref LOCK: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
    }

    #[test]
    fn test_odoo_default_url() {
        let lock = Arc::clone(&LOCK);
        let mut _data = lock.lock().unwrap();

        env::remove_var("ODOO_URL");
        env::remove_var("ODOO_HOST");
        env::remove_var("ODOO_PORT");
        let expected = Url::parse("http://localhost:8069").unwrap();

        assert_eq!(odoo_url_from_env().unwrap(), expected);
    }
    #[test]
    fn test_odoo_url_precedence() {
        let lock = Arc::clone(&LOCK);
        let mut _data = lock.lock().unwrap();
        env::set_var("ODOO_URL", "http://example.com");
        env::set_var("ODOO_HOST", "localhost");
        env::set_var("ODOO_PORT", "8069");

        assert_eq!(odoo_url_from_env().unwrap(), Url::parse("http://example.com").unwrap());
    }

    #[test]
    fn test_odoo_host_port() {
        let lock = Arc::clone(&LOCK);
        let mut _data = lock.lock().unwrap();

        env::remove_var("ODOO_URL");
        env::set_var("ODOO_HOST", "example.com");
        env::set_var("ODOO_PORT", "8068");

        assert_eq!(odoo_url_from_env().unwrap(), Url::parse("http://example.com:8068").unwrap());
    }

    #[test]
    fn test_port_80() {
        let lock = Arc::clone(&LOCK);
        let mut _data = lock.lock().unwrap();

        env::remove_var("ODOO_URL");
        env::set_var("ODOO_HOST", "example.com");
        env::set_var("ODOO_PORT", "80");

        assert_eq!(odoo_url_from_env().unwrap(), Url::parse("http://example.com").unwrap());
    }

    #[test]
    fn test_port_443() {
        let lock = Arc::clone(&LOCK);
        let mut _data = lock.lock().unwrap();

        env::remove_var("ODOO_URL");
        env::set_var("ODOO_HOST", "example.com");
        env::set_var("ODOO_PORT", "443");

        assert_eq!(odoo_url_from_env().unwrap(), Url::parse("https://example.com").unwrap());
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
