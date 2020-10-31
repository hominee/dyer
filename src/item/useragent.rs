use log::error;
use serde::{Deserialize, Serialize};
use serde_json;
use std::io::{BufRead, BufReader};

#[derive(Serialize, Clone, Deserialize)]
pub struct UserAgent {
    pub userAgent: String,
    pub weight: f64,
    pub platform: String,
    pub deviceCategory: String,
}

impl UserAgent {
    pub fn load(path: String) -> Vec<UserAgent> {
        match std::fs::File::open(path) {
            Ok(file) => {
                let buf = BufReader::new(file);
                let data: Vec<UserAgent> = serde_json::from_reader(buf).unwrap();
                data
            }
            Err(e) => {
                error!("User Agent file must provide!");
                panic!("User Agent file must provide!");
            }
        }
    }
}
