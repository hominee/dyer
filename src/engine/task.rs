extern crate config;
extern crate serde;
extern crate serde_json;

use config::Config;
use serde::{ Serialize, Deserialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;
use std::io::LineWriter;
use std::io::{BufRead, BufReader, ErrorKind};
use std::sync::{Arc, Mutex};
use crate::engine::{Response,Parser, ParseResult};
use crate::engine::Spider;


type Item = &'static dyn Fn(&'static dyn Spider, Response) -> Result<ParseResult, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Deserialize,  Debug, Serialize)]
pub struct Task {
    pub uri: String,
    pub method: String,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<HashMap<String, String>>,
    pub able: u64,
    //#[serde(skip)]
    pub fparser: Parser,
    pub targs: Option<TArgs>,
}


#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct TArgs {}

impl Task {
    pub fn stored(task: Arc<Mutex<Vec<Task>>>) {
        let mut setting = Config::default();
        setting.merge(config::File::with_name("setting")).unwrap();
        let path = setting.get_str("path_task").unwrap() + "/task.txt";
        let file = fs::File::open(path).unwrap();
        let mut writer = LineWriter::new(file);
        task.lock().unwrap().iter().for_each(|r| {
            serde_json::to_writer(&mut writer, r).unwrap();
        });
    }

    pub fn load() -> Option<Vec<Task>> {
        let mut setting = Config::default();
        setting
            // load from file
            .merge(config::File::with_name("setting"))
            .unwrap();
        // load from PATH
        //.merge(config::Environment::with_prefix("APP")).unwrap();
        match setting.get_str("path_task") {
            Ok(path) => {
                // load Profile here
                let file = fs::File::open(path.clone() + "task.txt");
                match file {
                    Err(e) => match e.kind() {
                        ErrorKind::NotFound => {
                            fs::File::create(path.clone() + "/task_old.txt").unwrap();
                            fs::File::create(path + "/task.txt").unwrap();
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
                        fs::remove_file(path.clone() + "/task_old.txt").unwrap();
                        fs::rename(path.clone() + "/task.txt", path + "/task_old.txt").unwrap();
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

impl Default for Task {
    fn default() -> Self {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        Task{
            uri: "".to_string(),
            method: "GET".to_string(),
            headers: None,
            body: None,
            able: now,
            fparser: Parser::default(),
            targs: None,
        }
    }
}
