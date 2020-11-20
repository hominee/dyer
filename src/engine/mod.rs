mod profile;
mod request;
mod response;
mod task;
mod useragent;
mod parser;
mod macros;

pub use profile::{ PArgs, Profile };
pub use request::{ Request};
pub use response::{Response, ParseResult, Parse,  Entity};
pub use task::{Task, TArgs};
pub use useragent::UserAgent;
pub use parser::Parser;
pub use macros::{S, Spider, MSpider, HandleErr};

use std::error::Error;
use std::fmt::Debug;

#[derive(Debug)]
pub struct PfileError {
    pub desc: String,
}
impl std::fmt::Display for PfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Profile Error.")
    }
}
impl Error for PfileError {}

#[derive(Debug)]
pub struct ParseError {
    pub desc: String,
}
impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse Error.")
    }
}

impl Error for ParseError {}
#[derive(Debug)]
pub struct ReqError {
    pub desc: String,
}
impl std::fmt::Display for ReqError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Request Error")
    }
}
impl Error for ReqError {}

#[derive(Debug)]
pub struct ResError {
    pub desc: String,
}
impl std::fmt::Display for ResError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Response Error.")
    }
}
impl Error for ResError {}

#[derive(Debug)]
pub struct TaskError {
    pub desc: String,
}
impl std::fmt::Display for TaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Task Error.")
    }
}
impl Error for TaskError {}
