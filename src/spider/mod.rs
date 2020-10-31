extern crate serde;
extern crate serde_json;

pub mod app;
pub mod parse;

pub use app::{App, Entity, ParseResult};
pub use parse::{get_parser};

use crate::item::{Profile, Request, Response, Task};
use hyper::{client::HttpConnector, Client as hClient};
use hyper_timeout::TimeoutConnector;
use hyper_tls::HttpsConnector;
use std::error::Error;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};


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

///the trait that make sure App has an entry
///as well as the struct itself
pub trait Entry {
    fn start_request(
        &self,
        client: hClient<TimeoutConnector<HttpsConnector<HttpConnector>>>,
        response: Arc<Mutex<Vec<Response>>>,
        profiles: Arc<Mutex<Vec<Profile>>>,
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

    fn parse(body: Response) -> Result<ParseResult, ParseError>;
    fn parse_all(vres: Arc<Mutex< Vec<Response> >>, vreq: Arc<Mutex<  Vec<Request> >>, vtask: Arc<Mutex< Vec<Task> >>, vpfile: Arc<Mutex< Vec<Profile> >>, entities: Arc<Mutex< Vec<Entity> >>, yield_err: Arc<Mutex< Vec<String> >>, round: usize  );

}

