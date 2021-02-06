extern crate serde;
extern crate serde_json;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex};

#[derive(Deserialize, Debug, Clone, Serialize)]
#[serde(bound = "TArgs: Serialize + for<'a> Deserialize<'a> + Debug + Clone")]
pub struct Task<TArgs>
where
    TArgs: Serialize + for<'a> Deserialize<'a> + Debug + Clone,
{
    pub uri: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: Option<HashMap<String, String>>,
    pub able: u64,
    pub trys: u8,
    pub parser: String,
    pub targs: Option<TArgs>,
}

impl<T> Task<T>
where
    T: Serialize + for<'a> Deserialize<'a> + Debug + Clone,
{
    pub fn stored(path: &str, task: &mut Arc<Mutex<Vec<Task<T>>>>) {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .unwrap();
        let mut buf = Vec::new();
        task.lock().unwrap().iter().for_each(|r| {
            let s = serde_json::to_string(&r).unwrap();
            buf.push(s);
        });
        file.write(buf.join("\n").as_bytes()).unwrap();
    }

    pub fn load(path: &str) -> Option<Vec<Task<T>>> {
        // load Profile here
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
                let task: Task<T> = serde_json::from_str(&s).unwrap();
                task
            })
            .collect::<Vec<Task<T>>>();
        return Some(data);
    }
}

impl<T> Default for Task<T>
where
    T: Serialize + for<'a> Deserialize<'a> + Clone + Debug,
{
    fn default() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Task::<T> {
            uri: "".to_string(),
            method: "GET".to_string(),
            headers: HashMap::new(),
            body: None,
            able: now,
            trys: 0,
            parser: "".to_string(),
            targs: None,
        }
    }
}
