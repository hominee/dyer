//#![allow(unused_imports)]

extern crate dyer;
extern crate serde;
extern crate tokio;
extern crate simple_logger;
extern crate select;
extern crate futures;

use simple_demo::*; 
use dyer::*;
use entity::{Entities, Targ, Parg};
use spider::MySpider;
use middleware::{handle_profile, handle_req, handle_task};
use pipeline::{open_file, store_item};
use std::sync::{Arc, Mutex};


#[tokio::main]
async fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();
    let middleware = plug!( MiddleWare<Entities, Targ, Parg> {
        handle_profile: handle_profile,
        handle_req: handle_req,
        handle_task: handle_task
    });
    let pipeline = plug!( PipeLine<Entities, std::fs::File> {
        open_pipeline: open_file,
        process_entity: store_item
    } );
    let spider = MySpider::new();
    let mut app = dyer::App::<Entities, Targ, Parg>::new();
    app.run(&spider, &middleware, pipeline).await.unwrap();
}
        