//! Instructions of components including [client], [profile], [request], [response], [task], [utils].
//!
//! # OverView
//!
//! `client` contains some methods to execute `Request` as Future;
//!
//! `utils` is a collection of useful tools like [hash] get now time stamp [now] and other stuff;
//!
//! the others, as the file name suggests, serve as component in integration
//!
//! [hash]: crate::component::utils::hash
//! [now]: crate::component::utils::now
//! [client]: crate::component::client
//! [profile]: crate::component::profile
//! [request]: crate::component::request
//! [response]: crate::component::response
//! [task]: crate::component::task
//! [utils]: crate::component::utils
//!
pub mod client;
pub mod profile;
pub mod request;
pub mod response;
pub mod task;
pub mod utils;

#[doc(hidden)]
pub use client::Client;
#[doc(hidden)]
pub use profile::Profile;
#[doc(hidden)]
pub use request::Request;
#[doc(hidden)]
pub use response::{ParseResult, Response};
#[doc(hidden)]
pub use task::Task;
#[doc(hidden)]
pub use utils::get_cookie;

use std::error::Error;
use std::fmt::Debug;

#[derive(Debug)]
pub struct ProfileError {
    pub desc: String,
}
impl std::fmt::Display for ProfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Profile Error.")
    }
}
impl Error for ProfileError {}

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
unsafe impl Send for ResError {}
unsafe impl Sync for ResError {}

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
