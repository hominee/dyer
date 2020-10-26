//extern crate brotli2;
extern crate bytes;
//extern crate flate2;
extern crate config;
extern crate hyper;
extern crate hyper_tls;
extern crate serde;
extern crate serde_json;

use crate::spider::{parse::get_parser, Entity};
use crate::{item::ParseError, Profile, Task};
use config::Config;
use hyper::header::{HeaderName, HeaderValue};
use hyper::{Body as hBody, Request as hRequest};
use serde::{Deserialize, Serialize};
use std::clone::Clone;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;
use std::io::LineWriter;
use std::io::{BufRead, BufReader, ErrorKind};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Request {
    pub uri: String,
    pub method: String,
    pub headers: Option<std::collections::HashMap<String, String>>,
    pub cookie: Option<std::collections::HashMap<String, String>>,
    pub body: Option<std::collections::HashMap<String, String>>,
    pub able: u64,
    pub created: u64,
    pub parser: Box<dyn Fn(String) -> Result<(Vec<Entity>, Vec<Task>), ParseError>>,
    pub raw_parser: String,
    pub args: Option<HashMap<String, Vec<String>>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RawRequest {
    pub uri: String,
    pub method: String,
    pub headers: Option<std::collections::HashMap<String, String>>,
    pub cookie: Option<std::collections::HashMap<String, String>>,
    pub body: Option<std::collections::HashMap<String, String>>,
    pub able: u64,
    pub created: u64,
    pub raw_parser: String,
    pub args: Option<HashMap<String, Vec<String>>>,
}
unsafe impl Send for Request {}

impl Request {
    pub fn init(self) -> Option<hRequest<hBody>> {
        let mut builder = hRequest::builder();
        // initialize headers
        let headers = builder.headers_mut().unwrap();
        let cookie: String;
        // initialize cookie
        let coo = self.cookie.to_owned();
        match coo {
            Some(cookies) => {
                let mut v: Vec<String> = Vec::new();
                cookies.iter().for_each(|(key, value)| {
                    v.push(format!("{}={}", key, value));
                });
                cookie = v.join(";");

                headers.insert(
                    HeaderName::from_str("cookie").unwrap(),
                    HeaderValue::from_str(&cookie).unwrap(),
                );
            }
            None => {}
        }
        let head = self.headers.to_owned().unwrap();
        head.iter().for_each(|(k, v)| {
            headers.insert(
                HeaderName::from_str(k.as_str()).unwrap(),
                HeaderValue::from_str(v.as_str()).unwrap(),
            );
        });

        // set method and uri
        let builds = builder.uri(self.uri.as_str()).method(self.method.as_str());

        // consume this builder  and create Hyper::Request
        let body = self.body.to_owned();
        match body {
            Some(d) => {
                let mut v: Vec<String> = Vec::new();
                d.iter().for_each(|(key, value)| {
                    v.push(format!("{}={}", key, value));
                });
                let s = v.join("&");
                return Some(builds.body(hBody::from(s)).unwrap());
            }
            None => {}
        }
        None
    }
}

impl Default for Request {
    fn default() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut headers: HashMap<String, String> = HashMap::new();
        headers.insert(
            "Accept".to_owned(),
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8".to_owned(),
        );
        headers.insert("Accept-Encoding".to_owned(), "gzip, deflate, br".to_owned());
        headers.insert("Accept-Language".to_owned(), "en-US,en;q=0.5".to_owned());
        headers.insert("Cache-Control".to_owned(), "no-cache".to_owned());
        headers.insert("Connection".to_owned(), "keep-alive".to_owned());
        headers.insert("Pragma".to_owned(), "no-cache".to_owned());
        headers.insert("Upgrade-Insecure-Requests".to_owned(), "1".to_owned());
        headers.insert(
            "User-Agent".to_owned(),
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:77.0) Gecko/20100101 Firefox/77.0"
                .to_owned(),
        );
        Request {
            parser: get_parser("".to_owned()),
            uri: "".to_owned(),
            method: "GET".to_owned(),
            headers: Some(headers),
            cookie: None,
            body: None,
            able: now,
            created: now,
            raw_parser: "".to_owned(),
            args: None,
        }
    }
}

impl Request {
    pub fn stored(reqs: Arc<Mutex<Vec<Request>>>) {
        let mut setting = Config::default();
        setting.merge(config::File::with_name("setting")).unwrap();
        let path = setting.get_str("path_request").unwrap() + "/request.txt";
        let file = fs::File::open(path).unwrap();
        let mut writer = LineWriter::new(file);
        reqs.lock().unwrap().iter().for_each(|req| {
            serde_json::to_writer(&mut writer, &Request::into_raw(req)).unwrap();
        });
    }
    pub fn load() -> Option<Vec<Request>> {
        let mut setting = Config::default();
        setting
            // load from file
            .merge(config::File::with_name("setting"))
            .unwrap();
        // load from PATH
        //.merge(config::Environment::with_prefix("APP")).unwrap();
        match setting.get_str("path_request") {
            Ok(path) => {
                // load Profile here
                let file = fs::File::open(path.clone() + "/request.txt");
                match file {
                    Err(e) => match e.kind() {
                        ErrorKind::NotFound => {
                            // create request_old file and  old file
                            fs::File::create(path.clone() + "/request.txt").unwrap();
                            fs::File::create(path + "/request_old.txt").unwrap();
                            return None;
                        }
                        _ => unreachable!(),
                    },
                    Ok(content) => {
                        let buf = BufReader::new(content).lines();
                        let mut data: Vec<Request> = Vec::new();
                        buf.into_iter().for_each(|line| {
                            let raw_req: RawRequest = serde_json::from_str(&line.unwrap()).unwrap();
                            let req = Request::from_raw(raw_req);
                            data.push(req);
                        });
                        // remove request_old file and rename current file to old file
                        fs::remove_file(path.clone() + "/request_old.txt").unwrap();
                        fs::rename(path.clone() + "/request.txt", path + "request_old.txt")
                            .unwrap();
                        return Some(data);
                    }
                }
            }
            Err(_) => {
                // file not found
                panic!("path_request is not configrated in setting.rs");
            }
        }
    }
}

impl Request {
    pub fn from_profile(&mut self, profile: Profile) {
        if let Some(mut cookie) = self.cookie.to_owned() {
            cookie.extend(profile.cookie.unwrap());
        } else {
            self.cookie = profile.cookie;
        }
        if let Some(mut headers) = self.headers.to_owned() {
            headers.extend(profile.headers.unwrap());
        } else {
            self.headers = profile.headers;
        }
        if self.able < profile.able {
            self.able = profile.able;
        }
    }
    pub fn from_task(&mut self, task: Task) {
        self.uri = task.uri;
        self.method = task.method;
        if let Some(mut headers) = self.headers.to_owned() {
            headers.extend(task.headers.unwrap());
        } else {
            self.headers = task.headers;
        }
        self.args = task.args;
        self.parser = task.parser;
        self.body = task.body;
        if self.able < task.able {
            self.able = task.able;
        }
    }
    pub fn from_raw(raw_req: RawRequest) -> Request {
        let parser = get_parser(raw_req.raw_parser.clone());
        Request {
            parser,
            uri: raw_req.uri,
            method: raw_req.method,
            body: raw_req.body,
            headers: raw_req.headers,
            cookie: raw_req.cookie,
            able: raw_req.able,
            created: raw_req.created,
            raw_parser: raw_req.raw_parser,
            args: raw_req.args,
        }
    }

    pub fn into_raw(req: &Request) -> RawRequest {
        RawRequest {
            uri: req.uri.clone(),
            method: req.method.clone(),
            body: req.body.clone(),
            headers: req.headers.clone(),
            cookie: req.cookie.clone(),
            able: req.able.clone(),
            created: req.created.clone(),
            raw_parser: req.raw_parser.clone(),
            args: req.args.clone(),
        }
    }
}
