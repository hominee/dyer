//#![allow(unused_imports)]

extern crate dyer;
extern crate select;
extern crate serde;
extern crate simple_demo;
extern crate simple_logger;
extern crate tokio;

use dyer::*;
use simple_demo::entity::{Entities, Parg, Targ};
use simple_demo::middleware::{handle_profile, handle_req, handle_task};
use simple_demo::pipeline::{open_file, store_item};
use simple_demo::MySpider;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();
    let middleware = plug!( MiddleWare<Entities, Targ, Parg> {
        handle_task: handle_task,
        handle_req: handle_req,
        handle_profile: handle_profile
    });
    let pipeline = plug!( PipeLine<Entities, std::fs::File> {
        open_pipeline: open_file,
        process_entity: store_item
    } );
    let spider = MySpider::new();
    let mut app = dyer::App::<Entities, Targ, Parg>::new();
    app.run(&spider, &middleware, pipeline).await.unwrap();
}
