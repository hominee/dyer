extern crate config;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;

use crate::item::{Profile, Task};
use crate::spider::{Entity, ParseError};
use hyper::{client::HttpConnector, Client as hClient};
use hyper_timeout::TimeoutConnector;
use hyper_tls::HttpsConnector;
use std::sync::{Arc, Mutex};

pub fn get_parser(
    index: String,
) -> Box<dyn Fn(String) -> Result<(Vec<Entity>, Vec<Task>), ParseError> + Send> {
    if index == "parse_index1".to_owned() {
        Box::new(parse_index1)
    } else if index == "parse_index2".to_owned() {
        Box::new(parse_index2)
    } else if index == "parse_index3".to_owned() {
        Box::new(parse_index3)
    } else {
        Box::new(parse_default)
    }
}

pub fn parse_index1(_s: String) -> Result<(Vec<Entity>, Vec<Task>), ParseError> {
    Err(ParseError {
        desc: "".to_owned(),
    })
}
pub fn parse_index2(_s: String) -> Result<(Vec<Entity>, Vec<Task>), ParseError> {
    Err(ParseError {
        desc: "".to_owned(),
    })
}
pub fn parse_index3(_s: String) -> Result<(Vec<Entity>, Vec<Task>), ParseError> {
    Err(ParseError {
        desc: "".to_owned(),
    })
}
pub fn parse_default(_s: String) -> Result<(Vec<Entity>, Vec<Task>), ParseError> {
    Err(ParseError {
        desc: "this is default parser".to_owned(),
    })
}
