extern crate config;
extern crate hyper;
extern crate hyper_tls;
extern crate serde;
extern crate serde_json;

use config::Config;

use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, ErrorKind};

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
}

impl Profile {
    pub fn stored(profiles: Arc<Mutex<Vec<Profile>>>) {
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
