mod profile;
mod request;
mod response;
mod task;
mod useragent;
mod weibo;

pub use profile::Profile;
pub use request::{ Request};
pub use response::Response;
pub use task::{Task};
pub use useragent::UserAgent;
pub use weibo::*;

use crate::spider::ParseError;
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
