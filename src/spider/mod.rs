extern crate serde;
extern crate serde_json;

pub mod app;
pub mod parse;

pub use app::App; 
pub use parse::{get_parser};

use crate::item::{Profile, Response, Task};
use hyper::{client::HttpConnector, Client as hClient};



///the trait that make sure App has an entry
///as well as the struct itself
pub trait Entry {
    fn entry_profile() -> String;
    fn entry_task( &self ) -> Vec<Task>;
}

/// the trait that handle the various Response
/// for status code above 300 or below 200 dispose these
pub trait HandleErr {
    fn hand100(&self, res: Response) -> (Task, Profile);
    fn hand300(&self, res: Response) -> (Task, Profile);
    fn hand400(&self, res: Response) -> (Task, Profile);
    fn hand500(&self, res: Response) -> (Task, Profile);
}


