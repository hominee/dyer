extern crate serde;
extern crate serde_json;

use serde::{Deserialize, Deserializer, Serializer, Serialize};
use crate::component::{Response, ParseResult};
use crate::macros::{Spider, MSpider};
use crate::macros::Mate;
use std::str::FromStr;

type Item<T> = &'static dyn Fn(&dyn Spider, &Response<T>) -> Result<ParseResult<T>, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Deserialize, Clone, Serialize)]
pub struct Parser<T> where T: MSpider + ?Sized { 
    #[serde(deserialize_with = "deserialize_data::<_, T>")]
    //#[serde(bound( deserialize = "T: MSpider,  for<'de2> T: Deserialize<'de2>" ))]
    #[serde(serialize_with = "serialize_data::<_, T>")]
    pub data: Item<T>,
    #[serde(skip)]
    marker: std::marker::PhantomData<T>,
}

fn deserialize_data<'de, D, T, >(d: D) -> Result<Item<T>, D::Error>
where D: Deserializer<'de>, T: MSpider + ?Sized {
    let data = <&str>::deserialize(d)?;
    match T::get_parser(data) {
        Some( parser )  => {
            let ptr: *const Item<T> = &parser;
            println!("deserialize_data, with pointer: {:?}", ptr);
            Ok(  parser  )
        } 
        None => Ok( T::get_parser("parse").unwrap() )
    }
}

fn serialize_data<S, T>(d: &Item<T>, serializer: S) -> Result<S::Ok, S::Error> 
where S: Serializer, T: MSpider + ?Sized {
    let name = T::fmap( d );
    println!("serialize name:{:?}", name);
    let ss = serializer.serialize_str(&name).unwrap();
    Ok(ss)
}

impl<T> Parser<T> where T: MSpider + ?Sized {
    pub fn get(ind: &str) -> Self {
        Parser::<T>{
           data:  &T::get_parser("parse").unwrap(),
           marker: std::marker::PhantomData::<T>,
        }
    }
}

impl<T> Default for Parser<T> where T: MSpider + ?Sized{
    fn default() -> Self {
        Parser::<T>{
           data:  &T::get_parser("parse").unwrap(),
           marker: std::marker::PhantomData::<T>,
        }
    }
}

impl<T> std::fmt::Debug for Parser<T> where T: MSpider+?Sized {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let d = &self.data;
        let name = T::fmap( d );
        f.debug_struct("Parser")
            .field("data", &name)
            .finish()
    }
}
