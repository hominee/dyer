extern crate serde;
extern crate serde_json;

use crate::item::{Profile, Request, ResError, Response, Task};
use crate::middleware::{hand0, hand100, hand300, hand400, hand500, hand_res, process_item_name1};
use crate::spider::get_parser;
use crate::spider::{Entry, Parse, ParseError};
use futures::executor::block_on;
use futures::future::join_all;
use hyper::{client::HttpConnector, Client as hClient};
use hyper_tls::HttpsConnector;
use hyper_timeout::TimeoutConnector;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::task;

#[derive(Debug, Clone)]
pub struct App {
    pub urls: Vec<String>,
    pub parsers: Vec<String>,
}

impl App {
    /// manual specify the starting point
    pub fn init() -> App {
        //initialize it as a default
        let urls = vec![
            "https://weibo.cn".to_owned(),
            "https://weibo.com".to_owned(),
        ];
        let parsers = vec!["parse_index1".to_owned(), "parse_index2".to_owned()];
        App { urls, parsers }
    }

    /// the url that Profile make
    pub fn start_page(ind: &str) -> String {
        if ind == "weibo" {
            return "https://weibo.cn/pub".to_string();
        }else {
            panic!("wrong url to fake a Profile.");
        }
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

impl Entry for App {
    fn start_request(
        &self,
        client: hClient<TimeoutConnector< HttpsConnector<HttpConnector> >>,
        response: Arc<Mutex< Vec<Response> >>,
        profiles: Arc<Mutex< Vec<Profile>  >>
    ) {
        //the started url that spark the crawler
        let mut reqs: Vec<Request> = Vec::new();
        let mut urls = self.urls.to_owned();
        let mut parsers = self.parsers.to_owned();
        for _ in 0..urls.len() {
            // fake a profile
            let mut req: Request = Request::default();
            let url = urls.pop().unwrap();
            let parser = parsers.pop().unwrap();
            req.uri = url.to_owned();
            req.parser = parser;
            let profile = profiles.lock().unwrap().pop().unwrap();
            req.from_profile(profile); //FIXME what if some profile fails
            reqs.push(req);
        }

        //perform Request
        let res = block_on(Response::exec_all(reqs, client, response));
    }
}

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

impl Parse for App {
    fn parse(mut res: Response) -> Result<ParseResult, ParseError> {
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
            let t = res._into().unwrap();
            let mut r = ParseResult {
                req: None,
                task: Some(vec![t.0]),
                profile: Some(t.1),
                entities: None,
                yield_err: None,
            };
            let content = res.content.to_owned().unwrap();
            let data = (get_parser(&res.parser))(content.clone());
            match data {
                Ok(mut v) => {
                    process_item_name1(&mut v.0);
                    r.entities = Some(v.0);
                }
                Err(_e) => {
                    // no entities comes in.
                    // leave None as default.
                    let s = format!("{}\n{}\n{}", res.uri, res.parser, content);
                    r.yield_err = Some(s);
                }
            }
            return Ok(r);
        }
    }

    fn parse_all(vres: Arc<Mutex< Vec<Response> >>, vreq: Arc<Mutex<  Vec<Request> >>, vtask: Arc<Mutex< Vec<Task> >>, vpfile: Arc<Mutex< Vec<Profile> >>, entities: Arc<Mutex< Vec<Entity> >>, yield_err: Arc<Mutex< Vec<String> >>, round: usize  )  {
        let mut v = Vec::new();
        let len = vres.lock().unwrap().len();
        vec![0; len.min(round) ].iter().for_each(|_|{
            let t = vres.lock().unwrap().pop().unwrap();
            v.push(t);
        });
        v.into_iter().for_each(| res |{
            match App::parse(res) {

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
