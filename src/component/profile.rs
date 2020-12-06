extern crate config;
extern crate serde;
extern crate serde_json;

use config::Config;

use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, ErrorKind};

use crate::component::{Request, Client, ResError, UserAgent};
use futures::future::join_all;
use hyper::{ Body as hBody, Request as hRequest};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::io::LineWriter;
use std::sync::{Arc, Mutex};
use futures::executor::block_on;
use rand::Rng;

#[derive(Debug, Deserialize, Serialize)]
pub struct Profile {
    pub headers: Option<HashMap<String, String>>,
    pub cookie: Option<HashMap<String, String>>,
    pub able: u64,
    pub created: u64,
    pub pargs: Option<PArgs>,
}

///the structure buffer that customize your needs
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PArgs {
    pub typ: ProfileType,
    pub inteval: Interval,
    pub expire: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Interval {
    Light,
    Middle,
    Night,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ProfileType {
    Web,
    Mobile,
}

impl Profile {
    pub async fn exec(
        req: hRequest<hBody>,
    ) -> Result<Profile, ResError> {
        let client = &Client::new(7, 23,7)[0];
        let mut p = Profile::default();
        let mut hd = p.headers.unwrap();
        let ua = req.headers().get("User-Agent").unwrap().clone().to_str().unwrap().to_string();
        hd.insert("User-Agent".to_string(), ua );
        let r = client.request(req).await;
        match r {
            Ok(res) => {
                let (bd, _) = res.into_parts();
                let raw_headers = bd.headers;

                let stop_word = ["path",  "expires", "domain", "httpOnly"];
                let mut cookie = HashMap::new();
                raw_headers.into_iter().for_each(|(k, v)| {
                    let key = k.unwrap().to_string().trim().to_lowercase();
                    if key == "set-cookie".to_string() {
                        let val = v.to_str().unwrap();
                        let v_str: Vec<&str> = val.split(";").filter(|c| !stop_word.contains( &c.trim() )).collect();
                        v_str.into_iter().for_each(|pair|{
                            let tmp: Vec<&str> = pair.split("=").collect();
                            if tmp.len() == 2 {
                                cookie.insert(tmp[0].to_string(), tmp[1].to_string() );
                            }
                        });
                    }
                });
                p.cookie = Some(cookie);
                p.headers = Some( hd );
                Ok(p)
            }
            Err(e) => {
                return Err(ResError {
                    desc: e.into_cause().unwrap().source().unwrap().to_string(),
                });
            }
        }
    }

    pub async fn exec_all(
        profiles: Arc<Mutex<Vec<Profile>>>,
        uri: &str,
        num: usize,
        uas: Arc< Vec<UserAgent> >,
    ) {
        let mut vreq = Vec::new();
        vec![0; num].iter().for_each(|_| {
            // select a ua
            let len = uas.len();
            let ind = rand::thread_rng().gen_range(0, len-1);
            let ua = uas[ind].clone().user_agent;
            // construct a new reqeust
            let mut req = Request::default();
            req.uri = uri.clone().to_string();
            let mut hd = req.headers.unwrap();
            hd.insert("User-Agent".to_string(), ua);
            req.headers = Some(hd);
            if let Some(t) = req.init() {
                vreq.push(Profile::exec(t));
            }
        });
        // poll all request concurrently
        let vres = block_on(  join_all(vreq) );
        let mut i = 0usize;
        vres.into_iter().for_each(|res| {
            if let Ok(p) = res {
                profiles.lock().unwrap().push(p);
                i += 1;
            }
        });
        if i == 0 {
            error!("get {} Profiles out of {}", i, num);
        } else {
            info!("get {} Profiles out of {}", i, num);
        }
    }
}

impl Profile {
    pub fn stored(profiles: &Arc<Mutex<Vec<Profile>>>) {
        let mut setting = Config::default();
        setting.merge(config::File::with_name("setting")).unwrap();
        let path = setting.get_str("path_profile").unwrap() + "/profile.txt";
        let file = fs::File::open(path).unwrap();
        let mut writer = LineWriter::new(file);
        profiles.lock().unwrap().iter().for_each(|r| {
            serde_json::to_writer(&mut writer, &r).unwrap();
        });
    }

    pub fn load() -> Option<Vec<Profile>> {
        let mut setting = Config::default();
        setting
            // load from file
            .merge(config::File::with_name("setting"))
            .unwrap();
        // load from PATH
        //.merge(config::Environment::with_prefix("APP")).unwrap();
        match setting.get_str("path_profile") {
            Ok(path) => {
                // load Profile here
                let file = fs::File::open(path.clone() + "profile.txt");
                match file {
                    Err(e) => match e.kind() {
                        ErrorKind::NotFound => {
                            fs::File::create(path.clone() + "/profile.txt").unwrap();
                            fs::File::create(path + "/profile_old.txt").unwrap();
                            return None;
                        }
                        _ => unreachable!(),
                    },
                    Ok(content) => {
                        let buf = BufReader::new(content).lines();
                        let mut data: Vec<Profile> = Vec::new();
                        buf.into_iter().for_each(|line| {
                            let profile: Profile = serde_json::from_str(&line.unwrap()).unwrap();
                            data.push(profile);
                        });
                        fs::remove_file(path.clone() + "/profile.txt").unwrap();
                        fs::rename(path.clone() + "/profile.txt", path + "/profile_old.txt")
                            .unwrap();
                        return Some(data);
                    }
                }
            }
            Err(_) => {
                // file not found
                panic!("path_profile is not configrated in setting.rs");
            }
        }
    }
}

impl Default for Profile {
    fn default() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut headers: HashMap<String, String> = HashMap::new();
        headers.insert(
            "Accept".to_string(),
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8".to_string(),
        );
        headers.insert("Accept-Encoding".to_string(), "gzip, deflate, br".to_string());
        headers.insert("Accept-Language".to_string(), "en-US,en;q=0.5".to_string());
        headers.insert("Cache-Control".to_string(), "no-cache".to_string());
        headers.insert("Connection".to_string(), "keep-alive".to_string());
        headers.insert("Pragma".to_string(), "no-cache".to_string());
        headers.insert("Upgrade-Insecure-Requests".to_string(), "1".to_string());
        headers.insert(
            "User-Agent".to_string(),
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:77.0) Gecko/20100101 Firefox/77.0"
                .to_string(),
        );
        Profile {
            headers: Some(headers),
            cookie: None,
            able: now,
            created: now,
            pargs: None,
        }
    }
}
