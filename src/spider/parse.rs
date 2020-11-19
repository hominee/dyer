extern crate config;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;

use crate::item::{Entity,ParseError, Task};
use hyper::{client::HttpConnector, Client as hClient};


pub fn parse_index1(_s: String) -> Result<(Vec<Entity>, Vec<Task>), ParseError> {
    Err(ParseError {
        desc: "".to_owned(),
    })
}
