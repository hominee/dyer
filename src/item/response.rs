extern crate bytes;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;

use crate::item::{Profile, PArgs, ParseError, Parser, Request, ResError, Task, TArgs};
use crate::spider::S as Sapp;
use crate::middleware::{hand0, hand100, hand300, hand400, hand500, hand_res, process_item_name1};
use crate::spider::{parse::get_parser };
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
use tokio::task;
use futures::executor::block_on;
use serde::{Serialize, Deserialize};

///all item prototypes intented to collected
#[derive(Debug, Serialize, Deserialize)]
pub enum Entity {}

pub struct ParseResult {
    pub req: Option<Request>,
    pub task: Option<Vec<Task>>,
    pub profile: Option<Profile>,
    pub entities: Option<Vec<Entity>>,
    pub yield_err: Option<String>,
}
unsafe impl Sync for ParseResult {}
unsafe impl Send for ParseResult{}

///the trait that parse the response
pub trait Parse {

    fn parse(body: Response, app: &'static Sapp ) -> Result<ParseResult, ParseError>;
    fn parse_all(vres: Arc<Mutex< Vec<Response> >>, vreq: Arc<Mutex<  Vec<Request> >>, vtask: Arc<Mutex< Vec<Task> >>, vpfile: Arc<Mutex< Vec<Profile> >>, entities: Arc<Mutex< Vec<Entity> >>, yield_err: Arc<Mutex< Vec<String> >>, round: usize, app: &'static Sapp);

}

pub struct Response {
    pub headers: HashMap<String, String>,
    pub pheaders: HashMap<String, String>,
    pub theaders: HashMap<String, String>,
    pub status: usize,
    pub content: Option<String>,

    pub body: HashMap<String, String>,
    pub uri: String,
    pub method: String,
    pub cookie: HashMap<String, String>,
    pub created: u64,
    pub parser: String, 
    pub fparser: Parser,
    pub targs: Option<TArgs>,
    pub msg: Option<String>,

    pub pargs: Option<PArgs>,
}
unsafe impl Sync for Response {}
unsafe impl Send for Response{}

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

    ///join spawned tokio-task
    pub fn join(
        res: Arc<Mutex<Vec< (u64, task::JoinHandle<()>) >>>,
        pfile: Arc<Mutex<Vec< (u64, task::JoinHandle<()>) >>>
    ) {
        let mut ind_r: Vec<usize> = Vec::new();
        let mut handle_r = Vec::new();
        let mut j = 0;
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as u64;
        res.lock().unwrap().iter().enumerate().for_each(|(ind, r)|{
            if now - r.0 >= 30 {
                ind_r.push(ind-j);
                j += 1;
            }
        });
        ind_r.into_iter().for_each(|ind|{
            let (_, handle) = res.lock().unwrap().remove(ind);
            handle_r.push(handle)
        });

        let mut ind_p: Vec<usize> = Vec::new();
        let mut j = 0;
        pfile.lock().unwrap().iter().enumerate().for_each(|(ind, r)|{
            if now - r.0 >= 30 {
                ind_p.push(ind-j);
                j += 1;
            }
        });
        ind_p.into_iter().for_each(|ind|{
            let (_, handle) = pfile.lock().unwrap().remove(ind);
            handle_r.push(handle)
        });
        block_on( join_all(handle_r) );
    }
}


impl Parse for Response {
    fn parse(mut res: Response, app: &'static Sapp) -> Result<ParseResult, ParseError> {
        //dispath handlers dependent on their status code
        let status = res.status;
        if status >= 500usize {
            let r = hand500(res);
            match r {
                Some(r) => Ok( ParseResult{
                    req: None,
                    task: Some(vec![r.0]),
                    profile: Some(r.1),
                    entities: None,
                    yield_err: None,
                }),
                None => Err(ParseError {
                    desc: "status 500 - 599, not good".to_owned(),
                }),
            }
        } else if status >= 400usize {
            let r = hand400(res);
            match r {
                Some(r) => Ok(ParseResult {
                    req: None,
                    task: Some(vec![r.0]),
                    profile: Some(r.1),
                    entities: None,
                    yield_err: None,
                }),
                None => Err(ParseError {
                    desc: "status 400 - 499, not good".to_owned(),
                }),
            }
        } else if status >= 300usize {
            let r = hand300(res);
            match r {
                Some(r) => Ok(ParseResult {
                    req: None,
                    task: Some(vec![r.0]),
                    profile: Some(r.1),
                    entities: None,
                    yield_err: None,
                }),
                None => Err(ParseError {
                    desc: "status  300 - 399 not good".to_owned(),
                }),
            }
        } else if status == 0usize {
            // only initialized and not modified
            // corroputed response caused this
            // recycle the Task and increase the error counter in Profile
            let r = hand0(res);
            match r {
                None => Err(ParseError {
                    desc: "status within 0, not good".to_owned(),
                }),
                Some(data) => Ok(ParseResult {
                    req: None,
                    task: Some(vec![data.0]),
                    profile: Some(data.1),
                    entities: None,
                    yield_err: None,
                }),
            }
        } else if status < 200usize {
            let r = hand100(res);
            match r {
                Some(r) => Ok(ParseResult {
                    req: Some(r),
                    task: None,
                    profile: None,
                    entities: None,
                    yield_err: None,
                }),
                None => Err(ParseError {
                    desc: " status within 100 - 199, not good".to_owned(),
                }),
            }
        } else {
            // status code between 200 - 299
            hand_res::pre_hand_res(&mut res);
            let (_, p) : (Task, Profile)= res.into1().unwrap();
            let mut r = ParseResult {
                req: None,
                task: None,
                profile: Some(p),
                entities: None,
                yield_err: None,
            };
            //let content = res.content.to_owned().unwrap();
            let data = (&res.fparser.data)(app, &res);
            match data {
                Ok(v) => {
                    match v.entities {
                        Some(mut en) => {
                            process_item_name1(&mut en);
                            r.entities = Some(en);
                        }
                        None => {}
                    }
                }
                Err(_) => {
                    // no entities comes in.
                    // leave None as default.
                    let content = res.content.clone().unwrap();
                    let s = format!("{}\n{}\n{}", &res.uri, &res.parser, content);
                    r.yield_err = Some(s);
                }
            }
            return Ok(r);
        }
    }

    fn parse_all(vres: Arc<Mutex< Vec<Response> >>, vreq: Arc<Mutex<  Vec<Request> >>, vtask: Arc<Mutex< Vec<Task> >>, vpfile: Arc<Mutex< Vec<Profile> >>, entities: Arc<Mutex< Vec<Entity> >>, yield_err: Arc<Mutex< Vec<String> >>, round: usize , app: &'static Sapp)  {
        let mut v = Vec::new();
        let len = vres.lock().unwrap().len();
        vec![0; len.min(round) ].iter().for_each(|_|{
            let t = vres.lock().unwrap().pop().unwrap();
            v.push(t);
        });
        v.into_iter().for_each(| res |{
            match Response::parse(res, app) {

               Ok(d) => {
                   if let Some(da) = d.profile {
                       vpfile.lock().unwrap().push(da);
                   }
                   if let Some(ta) = d.task {
                       vtask.lock().unwrap().extend(ta);
                   }
                   if let Some(re) = d.req {
                       vreq.lock().unwrap().push(re);
                   }
                   if let Some(err) = d.yield_err {
                       yield_err.lock().unwrap().push(err);
                   }
                   if let Some(en) = d.entities {
                       // pipeline out put the entities
                       entities.lock().unwrap().extend(en.into_iter());
                   }
               }
               Err(_e) => {
                           // res has err code (non-200) and cannot handled by error handle
                           // discard the response that without task or profile.
               }
            }
        });
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
                self.parser,
                self.targs
            );
        } else if status >= 200 {
            info!("status: {}, url: {} <++>", self.status, self.uri);
            debug!("body: {:?}, cookie: {:?} <++>", self.body, self.cookie);
            trace!(
                "method: {}, created: {}, parser: {}, args: {:?}",
                self.method,
                self.created,
                self.parser,
                self.targs
            );
        } else if status >= 100 {
            warn!("status: {}, url: {} <++>", self.status, self.uri);
            debug!("body: {:?}, cookie: {:?} <++>", self.body, self.cookie);
            trace!(
                "method: {}, created: {}, parser: {}, args: {:?}",
                self.method,
                self.created,
                self.parser,
                self.targs
            );
        } else {
            error!("status: {:?}, uri: {}, body: {:?}, cookie: {:?}, method: {}, created: {}, parser: {}, args: {:?}", self.status, self.uri, self.body, self.cookie, self.method, self.created, self.parser, self.targs );
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
                parser: r.parser.clone(),
                fparser: Parser::default(),
                targs: r.targs.clone(),
                msg: None,
                body: r.body.clone().unwrap(),

                content: None,
                headers: r.headers.clone().unwrap(),
                pheaders: r.pheaders.clone(),
                theaders: r.theaders.clone(),
                status: 0,

                pargs: r.pargs.clone(),
            },
            None => Response {
                headers: HashMap::new(),
                pheaders: HashMap::new(),
                theaders: HashMap::new(),
                status: 0,
                content: None,

                body: HashMap::new(),
                uri: "".to_owned(),
                method: "".to_owned(),
                cookie: HashMap::new(),
                created: 0,
                parser: "".to_owned(),
                fparser: Parser::default(),
                targs: None,
                msg: None,
                pargs: None,

            },
        }
    }

    pub fn into1(&self) -> Option<(Task, Profile)> {
        match self.content {
            None => return None,
            Some(_) => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as u64;
                let pheaders = self.pheaders.clone();
                let theaders = self.theaders.clone();
                let profile = Profile {
                    cookie: Some(self.cookie.clone()),
                    headers: Some(theaders),
                    able: now + 20,
                    created: self.created,
                    pargs: self.pargs.clone(),
                };
                let task = Task {
                    uri: self.uri.clone(),
                    method: self.method.clone(),
                    body: Some(self.body.clone()),
                    headers: Some(pheaders),
                    able: now + 20,
                    parser: self.parser.clone(),
                    fparser: Parser::get(&self.parser),
                    targs: self.targs.clone(),
                };
                debug!("convert a response to task and profile.");
                return Some((task, profile));
            }
        }
    }
}
