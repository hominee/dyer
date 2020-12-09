extern crate serde;
extern crate serde_json;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;
use std::io::LineWriter;
use std::io::{BufRead, BufReader, ErrorKind};
use std::sync::{Arc, Mutex};

#[derive(Deserialize, Debug, Serialize)]
pub struct Task {
    pub uri: String,
    pub method: String,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<HashMap<String, String>>,
    pub able: u64,
    pub parser: String,
    pub targs: Option<TArgs>,
}

#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct TArgs {}

impl Task {
    pub fn stored(path: &str, task: &Arc<Mutex<Vec<Task>>>) {
        let file = fs::File::open(path).unwrap();
        let mut writer = LineWriter::new(file);
        task.lock().unwrap().iter().for_each(|r| {
            serde_json::to_writer(&mut writer, r).unwrap();
        });
    }

    pub fn load(path: &str) -> Option<Vec<Task>> {
        // load Profile here
        let file = fs::File::open(path);
        match file {
            Err(e) => match e.kind() {
                ErrorKind::NotFound => {
                    fs::File::create(path.to_string() + "_old").unwrap();
                    fs::File::create(path).unwrap();
                    return None;
                }
                _ => unreachable!(),
            },
            Ok(content) => {
                let buf = BufReader::new(content).lines();
                let mut data: Vec<Task> = Vec::new();
                buf.into_iter().for_each(|line| {
                    let task: Task = serde_json::from_str(&line.unwrap()).unwrap();
                    data.push(task);
                });
                fs::remove_file(path.to_string() + "_old").unwrap();
                fs::rename(path, path.to_string() + "_old").unwrap();
                return Some(data);
            }
        }
    }
}

impl Default for Task {
    fn default() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Task {
            uri: "".to_string(),
            method: "GET".to_string(),
            headers: None,
            body: None,
            able: now,
            parser: "parse".to_string(),
            targs: None,
        }
    }
}
