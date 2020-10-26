extern crate config;
extern crate serde;
extern crate serde_json;

use crate::spider::get_parser;
use crate::spider::{Entity, ParseError};
use config::Config;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;
use std::io::LineWriter;
use std::io::{BufRead, BufReader, ErrorKind};
use std::sync::{Arc, Mutex};

pub struct Task {
    pub uri: String,
    pub method: String,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<HashMap<String, String>>,
    pub able: u64,
    pub parser: Box<dyn Fn(String) -> Result<(Vec<Entity>, Vec<Task>), ParseError> + Send>,
    pub raw_parser: String,
    pub args: Option<HashMap<String, Vec<String>>>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct RawTask {
    pub uri: String,
    pub method: String,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<HashMap<String, String>>,
    pub able: u64,
    pub raw_parser: String,
    pub args: Option<HashMap<String, Vec<String>>>,
}

impl Task {
    pub fn stored(task: Arc<Mutex<Vec<Task>>>) {
        let mut setting = Config::default();
        setting.merge(config::File::with_name("setting")).unwrap();
        let path = setting.get_str("path_task").unwrap() + "/task.txt";
        let file = fs::File::open(path).unwrap();
        let mut writer = LineWriter::new(file);
        task.lock().unwrap().iter().for_each(|r| {
            serde_json::to_writer(&mut writer, &Task::into_raw(r)).unwrap();
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
                            fs::File::create(path.clone() + "/task.txt").unwrap();
                            fs::File::create(path + "/task.txt").unwrap();
                            return None;
                        }
                        _ => unreachable!(),
                    },
                    Ok(content) => {
                        let buf = BufReader::new(content).lines();
                        let mut data: Vec<Task> = Vec::new();
                        buf.into_iter().for_each(|line| {
                            let raw_task: RawTask = serde_json::from_str(&line.unwrap()).unwrap();
                            let task = Task::from_raw(raw_task);
                            data.push(task);
                        });
                        fs::remove_file(path.clone() + "/task_old.txt").unwrap();
                        fs::rename(path.clone() + "/task.txt", path + "/task.txt").unwrap();
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

    pub fn from_raw(rtask: RawTask) -> Self {
        let parser = get_parser(rtask.raw_parser.clone());
        Task {
            uri: rtask.uri,
            method: rtask.method,
            headers: rtask.headers,
            body: rtask.body,
            able: rtask.able,
            args: rtask.args,
            raw_parser: rtask.raw_parser,
            parser,
        }
    }

    pub fn into_raw(task: &Task) -> RawTask {
        RawTask {
            uri: task.uri.clone(),
            method: task.method.clone(),
            headers: task.headers.clone(),
            body: task.body.clone(),
            able: task.able.clone(),
            args: task.args.clone(),
            raw_parser: task.raw_parser.clone(),
        }
    }
}
