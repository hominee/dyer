extern crate serde;
extern crate serde_json;
extern crate rand;

use crate::item::{Profile, ParseError, ParseResult, Request, ResError, Response, Task};
//use crate::spider::{Entry, };

/*
 *type Sitem<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
 *pub trait Spider {
 *    fn entry_profile(&self) -> Sitem<&'static str>;
 *    fn entry_task(&self) -> Sitem<Vec<Task>>;
 *    fn parse(&self, response: &Response) -> Sitem<ParseResult>;
 *}
 */

#[derive(Debug, Clone )]
pub struct App;


