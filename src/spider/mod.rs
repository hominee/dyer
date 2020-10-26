extern crate serde;
extern crate serde_json;

pub mod app;
pub mod parse;

pub use app::{App, Entity, PaerseResult};
pub use parse::{fake, get_parser};

use crate::item::{Profile, ResError, Response, Task};
use hyper::{client::HttpConnector, Client as hClient};
use hyper_tls::HttpsConnector;

use std::error::Error;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

///the trait that make sure App has an entry
///as well as the struct itself
pub trait Entry {
    fn start_request(
        &self,
        client: hClient<HttpsConnector<HttpConnector>>,
        response: Arc<Mutex< Vec<Response> >> 
    );
}

/// the trait that handle the various Response
/// for status code above 300 or below 200 dispose these
pub trait HandleErr {
    fn hand100(&self, res: Response) -> (Task, Profile);
    fn hand300(&self, res: Response) -> (Task, Profile);
    fn hand400(&self, res: Response) -> (Task, Profile);
    fn hand500(&self, res: Response) -> (Task, Profile);
}

///the trait that parse the response
pub trait Parse {
    fn parse(body: Response) -> Result<PaerseResult, ParseError>;
}

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
pub struct UtilError {
    pub desc: String,
}
impl std::fmt::Display for UtilError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Util Error.")
    }
}
impl Error for UtilError {}
