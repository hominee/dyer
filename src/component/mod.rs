pub mod client;
pub mod profile;
pub mod request;
pub mod response;
pub mod task;
//pub mod useragent;
pub mod utils;

pub use client::Client;
pub use profile::Profile;
pub use request::Request;
pub use response::{ParseResult, Response};
pub use task::Task;
//pub use useragent::UserAgent;
pub use utils::get_cookie;

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
