extern crate serde;
extern crate serde_json;

use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};

use crate::component::{utils, Client, Request, ResError, Response};
use futures::future::join_all;
use futures::future::BoxFuture;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::io::Write;
use std::sync::{Arc, Mutex};

/// Infomation represented to the server, generally, it provide extra, meta data about users and users' devices, required by server, basically, includes `User-Agent`, `Accept-Encoding` and so on. For the purposes of extension, customized generic parameter `P` is required.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(bound = "P: Serialize + for<'a> Deserialize<'a> + Debug + Clone")]
pub struct Profile<P>
where
    P: Serialize + for<'a> Deserialize<'a> + Debug + Clone,
{
    /// represents a heahers
    pub headers: HashMap<String, String>,
    /// cookie set by server or user
    pub cookie: HashMap<String, String>,
    /// checkpoint by which this `Profile` is valid for `Request`
    pub able: u64,
    /// meta data that the `Profile` is created
    pub created: u64,
    /// additional arguments for extensive application
    pub pargs: Option<P>,
}
unsafe impl<P> Send for Profile<P> where P: Serialize + for<'a> Deserialize<'a> + Debug + Clone {}

impl<P> Profile<P>
where
    P: Serialize + for<'a> Deserialize<'a> + Debug + Clone,
{
    /// default method used to extract `Profile` from a `Response`, collecting all cookie by
    /// `set-cookie` and ignoring others.
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

    /// generate multiple `Profile` and put them into `App`
    /// for different uri, different generator functions are required.
    pub async fn exec_all<'a, E, T>(
        profiles: Arc<Mutex<Vec<Profile<P>>>>,
        num: usize,
        f: (
            Request<T, P>,
            Option<
                &(dyn Fn(Response<T, P>) -> BoxFuture<'a, Result<Profile<P>, ResError>>
                      + Send
                      + Sync),
            >,
        ),
    ) where
        T: Serialize + for<'de> Deserialize<'de> + Debug + Clone,
        P: Serialize + for<'de> Deserialize<'de> + Debug + Clone,
        E: Serialize + std::fmt::Debug + Clone,
    {
        let mut rs = vec![];
        let mut vreq = Vec::new();
        for _ in 0..num {
            // construct a new reqeust
            let p = Response::default(Some(&f.0));
            rs.push(p);
            if let Some(t) = f.0.clone().init() {
                log::trace!("Request that to generate Profile: {:?}", t);
                vreq.push(Client::exec(t, Some(false)));
            }
        }
        // poll all request concurrently
        let vres = join_all(vreq).await;
        let mut i = 0usize;
        for r in vres.into_iter() {
            let mut p = rs.pop().unwrap();
            match r {
                Ok(res) => {
                    p.headers.extend(res.1);
                    p.status = res.2;
                    let mut pfiles = Vec::new();
                    if f.1.is_none() {
                        let profile = Profile::exec(&mut p).unwrap();
                        pfiles.push(profile);
                    } else {
                        match (f.1.unwrap())(p).await {
                            Ok(p) => pfiles.push(p),
                            Err(_) => {}
                        }
                    };
                    log::trace!("gen profile: {:?}", pfiles);
                    profiles.lock().unwrap().extend(pfiles);
                    i += 1;
                }
                Err(_) => {}
            }
        }
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
    /// store unfinished or extra `Profile`s,
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

    /// load unfinished or extra `Profile`s  
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
