extern crate serde;
extern crate serde_json;

use serde::{Deserialize, Deserializer, Serializer, Serialize};
use crate::spider::Spider;
use crate::item::{ParseResult, Response};
use crate::spider::S as Sapp;
use crate::spider::{MSpider};

type Item = &'static dyn Fn(&Sapp , &Response) -> Result<ParseResult, Box<dyn std::error::Error + Send + Sync>>;
type Ite = dyn Fn(&Sapp , &Response) -> Result<ParseResult, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Deserialize, Clone, Serialize)]
pub struct Parser {
    #[serde(deserialize_with = "deserialize_data")]
    #[serde(serialize_with = "serialize_data")]
    //pub data: Box<Item>,
    pub data: Item,
}

//fn deserialize_data<'de, D>(d: D) -> Result<Box<Item>, D::Error>
fn deserialize_data<'de, D>(d: D) -> Result<Item, D::Error>
where D: Deserializer<'de>, {
    let data = <&str>::deserialize(d)?;
    match Sapp::get_parser(data) {
        Some( parser )  => {
            let ptr: *const Item = &parser;
            println!("deserialize_data, with pointer: {:?}", ptr);
            Ok(  parser  )
        } 
        None => Ok(  &Spider::parse  )
        //Some( parser )  => Ok( Box::new( parser ) ),
        //None => Ok( Box::new( &Spider::parse ) )
    }
}

//fn serialize_data<S>(d:&Box<Item>, serializer: S) -> Result<S::Ok, S::Error> 
fn serialize_data<S>(d: &Item, serializer: S) -> Result<S::Ok, S::Error> 
where S: Serializer {
    let name = Sapp::fmap( d );
    println!("serialize name:{:?}", name);
    let ss = serializer.serialize_str(&name).unwrap();
    Ok(ss)
}

impl Parser {
    pub fn get(ind: &str) -> Self {
        Parser{
           //data: Box::new( &Sapp::parse )
           data:  &Sapp::parse 
        }
    }
}

impl Default for Parser {
    fn default() -> Self {
        Parser{
            //data: Box::new( &Sapp::parse )
           data:  &Sapp::parse 
        }
    }
}

impl std::fmt::Debug for Parser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let d = &self.data;
        let name = Sapp::fmap( d );
        //println!("debug: {:?}", name);
        f.debug_struct("Parser")
            .field("data", &name)
            .finish()
    }
}
