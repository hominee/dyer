extern crate serde;
extern crate serde_json;

use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};

use crate::component::{utils, Client, Request, ResError, Response, Task, UserAgent};
use futures::future::join_all;
use log::{error, info};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::io::Write;
use std::sync::{Arc, Mutex};

//type Sitem<U> = Result<U, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(bound = "PArgs: Serialize + for<'a> Deserialize<'a> + Debug + Clone")]
pub struct Profile<PArgs>
where
    PArgs: Serialize + for<'a> Deserialize<'a> + Debug + Clone,
{
    pub headers: HashMap<String, String>,
    pub cookie: HashMap<String, String>,
    pub able: u64,
    pub created: u64,
    pub pargs: Option<PArgs>,
}
unsafe impl<P> Send for Profile<P> where P: Serialize + for<'a> Deserialize<'a> + Debug + Clone {}

/*
 *the structure buffer that customize your needs
 *#[derive(Debug, Clone, Deserialize, Serialize)]
 *pub struct PArgs {
 *    pub typ: ProfileType,
 *    pub inteval: Interval,
 *    pub expire: u64,
 *}
 *
 *#[derive(Debug, Clone, Deserialize, Serialize)]
 *pub enum Interval {
 *    Light,
 *    Middle,
 *    Night,
 *}
 *
 *#[derive(Debug, Clone, Deserialize, Serialize)]
 *pub enum ProfileType {
 *    Web,
 *    Mobile,
 *}
 */

impl<P> Profile<P>
where
    P: Serialize + for<'a> Deserialize<'a> + Debug + Clone,
{
    pub fn exec<T>(res: &mut Response<T, P>) -> Result<Profile<P>, ResError>
    where
        T: Serialize + for<'a> Deserialize<'a> + Debug + Clone,
    {
        let raw_headers = &res.headers;
        let mut cookie = HashMap::new();
        if let Some(data) = raw_headers.get("set-cookie") {
            cookie = utils::get_cookie(data);
        };
        res.profile.cookie = cookie;
        res.content = Some("".to_string());
        Ok(res.profile.clone())
    }

    pub async fn exec_all<'a, E, T>(
        //f: Option<&'a (dyn Fn(Response<T, P>) -> Sitem<ParseResult<E, T, P>> + Send + Sync)>,
        profiles: Arc<Mutex<Vec<Profile<P>>>>,
        uri: &str,
        num: usize,
        uas: Arc<Vec<UserAgent>>,
    ) where
        T: Serialize + for<'de> Deserialize<'de> + Debug + Clone,
        P: Serialize + for<'de> Deserialize<'de> + Debug + Clone,
        E: Serialize + std::fmt::Debug + Clone,
    {
        let mut rs = vec![];
        let mut vreq = Vec::new();
        vec![0; num].iter().for_each(|_| {
            // select a ua
            let len = uas.len();
            let ind = rand::thread_rng().gen_range(0, len - 1);
            let ua = uas[ind].clone().user_agent;
            // construct a new reqeust
            let mut req = Request::<T, P>::default();
            req.task.uri = uri.clone().to_string();
            req.profile.headers.insert("user-agent".to_string(), ua);
            let p = Response::default(Some(&req));
            rs.push(p);
            if let Some(t) = req.init() {
                log::trace!("Request that to generate Profile: {:?}", t);
                vreq.push(Client::exec(t, Some(false)));
            }
        });
        // poll all request concurrently
        let vres = join_all(vreq).await;
        let mut i = 0usize;
        vres.into_iter().for_each(|r| {
            let mut p = rs.pop().unwrap();
            match r {
                Ok(res) => {
                    p.headers.extend(res.1);
                    p.status = res.2;
                    let profile = Some(Profile::exec(&mut p).unwrap());
                    /*
                     *let profile = match f {
                     *    None => Some(),
                     *    Some(func) => {
                     *        if let Elements::Pfile(mut result) = func(Elements::Res(p)).unwrap() {
                     *            result.able += gap;
                     *            Some(result)
                     *        } else {
                     *            None
                     *        }
                     *    }
                     *};
                     */
                    log::trace!("gen profile: {:?}", profile);
                    profiles.lock().unwrap().extend(profile);
                    i += 1;
                }
                Err(_) => {}
            }
        });
        if i == 0 {
            error!("get {} Profiles out of {}", i, num);
        } else {
            info!("get {} Profiles out of {}", i, num);
        }
    }
}

impl<P> Profile<P>
where
    P: Serialize + for<'a> Deserialize<'a> + Debug + Clone,
{
    pub fn stored(path: &str, profiles: &Arc<Mutex<Vec<Profile<P>>>>) {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .unwrap();
        let mut buf = Vec::new();
        profiles.lock().unwrap().iter().for_each(|r| {
            let s = serde_json::to_string(&r).unwrap();
            buf.push(s);
        });
        file.write(buf.join("\n").as_bytes()).unwrap();
    }

    pub fn load(path: &str) -> Option<Vec<Profile<P>>> {
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
                let task: Profile<P> = serde_json::from_str(&s).unwrap();
                task
            })
            .collect::<Vec<Profile<P>>>();
        return Some(data);
    }

    /// recycle Profile from Request
    pub fn recycle<T>(path: &str) -> (Vec<Profile<P>>, Vec<Task<T>>)
    where
        T: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + Clone,
    {
        let mut tasks = Vec::new();
        let mut profiles = Vec::new();
        if let Some(reqs) = Request::<T, P>::load(path) {
            reqs.into_iter().for_each(|req| {
                tasks.push(req.task);
                profiles.push(req.profile);
            });
        }
        (profiles, tasks)
    }
}

impl<P> Default for Profile<P>
where
    P: Serialize + for<'a> Deserialize<'a> + Debug + Clone,
{
    fn default() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut headers: HashMap<String, String> = HashMap::new();
        headers.insert(
            "accept".to_string(),
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8"
                .to_string(),
        );
        headers.insert(
            "accept-encoding".to_string(),
            "gzip, deflate, br".to_string(),
        );
        headers.insert("accept-language".to_string(), "en-US,en;q=0.5".to_string());
        headers.insert("cache-control".to_string(), "no-cache".to_string());
        headers.insert("connection".to_string(), "keep-alive".to_string());
        headers.insert("pragma".to_string(), "no-cache".to_string());
        headers.insert("upgrade-insecure-requests".to_string(), "1".to_string());
        headers.insert(
            "user-agent".to_string(),
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:77.0) Gecko/20100101 Firefox/77.0"
                .to_string(),
        );
        Profile::<P> {
            headers: headers,
            cookie: HashMap::new(),
            able: now,
            created: now,
            pargs: None,
        }
    }
}
