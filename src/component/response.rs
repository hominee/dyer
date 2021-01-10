extern crate bytes;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;

use crate::component::{ParseError, Profile, Request, Task};
use crate::engine::{App, Elements};
use crate::macros::{MethodIndex, MiddleWare, Spider};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;

pub struct ParseResult<E, T, P>
where
    T: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
    P: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
    E: Serialize + std::fmt::Debug + Clone,
{
    pub req: Option<Request<T, P>>,
    pub task: Option<Vec<Task<T>>>,
    pub profile: Option<Profile<P>>,
    pub entities: Option<Vec<E>>,
    pub yield_err: Option<String>,
    pub low_mode: bool,
}
unsafe impl<E, T, P> Sync for ParseResult<E, T, P>
where
    T: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
    P: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
    E: Serialize + std::fmt::Debug + Clone,
{
}
unsafe impl<E, T, P> Send for ParseResult<E, T, P>
where
    T: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
    P: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
    E: Serialize + std::fmt::Debug + Clone,
{
}

#[derive(Clone, Debug)]
pub struct Response<T, P>
where
    T: Debug + Clone,
    P: Debug + Clone,
{
    pub headers: HashMap<String, String>,
    pub pheaders: HashMap<String, String>,
    pub theaders: HashMap<String, String>,
    pub status: usize,
    pub trys: u8,
    pub content: Option<String>,
    pub body: HashMap<String, String>,
    pub uri: String,
    pub method: String,
    pub cookie: HashMap<String, String>,
    pub created: u64,
    pub parser: String,
    pub targs: Option<T>,
    pub msg: Option<String>,
    pub pargs: Option<P>,
}
unsafe impl<T, P> Sync for Response<T, P>
where
    T: Debug + Clone,
    P: Debug + Clone,
{
}
unsafe impl<T, P> Send for Response<T, P>
where
    T: Debug + Clone,
    P: Debug + Clone,
{
}

impl<T, P> Response<T, P>
where
    T: Serialize + for<'a> Deserialize<'a> + Debug + Clone,
    P: Serialize + for<'a> Deserialize<'a> + Debug + Clone,
{
    pub fn parse<E>(
        mut res: Response<T, P>,
        spd: &'static dyn Spider<E, T, P>,
        mware: &dyn MiddleWare<E, T, P>,
        gap: u64,
    ) -> Result<ParseResult<E, T, P>, ParseError>
    where
        E: Serialize + std::fmt::Debug + Clone,
    {
        //dispath handlers dependent on their status code
        let status = res.status;
        if status <= 299 && status >= 200 {
            debug!("successful Response: uri: {}", &res.uri[0..77]);
            // status code between 200 - 299
            mware.hand_res(&mut res);
            let (_, p): (Task<T>, Profile<P>) = res.into1(gap).unwrap();
            info!("recycle profile");
            let mut r = ParseResult {
                req: None,
                task: None,
                profile: Some(p),
                entities: None,
                yield_err: None,
                low_mode: false,
            };
            let content = res.content.clone().unwrap();
            let uri = res.uri.clone();
            let ind = (&res.parser).to_string();
            let parser = spd
                .get_parser(MethodIndex::String(ind))
                .expect(&format!("parser {} not found.", &res.parser));
            let data = (parser)(Elements::Res(res));
            match data {
                Ok(v) => match v {
                    Elements::PrsRst(prs) => {
                        if let Some(mut items) = prs.entities {
                            mware.hand_item(&mut items);
                            r.entities = Some(items);
                        }
                        r.yield_err = prs.yield_err;
                        r.task = prs.task;
                        r.req = prs.req;
                    }
                    _ => {
                        error!("in parsing Response encountering unexpected type,");
                        unreachable!();
                    }
                },
                Err(_) => {
                    // no entities comes in.
                    // leave None as default.
                    error!("cannot parse Response");
                    let s = format!("{}\n{}", uri, content);
                    r.yield_err = Some(s);
                }
            }
            return Ok(r);
        } else {
            log::error!("failed Response: {:?}", res);
            let r = mware.hand_err(res);
            match r {
                Some(r) => Ok(ParseResult {
                    task: r.0,
                    profile: r.1,
                    req: r.2,
                    entities: None,
                    yield_err: r.3,
                    low_mode: r.4,
                }),
                None => Err(ParseError {
                    desc: "Non-200s status code , not good".to_owned(),
                }),
            }
        }
    }

    pub fn parse_all<E>(
        apk: &mut App<E, T, P>,
        round: usize,
        spd: &'static dyn Spider<E, T, P>,
        mware: &dyn MiddleWare<E, T, P>,
        gap: u64,
    ) where
        E: Serialize + std::fmt::Debug + Clone,
    {
        let mut v = Vec::new();
        let len = apk.res.lock().unwrap().len();
        vec![0; len.min(round)].iter().for_each(|_| {
            let t = apk.res.lock().unwrap().pop().unwrap();
            v.push(t);
        });
        v.into_iter().for_each(|res| {
            match Response::parse(res, spd, mware, gap) {
                Ok(d) => {
                    if let Some(da) = d.profile {
                        apk.profile.lock().unwrap().push(da);
                    }
                    if let Some(ta) = d.task {
                        apk.task_tmp.lock().unwrap().extend(ta);
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
                    let mut rate = &mut apk.rt_args.lock().unwrap().rate;
                    if rate.active == true
                        && d.low_mode == true
                        && rate.uptime >= 0.5 * rate.interval
                    {
                        rate.active = false;
                        rate.uptime = 0.0;
                    }
                }
                Err(_) => {
                    // res has err code (non-200) and cannot handled by error handle
                    // discard the response that without task or profile.
                    log::error!("parse response failed.");
                }
            }
        });
    }
}

/*
 *impl<T, P> Drop for Response<T, P>
 *where
 *    T: Debug + Clone,
 *    P: Debug + Clone,
 *{
 *    fn drop(&mut self) {
 *        let status = self.status;
 *        if status >= 300 {
 *            error!(
 *                "status: {}, url: {}, msg: {} body: {:?}, cookie: {:?}",
 *                self.status,
 *                self.uri,
 *                self.msg.to_owned().unwrap(),
 *                self.body,
 *                self.cookie,
 *            );
 *        } else if status >= 200 {
 *            info!("status: {}, url: {}", self.status, self.uri);
 *        } else if status >= 100 {
 *            warn!("status: {}, url: {}", self.status, self.uri);
 *        } else {
 *            error!("status: {:?}, uri: {}, body: {:?}, cookie: {:?}, method: {}, created: {}, args: {:?}", self.status, self.uri, self.body, self.cookie, self.method, self.created, self.targs );
 *        }
 *    }
 *}
 */

impl<T, P> Response<T, P>
where
    T: Serialize + for<'a> Deserialize<'a> + Debug + Clone,
    P: Serialize + for<'a> Deserialize<'a> + Debug + Clone,
{
    pub fn default(req: Option<&Request<T, P>>) -> Response<T, P> {
        let r = match req {
            Some(r) => r.clone(),
            None => Request::default(),
        };
        let cookie = match r.cookie {
            Some(c) => c,
            None => std::collections::HashMap::new(),
        };
        let body = match r.body {
            Some(c) => c,
            None => std::collections::HashMap::new(),
        };
        let headers = match r.headers {
            Some(c) => c,
            None => std::collections::HashMap::new(),
        };
        Response {
            uri: r.uri,
            method: r.method,
            cookie: cookie,
            created: r.created,
            parser: r.parser,
            targs: r.targs,
            msg: None,
            body: body,
            content: None,
            headers: headers,
            pheaders: r.pheaders,
            theaders: r.theaders,
            status: 0,
            trys: 0,
            pargs: r.pargs,
        }
    }

    pub fn into1(&self, gap: u64) -> Option<(Task<T>, Profile<P>)>
    where
        T: Serialize + for<'a> Deserialize<'a> + Debug + Clone,
        P: Serialize + for<'a> Deserialize<'a> + Debug + Clone,
    {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let pheaders = self.pheaders.clone();
        let theaders = self.theaders.clone();
        let profile = Profile {
            cookie: Some(self.cookie.clone()),
            headers: Some(pheaders),
            able: now + gap,
            created: self.created,
            pargs: self.pargs.clone(),
        };
        let task = Task {
            uri: self.uri.clone(),
            method: self.method.clone(),
            body: Some(self.body.clone()),
            headers: Some(theaders),
            able: now + gap,
            trys: self.trys,
            parser: self.parser.clone(),
            targs: self.targs.clone(),
        };
        debug!("convert a response to task and profile.");
        return Some((task, profile));
    }
}
