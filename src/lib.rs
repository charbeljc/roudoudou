use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Map;
use serde_json::Value;
use std::{env, fmt};
use url::{ParseError, Url};
use std::fs;

use std::default::Default;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::io::Cursor;

use std::collections::BTreeMap;

use log::{debug, info};
use reqwest::blocking::Client;
use reqwest::blocking::Response;
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};

lazy_static! {
    static ref USER_MUTEX: Arc<Mutex<u16>> = Arc::new(Mutex::new(0u16));
}


#[macro_use]
extern crate error_chain;
error_chain! {
    errors {
        RpcError(t: ServerError) {
            description("blrub")
            display("blaaah: {:?}", t)
        }
        MyOtherError(t: String) {
            description("blrub")
            display("blaaah: {}", t)
        }
        ClientState(t: String) {
            description("odoo client state error")
            display("ClientState Error: {}", t)
        }
        NotConnected {
            description("odoo client must be connected")
            display("not connected")
        }
    }
    foreign_links {
        ParseError(ParseError);
        JsonError(serde_json::Error);
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OString {
    Filled(String),
    Absent(bool),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct VersionInfo {
    pub protocol_version: u16,
    #[serde(rename = "server_serie")]
    pub server_serial: OString,
    pub server_version: OString,
    server_version_info: Option<(u16, u16, u16, String, u16, String)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserContext {
    current_week: OString,
    current_week2: OString,
    pub lang: OString,
    pub tz: OString,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionInfo {
    pub company_id: u32,
    pub db: String,
    pub partner_id: u32,
    pub registered_contract: OString,
    pub session_id: String,
    pub uid: u32,
    pub user_context: UserContext,
    pub username: String,
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
    error: ServerError,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerError {
    pub code: u16,
    pub data: OdooError,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct OdooError {
    pub name: String,
    pub message: String,
    pub exception_type: String,
    pub arguments: Vec<Value>,
    // #[serde(serialize_with = "njoin")]
    // #[serde(deserialize_with = "nsplit")]
    // pub debug: Vec<String>,
    pub debug: String,
}
// fn njoin<S>(debug: Vec<String>, ser: S) -> Result<S::Ok> where S: Serializer {
//     let json = serde_json::json!(debug);
//     Ok(json)
// }

// fn nsplit<'de, D>(dede: D) -> Result<Vec<String>> where D: Deserializer<'de> {
//     let empty: Vec::new();
//     Ok(empty)
// }

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
                        // debug!("raw response: {}", raw);
                        let j = serde_json::from_str::<Value>(&raw).unwrap();
                        // debug!("serde response: {:#?}", j);
                        if let Some(_i) = j.get("result") {
                            let resp = serde_json::from_value::<RpcResponse>(j).unwrap();
                            let res: Value = resp.result;
                            // debug!("res: {:#?}", res);
                            match serde_json::from_value::<T>(res) {
                                Ok(o) => Ok(o),
                                Err(err) => {
                                    debug!("FAILED to deserialize res: {:#?}", err);
                                    Err(Error::from(ErrorKind::JsonError(err)))
                                }
                            }
                        } else if let Some(_) = j.get("error") {
                            let rcp_err = serde_json::from_value::<RpcError>(j).unwrap();
                            let res = rcp_err.error;

                            Err(Error::from(ErrorKind::RpcError(res)))
                        } else {
                            Err(Error::from(ErrorKind::MyOtherError(format!(
                                "Unknown payload: {:#}?",
                                raw
                            ))))
                        }
                    }
                    Err(err) => Err(err),
                }
            }
            Err(err) => Err(err),
        }
    }
}

#[derive(Debug)]
pub struct OdooApi {
    rpc: OdooRpc,
    version_url: Url,
    login_url: Url,
    logout_url: Url,
}

#[derive(Debug)]
pub struct OdooClient {
    pub api: OdooApi,
    session: Option<SessionInfo>,
}

impl OdooClient {
    pub fn new() -> Self {
        let rpc = OdooRpc::new();
        OdooClient {
            api: OdooApi::new(rpc),
            session: None,
        }
    }
    pub fn is_connected(&self) -> bool {
        if let Some(_) = self.session {
            true
        } else {
            false
        }
    }
    pub fn login(&mut self, db: &str, user: &str, password: &str) -> Result<&mut Self> {
        if self.is_connected() {
            return Err(Error::from_kind(ErrorKind::ClientState(
                "already connected".to_owned(),
            )));
        }
        let session = self.api.login(db, user, password);
        match session {
            Err(e) => Err(e),
            Ok(session) => {
                self.session = Some(session);
                Ok(self)
            }
        }
    }
    pub fn logout(&mut self) -> Result<&mut Self> {
        if !self.is_connected() {
            return Err(Error::from_kind(ErrorKind::ClientState(
                "already connected".to_owned(),
            )));
        }
        let res = self.api.logout();
        match res {
            Err(e) => Err(e),
            Ok(val) => {
                debug!("logout result: {:#?}", val);
                Ok(self)
            }
        }
    }
    pub fn get_model(&self, name: &str) -> Result<Model> {
        match &self.session {
            None => Err(Error::from_kind(ErrorKind::ClientState(
                "not connected".to_owned(),
            ))),
            Some(session) => {
                match self
                    .api
                    .object_fields_get(&session.db, session.uid, &session.username, name)
                {
                    Ok(desc) => Ok(Model { desc, cli: self }),
                    Err(err) => Err(err),
                }
            }
        }
    }
}
/// Odoo Model object
pub struct Model<'a> {
    desc: ObjectDescriptor,
    cli: &'a OdooClient,
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum MethodKind {
    #[serde(alias = "multi")]
    Multi,
    #[serde(alias = "model")]
    Model,
    #[serde(alias = "one")]
    One,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Method {
    pub name: String,
    pub kind: MethodKind
}


impl fmt::Debug for Model<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.debug_struct("Model")
            .field("name", &self.desc.name)
            .finish()
    }
}

impl Model<'_> {
    pub fn call(&self, method: &str, args: Option<Value>, kwargs: Option<Value>) -> Result<Value> {
        match &self.cli.session {
            None => Err(Error::from_kind(ErrorKind::ClientState(
                "not connected".to_owned(),
            ))),
            Some(session) => self.cli.api.recordset_call(
                &session.db,
                1,
                "admin",
                &self.desc.name,
                None,
                method,
                args,
                kwargs,
            ),
        }
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
    pub fn get(&self, name: &str) -> Option<&Value> {
        let head = &self.data[0];
        match head {
            Value::Object(obj) => obj.get(name),
            _ => {
                unreachable!()
            }
        }
    }
    pub fn set<T>(&self, name: &str, value: T) {
        let head = &self.data[0];
    }
    /// call `method` on this `RecordSet`
    pub fn call(&self, method: &str, args: Option<Value>, kwargs: Option<Value>) -> Result<Value> {
        debug!("call {:?}::{}({:?})", self, method, args);
        match &self.model.cli.session {
            None => Err(Error::from_kind(ErrorKind::NotConnected)),
            Some(session) => self.model.cli.api.recordset_call(
                &session.db,
                1,
                "admin",
                &self.model.desc.name,
                Some(&self.ids),
                method,
                args,
                kwargs,
            ),
        }
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

impl<'a> Model<'a> {
    pub fn get_methods(&self) -> Result<Vec<Method>> {
        match self.call("get_public_methods", None, None) {
            Err(err) => Err(err),
            Ok(value) => {
               match serde_json::from_value::<Vec<Method>>(value).chain_err(|| "woops") {
                   Err(err) => Err(err),
                   Ok(meths) => Ok(meths)
               }
                
            }
        }
    }
    pub fn search(&self, domain: Value) -> Result<Vec<u32>> {
        match &self.cli.session {
            None => Err(Error::from_kind(ErrorKind::NotConnected)),
            Some(session) => {
                self.cli
                    .api
                    .object_search(&session.db, 1, "admin", &self.desc.name, domain)
            }
        }
    }

    pub fn browse(&self, ids: &Vec<u32>) -> Result<RecordSet> {
        let names = self
            .desc
            .fields
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<&str>>();
        match self.read(&ids, &names) {
            Err(err) => Err(err),
            Ok(data) => Ok(RecordSet {
                ids: ids.to_owned(),
                model: self,
                data,
            }),
        }
    }

    pub fn search_browse(&self, domain: Value) -> Result<RecordSet> {
        match self.search(domain) {
            Err(err) => Err(err),
            Ok(ids) => self.browse(&ids),
        }
    }

    pub fn read(&self, ids: &Vec<u32>, names: &Vec<&str>) -> Result<Vec<Value>> {
        match &self.cli.session {
            None => Err(Error::from_kind(ErrorKind::NotConnected)),
            Some(session) => {
                let data =
                    self.cli
                        .api
                        .object_read(&session.db, 1, "admin", &self.desc.name, ids, names);
                match data {
                    Err(err) => Err(err),
                    Ok(data) => match serde_json::from_value::<Vec<Value>>(data) {
                        Err(err) => Err(Error::from_kind(ErrorKind::JsonError(err))),
                        Ok(data) => Ok(data),
                    },
                }
            }
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
    pub fn new(rpc: OdooRpc) -> Self {
        let version_url = rpc.base_url.join(ODOO_SERVER_VERSION).unwrap();
        let login_url = rpc.base_url.join(ODOO_LOGIN).unwrap();
        let logout_url = rpc.base_url.join(ODOO_LOGOUT).unwrap();

        let api: OdooApi = Self {
            rpc,
            version_url: version_url.clone(),
            login_url: login_url.clone(),
            logout_url: logout_url.clone(),
        };
        api
    }
    // fn decode_response<T>(&mut self, resp)

    pub fn version_info(&self) -> Result<VersionInfo> {
        let params = json!({});
        let payload = self.rpc.encode_query("call", params);
        self.rpc.decode_response::<VersionInfo>(
            self.rpc.send_payload(self.version_url.as_str(), payload),
        )
    }

    pub fn login(&self, db: &str, login: &str, password: &str) -> Result<SessionInfo> {
        let params = json!({"db": db, "login": login, "password": password});
        let payload = self.rpc.encode_query("call", params);
        let mutex = Arc::clone(&USER_MUTEX);
        let mut login_count = mutex.lock().unwrap();
        let resp = self.rpc.send_payload( self.login_url.as_str(), payload);
        match self.rpc.decode_response::<SessionInfo>(resp) {
            Err(err) => Err(err),
            Ok(session_info) => {

                info!("user logged in: {:#?}", session_info);
                *login_count += 1;
                Ok(session_info)
            }
        }
    }

    pub fn logout(&self) -> Result<Value> {
        let params = json!({});
        let payload = self.rpc.encode_query("call", params);
        let mutex = Arc::clone(&USER_MUTEX);
        let mut login_count = mutex.lock().unwrap();
        let resp = self.rpc.send_payload( self.logout_url.as_str(), payload);
        match self.rpc.decode_response::<Value>(resp) {
            Err(err) => Err(err),
            Ok(resp) => {
                debug!("data: {}", resp);
                *login_count -= 1;
                Ok(resp)
            }
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
        let payload = self.rpc.encode_query("call", params);
        let endpoint = self.rpc.base_url.join(service.path).unwrap();
        let resp = self.rpc.send_payload(endpoint.as_str(), payload);
        resp
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

        match self.rpc.decode_response::<Map<String, Value>>(resp) {
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
                    "current_week": "2108",
                    "tz": "Europe/Paris",
                    "uid": 1,
                    "current_week2": "2109"
                }
            }]
        );
        let resp = self.odoo_service_call(&OBJECT_SERVICE, "execute_kw", args);
        self.rpc.decode_response::<Vec<u32>>(resp)
    }
    pub fn object_read(
        &self,
        db: &str,
        uid: u32,
        login: &str,
        object: &str,
        ids: &Vec<u32>,
        fields: &Vec<&str>,
    ) -> Result<Value> {
        let ids = json!(ids);
        let args = json!(
            [db, uid, login, object, "read", (ids, fields),
             {
                "context": {
                    "lang": "en_US",
                    "current_week": "2108",
                    "tz": "Europe/Paris",
                    "uid": 1,
                    "current_week2": "2109"
                }
            }]
        );
        let resp = self.odoo_service_call(&OBJECT_SERVICE, "execute_kw", args);
        self.rpc.decode_response::<Value>(resp)
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
                            "current_week": "2108",
                            "tz": "Europe/Paris",
                            "uid": 1,
                            "current_week2": "2109"
                        }
                    }
                ])
            }
            (Some(ids), None) => json!([
            db, uid, login, object, method, (ids,),
            {
                "context": {
                    "lang": "en_US",
                    "current_week": "2108",
                    "tz": "Europe/Paris",
                    "uid": 1,
                    "current_week2": "2109"
                }
            }
            ]),
            (None, Some(args)) => json!([
            db, uid, login, object, method, (args,),
            {
                "context": {
                    "lang": "en_US",
                    "current_week": "2108",
                    "tz": "Europe/Paris",
                    "uid": 1,
                    "current_week2": "2109"
                }
            }
            ]),
            (None, None) => json!([
            db, uid, login, object, method, [],
            {
                "context": {
                    "lang": "en_US",
                    "current_week": "2108",
                    "tz": "Europe/Paris",
                    "uid": 1,
                    "current_week2": "2109"
                }
            }

            ]),
        };
        let resp = self.odoo_service_call(&OBJECT_SERVICE, "execute_kw", args);
        self.rpc.decode_response::<Value>(resp)
    }
}

#[derive(Debug)]
pub struct DBService<'a> {
    cli: &'a OdooClient
}
impl<'a> DBService<'a> {
    pub fn new(cli: &'a OdooClient) -> Self {
        DBService {
            cli
        }
    }
    
    pub fn list(&self) -> Result<Vec<String>> {
        let resp = self.cli.api.odoo_service_call(&DB_SERVICE, "list", json!([]));
        self.cli.api.rpc.decode_response::<Vec<String>>(resp)
    }
    pub fn dump(&self, master_password: &str, db: &str, path: &str) -> Result<()> {
        let resp = self.cli.api.odoo_service_call(&DB_SERVICE, "dump", json!([master_password, db, "zip"]));
        let data = self.cli.api.rpc.decode_response::<String>(resp); // FIXME: allocating a whole data dump is bad ...
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
    pub fn duplicate(&self, master_password: &str, db: &str, new_db: &str) -> Result<Value> {
        match self.cli.api.odoo_service_call(
            &DB_SERVICE,
            "duplicate_database",
            json!([master_password, db, new_db])
        ) {
            Err(err) =>  { return Err(err) }
            Ok(resp) => {
                let res: Result<Value> = serde_json::from_reader(resp).chain_err(|| "huu") ;

                match res {
                    Ok(data) => Ok(data),
                    Err(err) => Err(err)
                }
            }
        }
        //let data = serde_json::from_reader(resp.into_reader());
    }
    pub fn restore(&self, master_password: &str, db: &str, path: &str, _new_uid: bool) -> Result<()> {
        // ouch ...
        let res: Result<String> = fs::read_to_string(path).chain_err(|| "foobar");
        match res {
            Err(err) => Err(err),
            Ok(content) => {
                let encoded = base64::encode(content);
                
                let resp = self.cli.api.odoo_service_call(
                    &DB_SERVICE, "dump", json!([master_password, db, "zip"]));
                let data = self.cli.api.rpc.decode_response::<Value>(resp); // FIXME: allocating a whole data dump is bad ...

                match data {
                    Err(err) => Err(err),
                    Ok(data) => {
                        debug!("database restored: {:#?}", data);
                        Ok(())
                    }
                }
            }
        }
    }

    pub fn create(
        &self,
        master_password: &str,
        db: &str,
        demo: bool,
        lang: &str,
        admin_password: &str,
    ) -> Result<Value> {
        let resp = self.cli.api.odoo_service_call(
            &DB_SERVICE,
            "create_database",
            json!([master_password, db, demo, lang, admin_password]),
        );
        let result = self.cli.api.rpc.decode_response::<Value>(resp);
        result
    }

    pub fn drop(&self, master_password: &str, db: &str) -> Result<Value> {
        let resp = self.cli.api.odoo_service_call(&DB_SERVICE, "drop", json!([master_password, db]));
        let result = self.cli.api.rpc.decode_response::<Value>(resp);
        result
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
            ))
            .chain_err(|| "invalid url")
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

        assert_eq!(
            odoo_url_from_env().unwrap(),
            Url::parse("http://example.com").unwrap()
        );
    }

    #[test]
    fn test_odoo_host_port() {
        let lock = Arc::clone(&LOCK);
        let mut _data = lock.lock().unwrap();

        env::remove_var("ODOO_URL");
        env::set_var("ODOO_HOST", "example.com");
        env::set_var("ODOO_PORT", "8068");

        assert_eq!(
            odoo_url_from_env().unwrap(),
            Url::parse("http://example.com:8068").unwrap()
        );
    }

    #[test]
    fn test_port_80() {
        let lock = Arc::clone(&LOCK);
        let mut _data = lock.lock().unwrap();

        env::remove_var("ODOO_URL");
        env::set_var("ODOO_HOST", "example.com");
        env::set_var("ODOO_PORT", "80");

        assert_eq!(
            odoo_url_from_env().unwrap(),
            Url::parse("http://example.com").unwrap()
        );
    }

    #[test]
    fn test_port_443() {
        let lock = Arc::clone(&LOCK);
        let mut _data = lock.lock().unwrap();

        env::remove_var("ODOO_URL");
        env::set_var("ODOO_HOST", "example.com");
        env::set_var("ODOO_PORT", "443");

        assert_eq!(
            odoo_url_from_env().unwrap(),
            Url::parse("https://example.com").unwrap()
        );
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
