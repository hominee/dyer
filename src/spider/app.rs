extern crate serde;
extern crate serde_json;

use crate::item::{Profile, Request, ResError, Response, Task};
use crate::middleware::{hand0, hand100, hand300, hand400, hand500, hand_res, process_item_name1};
use crate::spider::fake;
use crate::spider::{Entry, Parse, ParseError};
use futures::executor::block_on;
use futures::future::join_all;
use hyper::{client::HttpConnector, Client as hClient};
use hyper_tls::HttpsConnector;
use hyper_timeout::TimeoutConnector;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct App {
    pub urls: Vec<String>,
    pub raw_parsers: Vec<String>,
}

impl App {
    pub fn init() -> App {
        //initialize it as a default
        let urls = vec![
            "https://weibo.cn".to_owned(),
            "https://weibo.com".to_owned(),
        ];
        let raw_parsers = vec!["parse_index1".to_owned(), "parse_index2".to_owned()];
        App { urls, raw_parsers }
    }
}

impl Entry for App {
    fn start_request(
        &self,
        client: hClient<TimeoutConnector< HttpsConnector<HttpConnector> >>,
        response: Arc<Mutex< Vec<Response> >> 
    ) {
        //the started url that spark the crawler
        let mut reqs: Vec<Request> = Vec::new();
        let mut profiles: Arc<Mutex< Vec<_> >> = Arc::new( Mutex::new( Vec::new() ));
        let mut prof: Vec<_> = Vec::new();
        let mut urls = self.urls.to_owned();
        let mut raw_parsers = self.raw_parsers.to_owned();
        for _ in 0..urls.len() {
            // fake a profile
            let profiles = profiles.clone();
            prof.push( fake(&client, profiles) );
            let mut req: Request = Request::default();
            let url = urls.pop().unwrap();
            let raw_parser = raw_parsers.pop().unwrap();
            req.uri = url.to_owned();
            req.raw_parser = raw_parser;
            reqs.push(req);
        }
        block_on(join_all(prof));
        for i in 0..urls.len() {
            let profile = profiles.lock().unwrap().pop().unwrap();
            reqs[i].from_profile(profile); //FIXME what if some profile fails
        }

        //perform Request
        let res = block_on(Response::exec_all(reqs, client, response));
        /*
         *let mut resps = Vec::new();
         *match res {
         *    Ok(vc) => {
         *        vc.into_iter().for_each(|r| {
         *            resps.push(r);
         *        });
         *        return Ok(resps);
         *    }
         *    Err(e) => Err(e),
         *}
         */
    }
}

///all item prototypes intented to collected
#[derive(Debug, Serialize, Deserialize)]
pub enum Entity {}

pub struct PaerseResult {
    pub req: Option<Request>,
    pub task: Option<Vec<Task>>,
    pub profile: Option<Profile>,
    pub entities: Option<Vec<Entity>>,
    pub yield_err: Option<String>,
}

impl Parse for App {
    fn parse(mut res: Response) -> Result<PaerseResult, ParseError> {
        //dispath handlers dependent on their status code
        let status = res.status;
        if status >= 500usize {
            let r = hand500(res);
            match r {
                Some(r) => Ok(PaerseResult {
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
                Some(r) => Ok(PaerseResult {
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
                Some(r) => Ok(PaerseResult {
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
                Some(data) => Ok(PaerseResult {
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
                Some(r) => Ok(PaerseResult {
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
            let t = res._into().unwrap();
            let mut r = PaerseResult {
                req: None,
                task: Some(vec![t.0]),
                profile: Some(t.1),
                entities: None,
                yield_err: None,
            };
            let content = res.content.to_owned().unwrap();
            let data = (res.parser)(content.clone());
            match data {
                Ok(mut v) => {
                    process_item_name1(&mut v.0);
                    r.entities = Some(v.0);
                }
                Err(_e) => {
                    // no entities comes in.
                    // leave None as default.
                    let s = format!("{}\n{}\n{}", res.uri, res.raw_parser, content);
                    r.yield_err = Some(s);
                }
            }
            return Ok(r);
        }
    }
}

//the trait you must implemented
//you can impl them at different place
/*
 *impl Entry for App {
 *    fn start_request(&self, client:&hClient<HttpsConnector<HttpConnector>>) {  }
 *}
 *impl Parse for App {
 *    fn parse<T>(&self, body: String) -> Result<Vec<T>, ParseError> {
 *
 *    }
 *}
 *impl HandleErr for App {
 *    fn hand100(&self, res: Response) ->(Task, Profile) {
 *
 *    }
 *    fn hand300(&self, res: Response) ->(Task, Profile) {
 *
 *    }
 *    fn hand400(&self, res: Response) ->(Task, Profile) {
 *
 *    }
 *    fn hand500(&self, res: Response) ->(Task, Profile) {
 *
 *    }
 *}
 */
