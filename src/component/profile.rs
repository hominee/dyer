extern crate serde;
extern crate serde_json;

use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, ErrorKind};

use crate::component::{Client, Response, Request, ResError, UserAgent};
use crate::macros::Spider;
use futures::future::join_all;
use log::{error, info};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::io::LineWriter;
use std::sync::{Arc, Mutex};

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
    pub fn exec(res: &mut Response) -> Result<Profile, ResError> {
        let raw_headers = &res.headers;
        let stop_word = ["path", "expires", "domain", "httpOnly"];
        let mut cookie = HashMap::new();
        raw_headers.into_iter().for_each(|(k, val)| {
            if k == "set-cookie" {
                let v_str: Vec<&str> = val
                    .split(";")
                    .filter(|c| !stop_word.contains(&c.trim()))
                    .collect();
                v_str.into_iter().for_each(|pair| {
                    let tmp: Vec<&str> = pair.split("=").collect();
                    if tmp.len() == 2 {
                        cookie.insert(tmp[0].to_string(), tmp[1].to_string());
                    }
                });
            }
        });
        res.cookie = cookie;
        res.content = Some("".to_string());
        let (_, profile) = res.into1().unwrap();
        Ok(profile)
    }

    pub async fn exec_all<Entity>(
        spd: &'static dyn Spider<Entity>,
        profiles: Arc<Mutex<Vec<Profile>>>,
        uri: &str,
        num: usize,
        uas: Arc<Vec<UserAgent>>,
    ) {
        let client = &Client::new(7, 23, 7)[0];
        let mut rs = vec![];
        let mut vreq = Vec::new();
        vec![0; num].iter().for_each(|_| {
            // select a ua
            let len = uas.len();
            let ind = rand::thread_rng().gen_range(0, len - 1);
            let ua = uas[ind].clone().user_agent;
            // construct a new reqeust
            let mut req = Request::default();
            req.uri = uri.clone().to_string();
            let mut hd = req.pheaders;
            hd.insert("User-Agent".to_string(), ua);
            req.pheaders = hd;
            let p = Response::default( Some( &req ) );
            rs.push( p );
            if let Some(t) = req.init() {
                vreq.push( client.request(t));
            }
        });
        // poll all request concurrently
        let vres = join_all(vreq).await;
        let mut i = 0usize;
        vres.into_iter().for_each(|r| {
            let mut p = rs.pop().unwrap();
            match r {
                Ok(res) => {
                    let mut hd_res = HashMap::new();
                    let (bd, _) = res.into_parts();
                    let raw_headers = bd.headers;
                    raw_headers.into_iter().for_each(|(k, v)| {
                        let key = k.unwrap().to_string();
                        let val = v.to_str().unwrap_or("").to_string();
                        hd_res.insert(key, val);
                    });
                    p.headers.extend( hd_res );
                    let f = spd.get_parser("gen_profile");
                    let profile = match f {
                        None => { 
                            Some( Profile::exec( &mut p ).unwrap() )
                        },
                        Some(func) => {
                            let result = func(&p).unwrap();
                            result.profile 
                        }
                    };
                    profiles.lock().unwrap().extend( profile );
                    i += 1;
                },
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

impl Profile {
    pub fn stored(path: &str, profiles: &Arc<Mutex<Vec<Profile>>>) {
        let file = fs::File::open(path).unwrap();
        let mut writer = LineWriter::new(file);
        profiles.lock().unwrap().iter().for_each(|r| {
            serde_json::to_writer(&mut writer, &r).unwrap();
        });
    }

    pub fn load(path: &str) -> Option<Vec<Profile>> {
        let file = fs::File::open(path);
        match file {
            Err(e) => match e.kind() {
                ErrorKind::NotFound => {
                    fs::File::create(path).unwrap();
                    fs::File::create(path.to_string() + "_old").unwrap();
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
                fs::remove_file(path.to_string() + "_old").unwrap();
                fs::rename(path, path.to_string() + "_old").unwrap();
                return Some(data);
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
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8"
                .to_string(),
        );
        headers.insert(
            "Accept-Encoding".to_string(),
            "gzip, deflate, br".to_string(),
        );
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
