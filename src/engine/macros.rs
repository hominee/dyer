extern crate serde;
extern crate serde_json;

use crate::engine::{Profile, ParseError, Response, Task};
use crate::engine::ParseResult;
use serde::{Serialize, Deserialize};
use crate::engine::Parser;
use std::sync::Once;



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
        #[derive(Serialize, std::fmt::Debug, Deserialize)]
        pub struct $name {
            $($field_name: $field_type),*
        }
        type Item =dyn Fn(&$name, &Response) -> Result<ParseResult, Box<dyn std::error::Error + Send + Sync>>;
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
            fn meta() -> &'static (Vec<&'static str>, Vec<&'static str>);
            fn methods() -> &'static Vec<&'static Item> ;
            fn get_parser(ind: &str) -> Option<&'static dyn Fn(&$name, &Response) -> Result<ParseResult, Box<dyn std::error::Error + Send + Sync>>>; 
            fn map() -> std::collections::HashMap<&'static str, &'static Item>;
            fn fmap(f: &&Item) -> String;
        }
        impl MSpider for $name {
            fn entry_profile(&self,) -> Sitem<&'static str> $profile

            fn entry_task(&self,) -> Sitem<Vec<Task>> $task

            fn meta() -> &'static (Vec<&'static str>, Vec<&'static str>) {
                static INIT: Once = Once::new();
                static mut VAL: (Vec<&'static str>, Vec<&'static str>) = ( vec![], vec![] );
                unsafe{
                    INIT.call_once(|| {
                        VAL = (
                            vec![ $( stringify!($field_name) ),* ],
                            vec![ $( stringify!($func2),)*]  
                        );
                    });
                    &VAL
                }
            }

            fn methods() -> &'static Vec<&'static Item>  {
                static INIT: Once = Once::new();
                static mut VAL: Vec<&'static Item> = vec![];
                unsafe{
                    INIT.call_once(|| {
                        VAL = vec![ $( &$name::$func2 as &'static Item ),*];
                    });
                    &VAL
                }
            }

            fn map() -> std::collections::HashMap<&'static str, &'static Item> {
                let mut mp = std::collections::HashMap::new();
                $( mp.insert(stringify!($func2), &$name::$func2 as &'static Item); )*
                mp
            }

            fn fmap(f: &&Item ) -> String {
                //let v = vec![ $( $name::$func2 as *const &dyn Fn(&$name, &Response) -> Result<ParseResult, Box<dyn std::error::Error + Send + Sync>>),*];
                let v0 = $name::methods();
                let mut v = Vec::new();
                v0.into_iter().for_each(|func| {
                    v.push( *func as *const Item );
                });
                println!("vec of pointer: {:?}", v);
                let vlen = v.len();
                let mut i = 0;
                for item in v.into_iter() {
                    //let prt: *const  &dyn Fn(&$name, &Response) -> Result<ParseResult, Box<dyn std::error::Error + Send + Sync>> = &f;
                    let prt: *const  dyn Fn(&$name, &Response) -> Result<ParseResult, Box<dyn std::error::Error + Send + Sync>> = &**f;
                    let iprt: *const Item = item;
                    println!("prt: {:?}, iprt: {:?}", prt, iprt);
                    if iprt == prt {
                        break;
                    }else {
                        i += 1;
                    }
                }
                let names = &$name::meta().1;
                println!("in fmap, len: {}, names: {:?}, i:{}", names.len(), names, i);
                if i == vlen {
                    panic!("not found the method.")
                } else {
                    names[i].to_string()
                }

            }
            fn get_parser(ind: &str) -> Option<&'static dyn Fn(&$name, &Response) -> Sitem<ParseResult>> 
            {
                let v0 = $name::methods();
                let mut v = Vec::new();
                v0.into_iter().for_each(|func| {
                    v.push( *func as &'static Item );
                });
                let names = &$name::meta().1;
                let mut i = 0;
                for name in names.into_iter() {
                    if ind == *name {
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
            Ok( ParseResult{
                entities: None,
                profile: None,
                task: None,
                req: None,
                yield_err: None,
            } )
        }

        pub fn m2(&self, response: &Response) -> Result< ParseResult, Box<dyn std::error::Error + Send + Sync>> {
            println!("m1 called");
            Ok( ParseResult{
                entities: None,
                profile: None,
                task: None,
                req: None,
                yield_err: None,
            } )
        }

        pub fn m3(&self, response: &Response) -> Result< ParseResult, Box<dyn std::error::Error + Send + Sync>> {
            println!("m1 called");
            Ok( ParseResult{
                entities: None,
                profile: None,
                task: None,
                req: None,
                yield_err: None,
            } )
        }

        pub fn parse(&self, response: &Response) -> Result<ParseResult, Box<dyn std::error::Error + Send + Sync>> {
            println!("parse called");
            Ok( ParseResult{
                entities: None,
                profile: None,
                task: None,
                req: None,
                yield_err: None,
            } )
        }
        pub fn parse_index(&self, response: &Response) -> Result<ParseResult, Box<dyn std::error::Error + Send + Sync>> {
            println!("parse called");
            Ok( ParseResult{
                entities: None,
                profile: None,
                task: None,
                req: None,
                yield_err: None,
            } )
        }
        pub fn parse_content(&self, response: &Response) -> Result<ParseResult, Box<dyn std::error::Error + Send + Sync>> {
            println!("parse called");
            Ok( ParseResult{
                entities: None,
                profile: None,
                task: None,
                req: None,
                yield_err: None,
            } )
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_spd() {
        let s = r#"{"parser":{"data":"parse"}}"#;
        let t2 = r#"{"parser":{"data":"parse_index"}}"#;
        let t3 = r#"{"parser":{"data":"parse_content"}}"#;
        let s1 = r#"{"parser":{"data":"m1"}}"#;
        let s2 = r#"{"parser":{"data":"m2"}}"#;
        let s3 = r#"{"parser":{"data":"m3"}}"#;
        let ob: S = serde_json::from_str(s).unwrap();
        let ot2: S = serde_json::from_str(t2).unwrap();
        let ot3: S = serde_json::from_str(t3).unwrap();
        let ob1: S = serde_json::from_str(s1).unwrap();
        let ob2: S = serde_json::from_str(s2).unwrap();
        let ob3: S = serde_json::from_str(s3).unwrap();
        let ss = S{
            parser: Parser{
                //data: Box::new( &S::parse ),
                data:  &S::parse 
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
            fparser: Parser{ data:  &S::parse  },
            targs: None,
            msg: None,

            pargs: None,
        };
        let r1 = (ob.parser.data)(&ob, &res).unwrap();
        let r3 = (ob1.parser.data)(&ob1, &res).unwrap();
        let r2 = (ss.parser.data)(&ss, &res).unwrap();
        println!("r1:{:?}\nr2:{:?}", r1.yield_err,r2.yield_err);
        println!("r3:{:?}", r3.entities);

        assert_eq!(s, serde_json::to_string(&ob).unwrap());
        println!("ob1: {:?}", ob1);
        assert_eq!(s1, serde_json::to_string(&ob1).unwrap());
        assert_eq!(s2, serde_json::to_string(&ob2).unwrap());
        assert_eq!(t2, serde_json::to_string(&ot2).unwrap());
        assert_eq!(t3, serde_json::to_string(&ot3).unwrap());
        assert_eq!(s3, serde_json::to_string(&ob3).unwrap());
    }
}
