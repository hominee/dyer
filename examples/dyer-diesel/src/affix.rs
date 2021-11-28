use dyer::plugin::Affixor;
use dyer::response::MetaResponse;
use dyer::*;

pub struct Aff {}

#[dyer::async_trait]
impl Affixor for Aff {
    async fn init(&mut self) {
        println!("act init");
    }
    async fn invoke(&mut self) -> Option<Request> {
        None
    }
    async fn after_invoke(&mut self) {}
    async fn before_parse(&mut self, _: Option<&mut Result<Response, MetaResponse>>) {}
    async fn parse(&mut self, _: Option<Result<Response, MetaResponse>>) -> Option<Affix> {
        None
    }
    async fn after_parse(&mut self) {}
    async fn close(&mut self) {}
}
