use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex};
use crate::utils;

/// `Task`, as it means, a scheduled job to be done, contains most infomation of `Request`. For the  purposes of extensive compatibility,
/// A generic parameter `T` is required in dealing with it.
#[derive(Deserialize, Debug, Clone, Serialize)]
#[serde(bound = "T: Serialize + for<'a> Deserialize<'a> + Debug + Clone")]
pub struct Task<T>
where
    T: Serialize + for<'a> Deserialize<'a> + Debug + Clone,
{
    pub uri: String,
    pub method: String,
    /// additional headers if necessary
    pub headers: HashMap<String, String>,
    /// Formdata or other request parameter stored here
    pub body: Option<HashMap<String, String>>,
    /// checkpoint in seconds by which this `Task` is allowed to be executed
    pub able: f64,
    // FIXME add an member to AppArg
    /// times that this `Task` has failed, by default, the threshold is 2, customize it in `ArgApp`
    pub trys: u8,
    /// the index to get the parser parsing the `Response` when it's done
    pub parser: String,
    /// additional arguments for extensive application
    pub targs: Option<T>,
}

impl<T> Task<T>
where
    T: Serialize + for<'a> Deserialize<'a> + Debug + Clone,
{
    /// create an instance
    pub fn new() -> Self {
        let now = utils::now();
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

    /// store unfinished or extra `Task`s,
    pub fn stored(path: &str, task: &mut Arc<Mutex<Vec<Task<T>>>>) {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .unwrap();
        let mut buf = Vec::new();
        while let Some(r) = task.lock().unwrap().pop() {
            let s = serde_json::to_string(&r).unwrap();
            buf.push(s);
        }
        file.write(buf.join("\n").as_bytes()).unwrap();
    }

    /// load unfinished or extra `Task`s  
    pub fn load(path: &str) -> Vec<Task<T>> {
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
        return data;
    }
}

