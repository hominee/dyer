//! The trait that produces a [Affix]
//! for extensive application. In general, it is not necessary to implement this
//! trait, most of request does not require too much, once uri is provided, the response
//! gives what you want.
//! For those that do not, the trait serves as a actor to adjust

use crate::component::{Affix, MetaResponse, Response};
use crate::Request;
use async_trait::async_trait;

/// Trait to adjust [Task] before make a request
///
/// [Task]: crate::component::task::Task
///
#[async_trait]
pub trait Affixor {
    /// Things to do before step into generating Affix
    async fn init(&mut self);

    /// Invoke an Request to generating affix
    /// if None is returned, this means affix generating does not depend on
    /// Request-Response. Then no Response or MetaResponse for `before_parse` and `parse`
    async fn invoke(&mut self) -> Option<Request>;

    /// Before executing the Request, modify the Request
    async fn after_invoke(&mut self);

    /// Before parsing affix, modify the response
    async fn before_parse(&mut self, _: Option<&mut Result<Response, MetaResponse>>);

    /// Parse the response into Affix
    async fn parse(&mut self, _: Option<Result<Response, MetaResponse>>) -> Option<Affix>;

    /// It is called only when parse is called
    /// Before collecting the parsed affix, modify the affix
    async fn after_parse(&mut self);

    /// Things to do before step out  
    async fn close(&mut self);

    /// get the marker of `Self`
    fn marker(&self) -> String {
        crate::utils::type_name(self)
    }
}

#[test]
fn test_affixor() {
    struct Res {}
    use crate::*;
    extern crate async_trait;

    #[async_trait::async_trait]
    impl Affixor for Res {
        async fn init(&mut self) {}
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

    let res = Res {};
    assert_eq!(res.marker(), "Res");
}
