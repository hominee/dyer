//! Instructions of components including [client], [affix], [request], [response], [task], [utils].
//!
//! # OverView
//!
//! `client` contains some methods to execute `Request` as Future;
//!
//! `utils` is a collection of useful tools like [hash] get now time stamp [now] and other stuff;
//!
//! the others, as the file name suggests, serve as component in integration
//!
//! [hash]: crate::component::utils::hash
//! [now]: crate::component::utils::now
//! [client]: crate::component::client
//! [affix]: crate::component::affix
//! [request]: crate::component::request
//! [response]: crate::component::response
//! [task]: crate::component::task
//! [utils]: crate::component::utils
//!
pub mod affix;
pub mod body;
pub mod client;
pub mod couple;
pub mod info;
pub mod parsed;
#[cfg_attr(docsrs, doc(cfg(feature = "proxy")))]
#[cfg(feature = "proxy")]
pub mod proxy;
pub mod request;
pub mod response;
pub mod task;
pub mod utils;
#[cfg_attr(docsrs, doc(cfg(feature = "xpath")))]
#[cfg(feature = "xpath")]
pub mod xpath;

use std::convert::TryInto;

/// fundamental data struct
pub enum Poly {
    Task(Task),
    Affix(Affix),
    Request(Request),
    Couple(Couple),
    Response(Response),
}

impl From<Task> for Poly {
    fn from(task: Task) -> Self {
        Self::Task(task)
    }
}
impl TryInto<Task> for Poly {
    type Error = std::io::Error;

    fn try_into(self) -> Result<Task, Self::Error> {
        if let Poly::Task(task) = self {
            Ok(task)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid Cast to Task",
            ))
        }
    }
}

impl From<Affix> for Poly {
    fn from(affix: Affix) -> Self {
        Self::Affix(affix)
    }
}
impl TryInto<Affix> for Poly {
    type Error = std::io::Error;

    fn try_into(self) -> Result<Affix, Self::Error> {
        if let Poly::Affix(affix) = self {
            Ok(affix)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid Cast to Affix",
            ))
        }
    }
}

impl From<Request> for Poly {
    fn from(request: Request) -> Self {
        Self::Request(request)
    }
}
impl TryInto<Request> for Poly {
    type Error = std::io::Error;

    fn try_into(self) -> Result<Request, Self::Error> {
        if let Poly::Request(request) = self {
            Ok(request)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid Cast to Request",
            ))
        }
    }
}

impl From<(u64, Couple)> for Poly {
    fn from(couple: (u64, Couple)) -> Self {
        Self::Couple(couple.1)
    }
}
impl TryInto<(u64, Couple)> for Poly {
    type Error = std::io::Error;

    fn try_into(self) -> Result<(u64, Couple), Self::Error> {
        if let Poly::Couple(couple) = self {
            Ok((couple.id, couple))
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid Cast to Couple",
            ))
        }
    }
}

impl From<Response> for Poly {
    fn from(response: Response) -> Self {
        Self::Response(response)
    }
}
impl TryInto<Response> for Poly {
    type Error = std::io::Error;

    fn try_into(self) -> Result<Response, Self::Error> {
        if let Poly::Response(response) = self {
            Ok(response)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid Cast to Response",
            ))
        }
    }
}

#[doc(hidden)]
pub use affix::Affix;
#[doc(hidden)]
pub use body::{Body, Chunk, Kind};
#[doc(hidden)]
pub use client::{Client, ClientType, CLIENTPOOL};
#[doc(hidden)]
pub use couple::Couple;
pub use hyper::body::{Buf, Bytes};
#[doc(hidden)]
pub use info::Info;
#[doc(hidden)]
pub use parsed::Parsed;
#[cfg_attr(docsrs, doc(cfg(feature = "proxy")))]
#[cfg(feature = "proxy")]
#[doc(hidden)]
pub use proxy::{Auth, AuthBasic, AuthBearer, AuthCustom, Proxy};
#[doc(hidden)]
pub use request::{Exts, InnerRequest, MetaRequest, Request, RequestBuilder};
#[doc(hidden)]
pub use response::{InnerResponse, MetaResponse, Response, ResponseBuilder};
#[doc(hidden)]
pub use task::{InnerTask, MetaTask, Task, TaskBuilder};
