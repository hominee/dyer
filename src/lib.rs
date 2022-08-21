#![cfg_attr(docsrs, feature(doc_cfg))]

//! [dyer] is designed for reliable, flexible and fast Request-Response based service, including data processing, web-crawling and so on, providing some friendly, interoperable, comprehensive  features without compromising speed.
//!
//! `dyer` provides some high-level features:  
//!
//! * asynchronous, lock-free, concurrent streaming and I/O, make the best of thread pool, network, and system
//! resource.
//! * Event-driven, once you set the initials and recursive generator, `dyer` will handle
//! the rest of it interoperably.
//! * User-friendly and flexible, `dyer` offers high-level, easy to use wrappers and APIs what does a lot for you.    
//!
//! ## Feature Flag
//! To reduce code redundancy and speed up compilation, dyer use feature flag to mark the necessary modules/functions, Currently here are some supported Features:
//!
//! - `xpath`: Enable parse the html response with xpath
//! - `compression`: Enable HTTP Compression: `br`, `deflate`, `gzip`
//! - `proxy`: Enable use proxies
//! - `full`: Enable all features
//!
//! **Get started** by installing [dyer-cli] and looking over the [examples].
//!
//! [dyer]: https://github.com/hominee/dyer
//! [examples]: https://github.com/hominee/dyer/tree/master/examples/
//! [dyer-cli]: https://github.com/hominee/dyer-cli
//!

pub mod component;
pub mod engine;
pub mod plugin;

#[cfg_attr(docsrs, doc(cfg(feature = "proxy")))]
#[cfg(feature = "proxy")]
#[doc(inline)]
pub use component::proxy::{self, Auth, AuthBasic, AuthBearer, AuthCustom, Proxy};
#[cfg_attr(docsrs, doc(cfg(feature = "xpath")))]
#[cfg(feature = "xpath")]
#[doc(inline)]
pub use component::xpath;
#[doc(inline)]
pub use component::{affix, body, client, couple, info, parsed, request, response, task, utils};
#[doc(inline)]
pub use component::{
    Affix, Body, Buf, Bytes, Client, ClientType, Couple, Info, MetaRequest, MetaResponse, MetaTask,
    Parsed, Request, Response, Task, CLIENTPOOL,
};
#[doc(inline)]
pub use engine::{Actor, App, ArgAffix, ArgApp, ArgRate};
#[doc(inline)]
pub use http::Extensions;
#[doc(inline)]
pub use plugin::deser::FNMAP;
#[doc(inline)]
pub use plugin::{Affixor, MiddleWare, MiddleWareBuilder, PipeLine, PipeLineBuilder};

#[doc(inline)]
pub use crate::plugin::{BoxFuture, LocalBoxFuture};
#[doc(inline)]
pub use async_trait::async_trait;
#[doc(inline)]
pub use dyer_macros::{self, actor, affix, entity, middleware, parser, pipeline};
#[doc(inline)]
pub use log;
