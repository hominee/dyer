#![feature(type_alias_impl_trait)]

extern crate serde;
extern crate serde_json;

pub mod app;
pub mod parse;

//pub use app::{App, Spider}; 
pub use parse::{get_parser};

use crate::item::{Profile, ParseError, Response, Task};
use crate::item::ParseResult;


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

        $(pub fn $func2: ident(&'static self, $($arg2_name: ident : $arg2_type: ty),*) -> $res2:ty $bk2: block )*
    }
    ) => {
        pub struct $name {
            $($field_name: $field_type),*
        }
        type Item =dyn Fn(&'static $name, &Response) -> Result<ParseResult, Box<dyn std::error::Error + Send + Sync>>;
        type Sitem<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
        pub trait Spider {
            $(
                fn $func2(&'static self, $($arg2_name: $arg2_type)*) -> $res2 ;
            )*
        }
        impl Spider for $name {
            $(
                fn $func2(&'static self, $($arg2_name: $arg2_type)*) -> $res2 $bk2
            )*
        }
        pub trait MSpider {
            fn entry_profile(&self,) -> Result<&'static str, Box<dyn std::error::Error + Send + Sync>> ;
            fn entry_task(&self,) -> Result<Vec<Task>, Box<dyn std::error::Error + Send + Sync>> ;
            fn fields() -> Vec<&'static str>;
            fn methods() -> Vec<&'static str>;
            fn get_parser(ind: &str) -> Option<&'static Item>; 
            fn map() -> std::collections::HashMap<&'static str, &'static Item>;
            fn fmap(f: &dyn Fn(&'static $name, &Response) -> Result<ParseResult, Box<dyn std::error::Error + Send + Sync>>) -> String;
        }
        impl MSpider for $name {
            fn entry_profile(&self,) -> Result<&'static str, Box<dyn std::error::Error + Send + Sync>> $profile

            fn entry_task(&self,) -> Result<Vec<Task>, Box<dyn std::error::Error + Send + Sync>> $task

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

            fn fmap(f: &dyn Fn(&'static $name, &Response) -> Result<ParseResult, Box<dyn std::error::Error + Send + Sync>>) -> String {
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
            fn get_parser(ind: &str) -> Option<&'static dyn Fn(&'static $name, &Response) -> Result<ParseResult, Box<dyn std::error::Error + Send + Sync>>> 
            {
                let v = vec![ $( &$name::$func2 as 
                    &'static dyn Fn(&'static $name, &Response) -> Result<ParseResult, Box<dyn std::error::Error + Send + Sync>>
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


spd!{
    pub struct S {
        a: Vec<&'static str>,
        b: String,
    }
    impl S {
        pub fn entry_profile(&self,) -> Result<&'static str, Box<dyn std::error::Error + Send + Sync>> {
            println!("profile");
            Ok( "profile")
        }

        pub fn entry_task(&self,) -> Result<Vec<Task>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(vec![])
        }
        
        pub fn m1(&'static self, response: &Response) -> Result< ParseResult, Box<dyn std::error::Error + Send + Sync>> {
            println!("m1 called");
            Err( Box::new(ParseError{desc: "".to_owned()}) )
        }

        pub fn parse(&'static self, response: &Response) -> Result<ParseResult, Box<dyn std::error::Error + Send + Sync>> {
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

use S as Sapp;
pub fn ns() {
    Sapp::get_parser("m1");
}
