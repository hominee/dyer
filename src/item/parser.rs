extern crate serde;
extern crate serde_json;

use serde::{Deserialize, Deserializer, Serializer, Serialize};
use serde::ser::SerializeStruct;
use crate::spider::Spider;
use crate::item::{ParseResult, Response};
use crate::spider::S as Sapp;
use crate::spider::{MSpider};

type Item = &'static dyn Fn(&Sapp , &Response) -> Result<ParseResult, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Deserialize, Serialize)]
pub struct Parser {
    #[serde(deserialize_with = "deserialize_data")]
    #[serde(serialize_with = "serialize_data")]
    pub data: Box<Item>,
}

fn deserialize_data<'de, D>(d: D) -> Result<Box<Item>, D::Error>
where D: Deserializer<'de>, {
    let data = <&str>::deserialize(d)?;
    match Sapp::get_parser(data) {
        Some( parser )  => Ok( Box::new( parser ) ),
        None => Ok( Box::new( &Spider::parse ) )
    }
}

fn serialize_data<S>(d:&Box<Item>, serializer: S) -> Result<S::Ok, S::Error> 
where S: Serializer {
    let name = Sapp::fmap( d );
    let mut s = serializer.serialize_struct("Parser", 1)?;
    s.serialize_field("data", &name).unwrap();
    s.end()
}

impl Parser {
    pub fn get(ind: &str) -> Self {
        Parser{
           data: Box::new( &Sapp::parse )
        }
    }
}

impl Default for Parser {
    fn default() -> Self {
        Parser{
            data: Box::new( &Sapp::parse )
        }
    }
}
impl std::fmt::Debug for Parser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let d = &self.data;
        let name = Sapp::fmap( d );
        f.debug_struct("Parser")
            .field("data", &name)
            .finish()
    }
}
