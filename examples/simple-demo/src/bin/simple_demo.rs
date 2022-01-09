use dyer::plugin::pipeline::PipeLine;
use dyer::*;
use simple_demo::entity::Entities;
use simple_demo::middleware::{handle_affix, handle_req, handle_task};
use simple_demo::pipeline::*;
use simple_demo::MyActor;

#[tokio::main]
async fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();
    let middleware = dyer::MiddleWare::builder()
        .task(&handle_task)
        .req(&handle_req)
        .affix(&handle_affix)
        .build("quote".into());
    let pipeline = PipeLine::<Entities, _>::builder()
        //.initializer(&open_file)  // the normal way
        .initializer(&opener::<Entities>) // also you can specify generic type here
        .entity(&store_item)
        .build("quote".into());
    let mut actor = MyActor::new().await;
    let mut app = dyer::App::<Entities>::new();
    app.run(&mut actor, &middleware, &pipeline).await.unwrap();
}
