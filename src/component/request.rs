extern crate bytes;
extern crate hyper;
extern crate hyper_tls;
extern crate serde;
extern crate serde_json;

use crate::component::{Profile, Task};
use hyper::header::{HeaderName, HeaderValue};
use hyper::{Body as hBody, Request as hRequest};
use log::debug;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::fs;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

/// Generally, `Profile` and `Task` roughly add up to a `Request`,  
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(
    bound = "T: Serialize + for<'a> Deserialize<'a> + Debug + Clone, P: Serialize + for<'a> Deserialize<'a> + Debug + Clone"
)]
pub struct Request<T, P>
where
    T: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
    P: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
{
    pub task: Task<T>,
    pub profile: Profile<P>,
    pub able: u64,
    pub headers: Option<std::collections::HashMap<String, String>>,
}
unsafe impl<T, P> Send for Request<T, P>
where
    T: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
    P: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
{
}

impl<T, P> Request<T, P>
where
    T: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + Clone,
    P: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + Clone,
{
    /// based on the length of both profiles and tasks
    /// to restrict the gen size of request
    /// the num should be provided
    pub fn gen(
        profile: Arc<Mutex<Vec<Profile<P>>>>,
        tasks: Arc<Mutex<Vec<Task<T>>>>,
        round: usize,
    ) -> Vec<Request<T, P>> {
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
            let task = tasks.lock().unwrap().remove(0);
            let mut req = Request::default();
            req.from_task(task);
            req.from_profile(p);
            debug!("generate 1 request: {:?}", req);
            if &req.task.parser == "" {
                panic!("generate request failer missing parser : {:?}", req);
            }
            ndy.push(req);
        });
        ndy
    }
}

impl<T, P> Request<T, P>
where
    T: Serialize + for<'de> Deserialize<'de> + Debug + Clone,
    P: Serialize + for<'de> Deserialize<'de> + Debug + Clone,
{
    /// transform a `Request` into `hyper::Request`
    pub fn init(self) -> Option<hRequest<hBody>> {
        let mut builder = hRequest::builder();
        // initialize headers
        let headers = builder.headers_mut().unwrap();
        let cookie: String;
        // initialize cookie
        let cookies = self.profile.cookie;
        let mut v: Vec<String> = Vec::new();
        cookies.iter().for_each(|(key, value)| {
            v.push(format!("{}={}", key, value));
        });
        cookie = v.join("; ");

        headers.insert(
            HeaderName::from_str("cookie").unwrap(),
            HeaderValue::from_str(&cookie).unwrap(),
        );
        if let Some(head) = self.headers.as_ref() {
            head.iter().for_each(|(k, v)| {
                headers.insert(
                    HeaderName::from_str(k.as_str()).unwrap(),
                    HeaderValue::from_str(v.as_str()).unwrap(),
                );
            });
        }
        let thead = self.task.headers;
        thead.iter().for_each(|(k, v)| {
            headers.insert(
                HeaderName::from_str(k.as_str()).unwrap(),
                HeaderValue::from_str(v.as_str()).unwrap(),
            );
        });
        let phead = self.profile.headers;
        phead.iter().for_each(|(k, v)| {
            headers.insert(
                HeaderName::from_str(k.as_str()).unwrap(),
                HeaderValue::from_str(v.as_str()).unwrap(),
            );
        });

        // set method and uri
        let builds = builder
            .uri(self.task.uri.as_str())
            .method(self.task.method.as_str());

        // consume this builder  and create Hyper::Request
        match self.task.body.as_ref() {
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
        Request::<T, P> {
            task: Task::default(),
            profile: Profile::default(),
            headers: None,
            able: now,
        }
    }
}

impl<T, P> Request<T, P>
where
    T: Serialize + for<'a> Deserialize<'a> + Debug + Clone,
    P: Serialize + for<'a> Deserialize<'a> + Debug + Clone,
{
    /// store unfinished or extra `Request`s,
    pub fn stored(path: &str, reqs: &Arc<Mutex<Vec<Request<T, P>>>>) {
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

    /// load unfinished or extra `Request`s,
    pub fn load(path: &str) -> Vec<Request<T, P>> {
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
        return data;
    }
}

impl<T, P> Request<T, P>
where
    T: Serialize + for<'a> Deserialize<'a> + Debug + Clone,
    P: Serialize + for<'a> Deserialize<'a> + Debug + Clone,
{
    /// construct `Request` from `Profile`
    pub fn from_profile(&mut self, profile: Profile<P>) {
        if self.able < profile.able {
            self.able = profile.able;
        }
        self.profile = profile;
    }

    /// construct `Request` from `Task`
    pub fn from_task(&mut self, task: Task<T>) {
        if self.able < task.able {
            self.able = task.able;
        }
        self.task = task;
    }
}
