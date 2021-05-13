extern crate serde;
extern crate serde_json;

use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};

use crate::component::{utils,ProfileError, Client, ResError, Response};
use crate::engine::ProfileInfo;
use futures::future::join_all;
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
    pub able: f64,
    /// meta data that the `Profile` is created
    pub created: f64,
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
    pub fn exec<T>(res: Response<T, P>) -> Result<Profile<P>, ResError>
    where
        T: Serialize + for<'a> Deserialize<'a> + Debug + Clone,
    {
        let mut profile = res.profile.unwrap_or(Profile::default());
        let raw_headers = &res.headers;
        let mut cookie = HashMap::new();
        if let Some(data) = raw_headers.get("set-cookie") {
            cookie = utils::get_cookie(data);
        };
        profile.cookie = cookie;
        //res.content = Some("".to_string());
        Ok(profile)
    }


    /// generate one `Profile` 
    pub async fn exec_one<'a, E, T>( f: ProfileInfo<'a, T, P>,) -> Result<Profile<P>, ProfileError> 
    where
        T: Serialize + for<'de> Deserialize<'de> + Debug + Clone + Send,
        P: Serialize + for<'de> Deserialize<'de> + Debug + Clone,
        E: Serialize + std::fmt::Debug + Clone,
    {
            // construct a new reqeust
            let mut p = Response::new(Some(f.req.as_ref().unwrap()));
            if let Some(t) = f.req.clone().unwrap().init() {
                log::trace!("Request that to generate Profile: {:?}", t);
                let result = Client::exec(t, Some(false)).await;
                match result {
                    Ok(res) => {
                        p.headers.extend(res.1);
                        p.status = res.2;
                        if f.parser.is_none() {
                            let profile = Profile::exec(p).unwrap();
                            Ok(profile)
                        } else {
                            match (f.parser.unwrap())(p).await {
                                Ok(profile) => {
                                    log::trace!("gen profile: {:?}", profile);
                                    Ok(profile)
                                },
                                Err(_) => {
                                    Err(ProfileError{desc: "the parser of ProfileInfo cannot extract profile from the response".into()})
                                }
                            }
                        }
                    }
                    Err(_) => { Err( ProfileError{desc: "the parser of ProfileInfo cannot extract profile from the response".into()} ) }
                }
            }else {
                 Err( ProfileError{desc: "req of ProfileInfo cannot init into a hyper::Request".into()} ) 
            }
    }

    /// generate multiple `Profile` and put them into `App`
    pub async fn exec_all<'a, E, T>(
        profiles: Arc<Mutex<Vec<Profile<P>>>>,
        num: usize,
        f: ProfileInfo<'a, T, P>,
    ) where
        T: Serialize + for<'de> Deserialize<'de> + Debug + Clone + Send,
        P: Serialize + for<'de> Deserialize<'de> + Debug + Clone,
        E: Serialize + std::fmt::Debug + Clone,
    {
        let mut rs = vec![];
        let mut vreq = Vec::new();
        for _ in 0..num {
            // construct a new reqeust
            let p = Response::new(Some(f.req.as_ref().unwrap()));
            rs.push(p);
            if let Some(t) = f.req.clone().unwrap().init() {
                log::trace!("Request that to generate Profile: {:?}", t);
                vreq.push(Client::exec(t, Some(false)));
            }
        }
        // poll all request concurrently
        let vres = join_all(vreq).await;
        let mut i = 0usize;
        let mut pfiles = Vec::new();
        for r in vres.into_iter() {
            let mut p = rs.pop().unwrap();
            match r {
                Ok(res) => {
                    p.headers.extend(res.1);
                    p.status = res.2;
                    if f.parser.is_none() {
                        let profile = Profile::exec(p).unwrap();
                        pfiles.push(profile);
                    } else {
                        match (f.parser.unwrap())(p).await {
                            Ok(profile) => pfiles.push(profile),
                            Err(_) => {}
                        }
                    };
                    log::trace!("gen profile: {:?}", pfiles);
                    i += 1;
                }
                Err(_) => {}
            }
        }
        if i == 0 {
            log::error!("get {} / {} Profiles", i, num);
        } else {
            profiles.lock().unwrap().extend(pfiles);
            log::info!("get {} / {} Profiles ", i, num);
        }
    }
}

impl<P> Profile<P>
where
    P: Serialize + for<'a> Deserialize<'a> + Debug + Clone,
{
    /// store unfinished or extra `Profile`s,
    pub fn stored(path: &str, profiles: &mut Arc<Mutex<Vec<Profile<P>>>>) {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .unwrap();
        let mut buf = Vec::new();
        while let Some(r) = profiles.lock().unwrap().pop() {
            let s = serde_json::to_string(&r).unwrap();
            buf.push(s);
        }
        file.write(buf.join("\n").as_bytes()).unwrap();
    }

    /// load unfinished or extra `Profile`s  
    pub fn load(path: &str) -> Vec<Profile<P>> {
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
        return data;
    }
}

impl<P> Default for Profile<P>
where
    P: Serialize + for<'a> Deserialize<'a> + Debug + Clone,
{
    fn default() -> Self {
        let now = utils::now();
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
