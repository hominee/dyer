extern crate bytes;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;

use crate::item::{Profile, Request, ResError, Task};
use crate::spider::{parse::get_parser, Entity, ParseError};
use log::{debug, error, info, trace, warn};
//use crate::request::Request;
use hyper::Client as hClient;
use hyper_timeout::TimeoutConnector;
use std::collections::HashMap;
//use std::time::{SystemTime, UNIX_EPOCH};
use bytes::buf::ext::BufExt;
use std::io::{BufReader, Read};

use futures::future::join_all;
use hyper::{client::HttpConnector, Body as hBody, Request as hRequest};
use hyper_tls::HttpsConnector;
use std::sync::{Arc, Mutex};

pub struct Response {
    pub headers: HashMap<String, String>,
    pub status: usize,
    pub content: Option<String>,

    pub body: HashMap<String, String>,
    pub uri: String,
    pub method: String,
    pub cookie: HashMap<String, String>,
    pub created: u64,
    pub raw_parser: String,
    pub parser: Box<dyn Fn(String) -> Result<(Vec<Entity>, Vec<Task>), ParseError> + Send>,
    pub args: HashMap<String, Vec<String>>,
    pub msg: Option<String>,
}

impl Response {
    ///this function require a `hyper::Request` and `hyper::Client` to return the Response

    pub async fn exec(
        req: hRequest<hBody>,
        client: &hClient<TimeoutConnector<HttpsConnector<HttpConnector>>>,
    ) -> Result<(Option<String>, HashMap<String, String>, usize), ResError> {
        let response = client.request(req).await.unwrap();
        let (header, bd) = response.into_parts();
        let bod = hyper::body::aggregate(bd).await;
        match bod {
            Ok(body) => {
                let mut reader = BufReader::new(body.reader());

                let status = header.status.as_u16() as usize;
                let mut headers: HashMap<String, String> = HashMap::new();
                header.headers.into_iter().for_each(|(key, value)| {
                    headers.insert(
                        key.unwrap().to_string(),
                        value.to_str().unwrap().to_string(),
                    );
                });

                // Response Content
                let mut data = String::new();
                reader.read_to_string(&mut data).unwrap();

                Ok((Some(data), headers, status))
            }
            Err(e) => Err(ResError {
                desc: e.into_cause().unwrap().source().unwrap().to_string(),
            }),
        }
    }

    pub async fn exec_one(
        req: Request,
        client: &hClient<TimeoutConnector<HttpsConnector<HttpConnector>>>,
    ) -> Result<Response, ResError> {
        let mut r = Response::default(Some(&req));
        let req = req.init().unwrap();
        let response = Response::exec(req, &client).await;

        match response {
            Ok(data) => {
                r.headers.extend(data.1);
                r.content = data.0;
                r.status = data.2;
            }
            Err(e) => {
                r.msg = Some(e.desc);
            }
        }
        Ok(r)
    }

    // FIXME it's not necessary to return Result, Vec<> will be fine.
    pub async fn exec_all(
        reqs: Vec<Request>,
        client: hClient<TimeoutConnector<HttpsConnector<HttpConnector>>>,
        result: Arc<Mutex<Vec<Response>>>,
    ) {
        let mut v = Vec::new();
        let mut rs = Vec::new();
        reqs.into_iter().for_each(|req| {
            rs.push(Response::default(Some(&req)));
            if let Some(r) = req.init() {
                v.push(r);
            }
        });

        let mut futs = Vec::new();
        v.into_iter().for_each(|req| {
            let fut = Response::exec(req, &client);
            futs.push(fut);
        });
        let mut res = join_all(futs).await;
        for _ in 0..rs.len() {
            let mut r = rs.pop().unwrap();
            let d = res.pop().unwrap();
            match d {
                Ok(da) => {
                    r.content = da.0;
                    r.headers = da.1;
                    r.status = da.2;
                }
                Err(e) => {
                    r.msg = Some(e.desc);
                }
            }
        }
        result.lock().unwrap().extend(rs);
    }
}

impl Drop for Response {
    fn drop(&mut self) {
        let status = self.status;
        if status >= 300 {
            error!(
                "status: {}, url: {}, msg: {} <++>",
                self.status,
                self.uri,
                self.msg.to_owned().unwrap()
            );
            info!("body: {:?}, cookie: {:?} <++>", self.body, self.cookie);
            trace!(
                "method: {}, created: {}, parser: {}, args: {:?}",
                self.method,
                self.created,
                self.raw_parser,
                self.args
            );
        } else if status >= 200 {
            info!("status: {}, url: {} <++>", self.status, self.uri);
            debug!("body: {:?}, cookie: {:?} <++>", self.body, self.cookie);
            trace!(
                "method: {}, created: {}, parser: {}, args: {:?}",
                self.method,
                self.created,
                self.raw_parser,
                self.args
            );
        } else if status >= 100 {
            warn!("status: {}, url: {} <++>", self.status, self.uri);
            debug!("body: {:?}, cookie: {:?} <++>", self.body, self.cookie);
            trace!(
                "method: {}, created: {}, parser: {}, args: {:?}",
                self.method,
                self.created,
                self.raw_parser,
                self.args
            );
        } else {
            error!("status: {:?}, uri: {}, body: {:?}, cookie: {:?}, method: {}, created: {}, parser: {}, args: {:?}", self.status, self.uri, self.body, self.cookie, self.method, self.created, self.raw_parser, self.args );
        }
    }
}

impl Response {
    fn default(req: Option<&Request>) -> Self {
        match req {
            Some(r) => Response {
                uri: r.uri.clone(),
                method: r.method.clone(),
                cookie: r.cookie.clone().unwrap(),
                created: r.created.clone(),
                parser: get_parser(r.raw_parser.clone()),
                raw_parser: r.raw_parser.clone(),
                args: r.args.clone().unwrap(),
                msg: None,
                body: r.body.clone().unwrap(),

                content: None,
                headers: r.headers.clone().unwrap(),
                status: 0,
            },
            None => Response {
                headers: HashMap::new(),
                status: 0,
                content: None,

                body: HashMap::new(),
                uri: "".to_owned(),
                method: "".to_owned(),
                cookie: HashMap::new(),
                created: 0,
                parser: get_parser("".to_owned()),
                raw_parser: "".to_owned(),
                args: HashMap::new(),
                msg: None,
            },
        }
    }

    pub fn _into(&self) -> Option<(Task, Profile)> {
        match self.content {
            None => return None,
            Some(_) => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as u64;
                let task_index = self.args.get("task").unwrap();
                let mut pheaders = self.headers.clone();
                let mut theaders = self.headers.clone();
                pheaders.retain(|k, _| task_index.contains(&k));
                theaders.retain(|k, _| !task_index.contains(&k));
                let profile = Profile {
                    cookie: Some(self.cookie.clone()),
                    headers: Some(theaders),
                    able: now + 20,
                    created: self.created,
                };
                let task = Task {
                    uri: self.uri.clone(),
                    method: self.method.clone(),
                    body: Some(self.body.clone()),
                    headers: Some(pheaders),
                    able: now + 20,
                    raw_parser: self.raw_parser.clone(),
                    args: Some(self.args.clone()),
                    parser: get_parser(self.raw_parser.clone()),
                };
                debug!("convert a response to task and profile.");
                return Some((task, profile));
            }
        }
    }
}
