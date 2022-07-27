extern crate dyer;
extern crate serde;
extern crate simple_logger;
extern crate template;
extern crate tokio;

use dyer::*;
use template::entity::*;
use template::middleware::*;
use template::pipeline::*;
use template::MyActor;

#[tokio::main]
async fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();
    let middleware = MiddleWare::<Entities>::builder()
        .entity(&handle_entities)
        // Identifier of the middleware
        .build("MyActor".into());
    let pipeline = PipeLine::<Entities, std::fs::File>::builder()
        .initializer(&func_name)
        // Identifier of the pipeline
        .build("MyActor".into());
    let mut actor = MyActor::new().await;
    let mut app = dyer::App::<Entities>::new();
    app.run(&mut actor, &middleware, &pipeline).await.unwrap();
}
