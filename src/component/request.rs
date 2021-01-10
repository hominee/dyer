extern crate bytes;
extern crate hyper;
extern crate hyper_tls;
extern crate serde;
extern crate serde_json;

use crate::component::{Profile, Task};
use crate::engine::Elements;
use hyper::header::{HeaderName, HeaderValue};
use hyper::{Body as hBody, Request as hRequest};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

type Sitem<U> = Result<U, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(
    bound = "TArgs: Serialize + for<'a> Deserialize<'a> + Debug + Clone, PArgs: Serialize + for<'a> Deserialize<'a> + Debug + Clone"
)]
pub struct Request<TArgs, PArgs> {
    pub uri: String,
    pub method: String,
    pub headers: Option<std::collections::HashMap<String, String>>,
    pub pheaders: std::collections::HashMap<String, String>,
    pub theaders: std::collections::HashMap<String, String>,
    pub cookie: Option<std::collections::HashMap<String, String>>,
    pub body: Option<std::collections::HashMap<String, String>>,
    pub able: u64,
    pub trys: u8,
    pub created: u64,
    pub parser: String,
    pub targs: Option<TArgs>,
    pub pargs: Option<PArgs>,
}
unsafe impl<TArgs, PArgs> Send for Request<TArgs, PArgs> {}

impl<T, P> Request<T, P>
where
    T: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + Clone,
    P: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + Clone,
{
    /// based on the length of both profiles and tasks
    /// to restrict the gen size of request
    /// the num should be provided
    pub fn gen<'a, E>(
        profile: Arc<Mutex<Vec<Profile<P>>>>,
        tasks: Arc<Mutex<Vec<Task<T>>>>,
        round: usize,
        f: Option<
            &'a (dyn Fn(Elements<'a, E, T, P>) -> Sitem<Elements<'a, E, T, P>> + Send + Sync),
        >,
    ) -> Vec<Request<T, P>>
    where
        E: Serialize + std::fmt::Debug + Clone,
    {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        //split them into two parts
        let mut ind = Vec::new();
        let mut ndy = Vec::new();
        let mut j = 0;

        let len_p = profile.lock().unwrap().len();
        let len_t = tasks.lock().unwrap().len();
        let len = len_t.min(len_p);
        for i in 0..round.min(len) {
            let p = &profile.lock().unwrap()[i];
            if p.able <= now {
                ind.push(i - j);
                j += 1;
            }
        }

        debug!("all {} request are going to created.", j);
        ind.into_iter().for_each(|index| {
            let p = profile.lock().unwrap().remove(index);
            let task = tasks.lock().unwrap().pop().unwrap();
            let result = if let Some(func) = f {
                let req = if let Ok(Elements::Req(request)) = func(Elements::Array(vec![
                    Elements::Pfile(p),
                    Elements::Tsk(task),
                ])) {
                    request
                } else {
                    panic!("must return request correctly.");
                };
                req
            } else {
                let mut req = Request::default();
                req.from_task(task);
                req.from_profile(p);
                req
            };
            debug!("generate 1 request: {:?}", result);
            if result.parser == "".to_string() {
                panic!("generate request failer missing parser : {:?}", result);
            }
            ndy.push(result);
        });
        ndy
    }
}

impl<T, P> Request<T, P>
where
    T: Serialize + for<'de> Deserialize<'de> + Debug + Clone,
    P: Serialize + for<'de> Deserialize<'de> + Debug + Clone,
{
    pub fn init<'a, E>(
        mut self,
        f: Option<
            &'a (dyn Fn(Elements<'a, E, T, P>) -> Sitem<Elements<'a, E, T, P>> + Send + Sync),
        >,
    ) -> Option<hRequest<hBody>>
    where
        E: Serialize + std::fmt::Debug + Clone,
    {
        if let Some(ff) = f {
            let request = match ff(Elements::Req(self.clone())) {
                Ok(req) => {
                    if let Elements::Req(req) = req {
                        req
                    } else {
                        self
                    }
                }
                Err(_) => {
                    error!("not use request_init or use it right.");
                    self
                }
            };
            self = request;
        };
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
                cookie = v.join("; ");

                headers.insert(
                    HeaderName::from_str("cookie").unwrap(),
                    HeaderValue::from_str(&cookie).unwrap(),
                );
            }
            None => {}
        }
        if let Some(head) = self.headers.to_owned() {
            head.iter().for_each(|(k, v)| {
                headers.insert(
                    HeaderName::from_str(k.as_str()).unwrap(),
                    HeaderValue::from_str(v.as_str()).unwrap(),
                );
            });
        }
        let thead = self.theaders.to_owned();
        thead.iter().for_each(|(k, v)| {
            headers.insert(
                HeaderName::from_str(k.as_str()).unwrap(),
                HeaderValue::from_str(v.as_str()).unwrap(),
            );
        });
        let phead = self.pheaders.to_owned();
        phead.iter().for_each(|(k, v)| {
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
                let data = Some(builds.body(hBody::from(s)).unwrap());
                log::trace!("request with body: {:?}", data);
                return data;
            }
            None => {
                let data = Some(builds.body(hBody::default()).unwrap());
                log::trace!("request without body: {:?}", data);
                return data;
            }
        }
    }
}

impl<T, P> Default for Request<T, P>
where
    T: Serialize + for<'a> Deserialize<'a> + Debug + Clone,
    P: Serialize + for<'a> Deserialize<'a> + Debug + Clone,
{
    fn default() -> Request<T, P> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut headers: HashMap<String, String> = HashMap::new();
        headers.insert(
            "accept".to_owned(),
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8".to_owned(),
        );
        headers.insert("accept-encoding".to_owned(), "gzip, deflate, br".to_owned());
        headers.insert("accept-language".to_owned(), "en-US,en;q=0.5".to_owned());
        headers.insert("cache-control".to_owned(), "no-cache".to_owned());
        headers.insert("connection".to_owned(), "keep-alive".to_owned());
        headers.insert("pragma".to_owned(), "no-cache".to_owned());
        headers.insert("upgrade-insecure-requests".to_owned(), "1".to_owned());
        headers.insert(
            "user-agent".to_owned(),
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:77.0) Gecko/20100101 Firefox/77.0"
                .to_owned(),
        );
        Request::<T, P> {
            uri: "".to_string(),
            method: "GET".to_owned(),
            parser: "".to_string(),
            headers: None,
            pheaders: headers,
            theaders: HashMap::new(),
            cookie: None,
            body: None,
            able: now,
            trys: 0,
            created: now,
            targs: None,
            pargs: None,
        }
    }
}

impl<T, P> Request<T, P>
where
    T: Serialize + for<'a> Deserialize<'a> + Debug + Clone,
    P: Serialize + for<'a> Deserialize<'a> + Debug + Clone,
{
    pub fn stored(path: &str, reqs: &mut Arc<Mutex<Vec<Request<T, P>>>>) {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .unwrap();
        let mut buf = Vec::new();
        reqs.lock().unwrap().iter().for_each(|req| {
            let s = serde_json::to_string(&req).unwrap();
            buf.push(s);
        });
        file.write(buf.join("\n").as_bytes()).unwrap();
    }

    pub fn load(path: &str) -> Option<Vec<Request<T, P>>> {
        // load Profile here
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .unwrap();
        let data = BufReader::new(&file)
            .lines()
            .map(|line| {
                let s = line.unwrap().to_string();
                let task: Request<T, P> = serde_json::from_str(&s).unwrap();
                task
            })
            .collect::<Vec<Request<T, P>>>();
        return Some(data);
    }
}

impl<T, P> Request<T, P>
where
    T: Serialize + for<'a> Deserialize<'a> + Debug + Clone,
    P: Serialize + for<'a> Deserialize<'a> + Debug + Clone,
{
    pub fn from_profile(&mut self, profile: Profile<P>) {
        if let Some(mut cookie) = self.cookie.to_owned() {
            cookie.extend(profile.cookie.unwrap());
        } else {
            self.cookie = profile.cookie;
        }
        if let Some(p) = profile.headers {
            self.pheaders = p;
        };
        if self.able < profile.able {
            self.able = profile.able;
        }
        self.created = profile.created;
        self.pargs = profile.pargs;
    }

    pub fn from_task(&mut self, task: Task<T>) {
        self.uri = task.uri;
        self.method = task.method;
        if let Some(t) = task.headers {
            self.theaders = t;
        };
        self.targs = task.targs;
        self.parser = task.parser;
        self.body = task.body;
        if self.able < task.able {
            self.able = task.able;
        }
    }

    pub fn into1(self) -> (Profile<P>, Task<T>) {
        let task = Task {
            uri: self.uri,
            method: self.method,
            headers: self.headers,
            targs: self.targs,
            parser: self.parser,
            body: self.body,
            able: self.able,
            trys: self.trys,
        };
        let profile = Profile {
            cookie: self.cookie,
            headers: Some(self.pheaders),
            able: self.able,
            created: self.created,
            pargs: self.pargs,
        };
        (profile, task)
    }
}
