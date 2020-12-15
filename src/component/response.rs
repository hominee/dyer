extern crate bytes;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;

use crate::component::{PArgs, ParseError, Profile, Request, TArgs, Task};
use crate::engine::App;
use crate::macros::MiddleWare;
use crate::macros::Spider;
use log::{debug, error, info, trace, warn};
use std::collections::HashMap;

pub struct ParseResult<T> {
    pub req: Option<Request>,
    pub task: Option<Vec<Task>>,
    pub profile: Option<Profile>,
    pub entities: Option<Vec<T>>,
    pub yield_err: Option<String>,
}
unsafe impl<T> Sync for ParseResult<T> {}
unsafe impl<T> Send for ParseResult<T> {}

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
    pub targs: Option<TArgs>,
    pub msg: Option<String>,
    pub pargs: Option<PArgs>,
}
unsafe impl Sync for Response {}
unsafe impl Send for Response {}

impl Response {
    pub fn parse<T>(
        mut res: Response,
        spd: &'static dyn Spider<T>,
        mware: &dyn MiddleWare<T>,
    ) -> Result<ParseResult<T>, ParseError> {
        //dispath handlers dependent on their status code
        let status = res.status;
        if status <= 299 && status >= 200 {
            // status code between 200 - 299
            mware.hand_res(&mut res);
            let (_, p): (Task, Profile) = res.into1().unwrap();
            let mut r = ParseResult {
                req: None,
                task: None,
                profile: Some(p),
                entities: None,
                yield_err: None,
            };
            let ind = &res.parser;
            let parser = spd.get_parser(ind).unwrap();
            let data = (parser)(&res);
            match data {
                Ok(v) => match v.entities {
                    Some(mut en) => {
                        mware.hand_item(&mut en);
                        r.entities = Some(en);
                    }
                    None => {}
                },
                Err(_) => {
                    // no entities comes in.
                    // leave None as default.
                    let content = res.content.clone().unwrap();
                    let s = format!("{}\n{}", &res.uri, content);
                    r.yield_err = Some(s);
                }
            }
            return Ok(r);
        } else {
            let r = mware.hand_err(res);
            match r {
                Some(r) => Ok(ParseResult {
                    task: r.0,
                    profile: r.1,
                    req: r.2,
                    entities: None,
                    yield_err: None,
                }),
                None => Err(ParseError {
                    desc: "Non-200s status code , not good".to_owned(),
                }),
            }
        }
    }

    pub fn parse_all<T>(
        apk: &mut App<T>,
        round: usize,
        spd: &'static dyn Spider<T>,
        mware: &dyn MiddleWare<T>,
    ) {
        let mut v = Vec::new();
        let len = apk.res.lock().unwrap().len();
        vec![0; len.min(round)].iter().for_each(|_| {
            let t = apk.res.lock().unwrap().pop().unwrap();
            v.push(t);
        });
        v.into_iter().for_each(|res| {
            match Response::parse(res, spd, mware) {
                Ok(d) => {
                    if let Some(da) = d.profile {
                        apk.profile.lock().unwrap().push(da);
                    }
                    if let Some(ta) = d.task {
                        apk.task.lock().unwrap().extend(ta);
                    }
                    if let Some(re) = d.req {
                        apk.req.lock().unwrap().push(re);
                    }
                    if let Some(err) = d.yield_err {
                        apk.yield_err.lock().unwrap().push(err);
                    }
                    if let Some(en) = d.entities {
                        // pipeline out put the entities
                        apk.result.lock().unwrap().extend(en.into_iter());
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
                "method: {}, created: {}, args: {:?}",
                self.method,
                self.created,
                self.targs
            );
        } else if status >= 200 {
            info!("status: {}, url: {} <++>", self.status, self.uri);
            debug!("body: {:?}, cookie: {:?} <++>", self.body, self.cookie);
            trace!(
                "method: {}, created: {},args: {:?}",
                self.method,
                self.created,
                self.targs
            );
        } else if status >= 100 {
            warn!("status: {}, url: {} <++>", self.status, self.uri);
            debug!("body: {:?}, cookie: {:?} <++>", self.body, self.cookie);
            trace!(
                "method: {}, created: {}, args: {:?}",
                self.method,
                self.created,
                self.targs
            );
        } else {
            error!("status: {:?}, uri: {}, body: {:?}, cookie: {:?}, method: {}, created: {}, args: {:?}", self.status, self.uri, self.body, self.cookie, self.method, self.created, self.targs );
        }
    }
}

impl Response {
    pub fn default(req: Option<&Request>) -> Self {
        if let Some(r) = req {
             Response {
                uri: r.uri.clone(),
                method: r.method.clone(),
                cookie: r.cookie.clone().unwrap(),
                created: r.created.clone(),
                parser: "parse".to_owned(),
                targs: r.targs.clone(),
                msg: None,
                body: r.body.clone().unwrap(),
                content: None,
                headers: r.headers.clone().unwrap(),
                pheaders: r.pheaders.clone(),
                theaders: r.theaders.clone(),
                status: 0,
                pargs: r.pargs.clone(),
            }
        } else {
            let r = Request::default();
             Response {
                uri: r.uri,
                method: r.method,
                cookie: r.cookie.unwrap(),
                created: r.created,
                parser: "parse".to_owned(),
                targs: r.targs,
                msg: None,
                body: r.body.unwrap(),
                content: None,
                headers: r.headers.unwrap(),
                pheaders: r.pheaders,
                theaders: r.theaders,
                status: 0,
                pargs: r.pargs,
            }
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
                    headers: Some(pheaders),
                    able: now + 20,
                    created: self.created,
                    pargs: self.pargs.clone(),
                };
                let task = Task {
                    uri: self.uri.clone(),
                    method: self.method.clone(),
                    body: Some(self.body.clone()),
                    headers: Some(theaders),
                    able: now + 20,
                    parser: self.parser.clone(),
                    targs: self.targs.clone(),
                };
                debug!("convert a response to task and profile.");
                return Some((task, profile));
            }
        }
    }
}
