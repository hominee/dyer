//! Instructions of plugins including [middleware], [pipeline] and their usage.
//!
//! # OverView
//!
//! [middleware] serve as a processor between [components], processes data in-and-out.
//!
//! [pipeline] serve as the end of the data flow, data-storage happens here.
//!
//! [components]: crate::component
//! [middleware]: crate::plugin::middleware
//! [pipeline]: crate::plugin::pipeline
//!
pub mod affixor;
pub mod deser;
pub mod middleware;
pub mod pipeline;

#[doc(inline)]
pub use affixor::*;
#[doc(inline)]
pub use deser::*;
#[doc(inline)]
pub use middleware::{MiddleWare, MiddleWareBuilder};
#[doc(inline)]
pub use pipeline::{PipeLine, PipeLineBuilder};

use std::future::Future;
use std::pin::Pin;
pub type LocalBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
