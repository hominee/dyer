extern crate serde;
extern crate serde_json;
extern crate rand;

use crate::item::{Profile, ParseError, Request, ResError, Response, Task};
use crate::spider::{Entry, };
use futures::executor::block_on;
use hyper::{client::HttpConnector, Client as hClient};
use hyper_tls::HttpsConnector;
use hyper_timeout::TimeoutConnector;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone )]
pub struct App {
    pub urls: Vec<String>,
    pub parsers: Vec<String>,
}

impl App {
    /// manual specify the starting point
    pub fn new() -> App {
        //initialize it as a default
        App { 
            urls:Vec::new(), 
            parsers: Vec::new() 
        }
    }

}

impl Entry for App {
    /// the url that Profile make
    fn entry_profile() -> String {
        return "https://www.zhihu.com/topics".to_string();
    }

    fn entry_task( &self) -> Vec<Task> {
        //the started url that spark the crawler
        let mut tasks: Vec<Task> = Vec::new();
        let mut urls = self.urls.to_owned();
        let mut parsers = self.parsers.to_owned();
        for _ in 0..urls.len() {
            // fake a profile
            let mut task: Task = Task::default();
            let url = urls.pop().unwrap();
            let parser = parsers.pop().unwrap();
            task.uri = url.to_owned();
            task.parser = parser;
            tasks.push(task);
        }

        tasks
    }
}

