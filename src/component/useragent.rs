use log::error;
use serde::{Deserialize, Serialize};
use serde_json;
use std::io::BufReader;

#[derive(Serialize, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserAgent {
    pub user_agent: String,
    pub weight: f64,
    pub platform: String,
    pub device_category: String,
}

impl UserAgent {
    pub fn load(path: String) -> Vec<UserAgent> {
        match std::fs::File::open(path) {
            Ok(file) => {
                let buf = BufReader::new(file);
                let data: Vec<UserAgent> = serde_json::from_reader(buf).unwrap();
                data
            }
            Err(_) => {
                error!("User Agent file must provide!");
                panic!("User Agent file must provide!");
            }
        }
    }
}
