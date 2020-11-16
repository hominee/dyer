extern crate serde;
extern crate serde_json;

pub mod app;
pub mod parse;

//pub use app::{App, Spider}; 
pub use parse::{get_parser};

use crate::item::{Profile, ParseError, Response, Task};
use crate::item::ParseResult;
use serde::{Serialize, Deserialize};


///the trait that make sure App has an entry
///as well as the struct itself
/*
 *pub trait Entry {
 *    fn entry_profile() -> String;
 *    fn entry_task( &self ) -> Vec<Task>;
 *}
 */

/// the trait that handle the various Response
/// for status code above 300 or below 200 dispose these
pub trait HandleErr {
    fn hand100(&self, res: Response) -> (Task, Profile);
    fn hand300(&self, res: Response) -> (Task, Profile);
    fn hand400(&self, res: Response) -> (Task, Profile);
    fn hand500(&self, res: Response) -> (Task, Profile);
}

#[macro_export]
macro_rules! spd {
    (pub struct $name: ident {
        $($field_name: ident : $field_type: ty,)*
    }
    impl $name2: ident {
        pub fn entry_profile(&self,) -> Result<&'static str, Box<dyn std::error::Error + Send + Sync>> $profile: block

        pub fn entry_task(&self,) -> Result<Vec<Task>, Box<dyn std::error::Error + Send + Sync>> $task: block

        $(pub fn $func2: ident(&self, $($arg2_name: ident : $arg2_type: ty),*) -> $res2:ty $bk2: block )*
    }
    ) => {
        #[derive(Serialize, Deserialize)]
        pub struct $name {
            $($field_name: $field_type),*
        }
        type Item =dyn Fn(&'static $name, &Response) -> Result<ParseResult, Box<dyn std::error::Error + Send + Sync>>;
        type Sitem<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
        pub trait Spider {
            $(
                fn $func2(&self, $($arg2_name: $arg2_type)*) -> $res2 ;
            )*
        }
        impl Spider for $name {
            $(
                fn $func2(&self, $($arg2_name: $arg2_type)*) -> $res2 $bk2
            )*
        }
        pub trait MSpider {
            fn entry_profile(&self,) -> Sitem<&'static str> ;
            fn entry_task(&self,) -> Sitem<Vec<Task>> ;
            fn fields() -> Vec<&'static str>;
            fn methods() -> Vec<&'static str>;
            fn get_parser(ind: &str) -> Option<&'static dyn Fn(&$name, &Response) -> Result<ParseResult, Box<dyn std::error::Error + Send + Sync>>>; 
            fn map() -> std::collections::HashMap<&'static str, &'static Item>;
            fn fmap(f: &dyn Fn(&'static $name, &Response) -> Sitem<ParseResult>) -> String;
        }
        impl MSpider for $name {
            fn entry_profile(&self,) -> Sitem<&'static str> $profile

            fn entry_task(&self,) -> Sitem<Vec<Task>> $task

            fn fields() -> Vec<&'static str> {
                vec![ $( stringify!($field_name) ),* ]
            }

            fn methods() -> Vec<&'static str> {
                vec![ $( stringify!($func2),)* "entry_profile", "entry_task"  ] 
            }

            fn map() -> std::collections::HashMap<&'static str, &'static Item> {
                let mut mp = std::collections::HashMap::new();
                $( mp.insert(stringify!($func2), &$name::$func2 as &'static Item); )*
                mp
            }

            fn fmap(f: &dyn Fn(&'static $name, &Response) -> Sitem<ParseResult>) -> String {
                let v = vec![ $( $name::$func2 as *const &'static dyn Fn(&'static $name, &Response) -> Result<ParseResult, Box<dyn std::error::Error + Send + Sync>>),*];
                let mut i = 0;
                for item in v.into_iter() {
                    let prt: *const  &dyn Fn(&'static $name, &Response) -> Result<ParseResult, Box<dyn std::error::Error + Send + Sync>> = &f;
                    let iprt: *const Item = item;
                    if iprt == prt {
                        break;
                    }else {
                        i += 1;
                    }
                }
                let names = vec![ $( stringify!($func2).to_string(),)* ]; 
                names[i].to_string()

            }
            fn get_parser(ind: &str) -> Option<&'static dyn Fn(&$name, &Response) -> Sitem<ParseResult>> 
            {
                let v = vec![ $( &$name::$func2 as 
                    &'static dyn Fn(&$name, &Response) -> Result<ParseResult, Box<dyn std::error::Error + Send + Sync>>
                    ),*];
                let names = vec![ $( stringify!($func2) ),* ];
                let mut i = 0;
                for name in names.into_iter() {
                    if ind == name {
                        break;
                    } else {
                        i += 1;
                    }
                }
                if i <= v.len() -1 {
                    Some( v[i] )
                }else {
                    None
                }

            }

        }

    };
}


use crate::item::Parser;
spd!{
    pub struct S {
        parser: Parser,
    }
    impl S {
        pub fn entry_profile(&self,) -> Result<&'static str, Box<dyn std::error::Error + Send + Sync>> {
            println!("profile");
            Ok( "profile")
        }

        pub fn entry_task(&self,) -> Result<Vec<Task>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(vec![])
        }
        
        pub fn m1(&self, response: &Response) -> Result< ParseResult, Box<dyn std::error::Error + Send + Sync>> {
            println!("m1 called");
            Err( Box::new(ParseError{desc: "".to_owned()}) )
        }

        pub fn parse(&self, response: &Response) -> Result<ParseResult, Box<dyn std::error::Error + Send + Sync>> {
            println!("parse");
            Ok( ParseResult{
                entities: None,
                profile: None,
                task: None,
                req: None,
                yield_err: None,
            } )
        }
    }
    //impl Spider for S {
    //}
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_spd() {
        let s = r#"{"parser": {"data": "parse"}}"#;
        let s1 = r#"{"parser": {"data": "m1"}}"#;
        let ob: S = serde_json::from_str(s).unwrap();
        let ob1: S = serde_json::from_str(s1).unwrap();
        let ss = S{
            parser: Parser{
                data: Box::new( &S::parse ),
            }
        };
        let res = Response{
            headers: HashMap::new(),
            pheaders: HashMap::new(),
            theaders: HashMap::new(),
            status: 0,
            content: None,

            body: HashMap::new(),
            uri: "http://127.0.0.1".to_string(),
            method: "GET".to_string(),
            cookie: HashMap::new(),
            created: 64,
            parser: "parse".to_string(), 
            fparser: Parser{ data: Box::new( &S::parse ) },
            targs: None,
            msg: None,

            pargs: None,
        };
        let r1 = (ob.parser.data)(&ob, &res).unwrap();
        let r3 = (ob.parser.data)(&ob1, &res).unwrap();
        let r2 = (ss.parser.data)(&ss, &res).unwrap();
        println!("r1:{:?}\nr2:{:?}", r1.yield_err,r2.yield_err);
        println!("r3:{:?}", r3.entities);
        //(ob.parser.data)(&ob, &res);
        //assert_eq!( r1, r2);
    }
}
