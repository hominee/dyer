//! This module contains structs related to HTTP requests, notably the Request type itself as well
//! as a builder to create requests. Typically you’ll import the http::Request type rather than
//! reaching into this module itself.
//!
use crate::component::couple::Couple;
use crate::task::InnerTask;
use crate::task::MetaTask;
use crate::{
    component::{body::Body, info::Info},
    plugin::deser::*,
};
use bytes::Buf;
use http::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Extensions, Method, Uri, Version,
};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;

/// An Wrapper of [http::Request]
///
/// An HTTP request consists of a head and a potentially optional body. The body component is
/// generic, enabling arbitrary types to represent the HTTP body. For example, the body could be
/// Vec<u8>, a Stream of byte chunks, or a value that has been deserialized.
/// Generally, `Affix` and `Task` roughly add up to a `Request`,  
#[derive(Serialize, Default, fmt::Debug, Deserialize)]
pub struct Request {
    pub inner: InnerRequest,
    pub body: Body,
    pub metar: MetaRequest,
}

/// An Wrapper of [http::request::Parts]
///
/// Component parts of an HTTP `Request`
///
/// The HTTP request head consists of a method, uri, version, and a set of
/// header fields.
#[derive(Deserialize, Default, fmt::Debug, Serialize)]
pub struct InnerRequest {
    #[serde(with = "serde_uri")]
    pub uri: Uri,
    /// request's vesoin
    #[serde(with = "serde_version")]
    pub version: Version,
    #[serde(with = "serde_method")]
    pub method: Method,
    /// additional headers if necessary
    #[serde(with = "serde_headermap")]
    pub headers: HeaderMap<HeaderValue>,
    /// additional arguments for extensive application
    #[serde(skip)]
    pub extensions: Exts,
}

/// An Wrapper of [http::Extensions]
///
/// A type map of protocol extensions.
///
/// Extensions` can be used by `Request` and `Response` to store
/// extra data derived from the underlying protocol.
#[derive(fmt::Debug, Default)]
pub struct Exts(
    pub(crate) Extensions,
    pub(crate) Extensions,
    pub Extensions,
    pub Extensions,
);

impl Exts {
    pub fn as_mut(&mut self, index: usize) -> &mut Extensions {
        match index {
            0 | 1 => unreachable!("cannot mutate the extension data for Safety!"),
            2 => &mut self.2,
            3 => &mut self.3,
            _ => unreachable!("out of range, no more than 3"),
        }
    }

    pub fn as_ref(&self, index: usize) -> &Extensions {
        match index {
            0 => &self.0,
            1 => &self.1,
            2 => &self.2,
            3 => &self.3,
            _ => unreachable!("out of range, no more than 3"),
        }
    }
}

/// Meta Data of the Request
///
/// `MetaRequest` can be used to carry infomation about the worlflow and beyond
#[derive(Deserialize, Serialize)]
pub struct MetaRequest {
    /// identifier of the entity
    pub info: Info,
    /// parsing the `Response` when it's done
    #[serde(serialize_with = "serde_fn::serfn")]
    #[serde(deserialize_with = "serde_fn::defn")]
    pub(crate) parser: *const (),
    /// parsing the `Response` when it failed
    #[serde(serialize_with = "serde_fn::serfn_op")]
    #[serde(deserialize_with = "serde_fn::defn_op")]
    pub(crate) err_parser: Option<*const ()>,
    /// convert the `Body`s from `Task` and `Affix` to make a new `Body`
    #[serde(serialize_with = "serde_fn::serfn_op")]
    #[serde(deserialize_with = "serde_fn::defn_op")]
    pub body_fn: Option<*const ()>,
    /// additional arguments for extensive application
    #[serde(skip)]
    pub exts: Exts,
}

// Safety: since *const () is a static function pointer(a usize that indicating hardware address)
// which is `Copy` so it owns the data, and no one else has it, the data can be safely transfered
// to another thread
unsafe impl Send for MetaRequest {}
unsafe impl Sync for MetaRequest {}

impl From<MetaTask> for MetaRequest {
    fn from(mut m: MetaTask) -> Self {
        m.info.used += 1;
        Self {
            info: m.info,
            parser: m.parser,
            err_parser: m.err_parser,
            body_fn: None,
            exts: Exts(
                m.exts,
                Extensions::new(),
                Extensions::new(),
                Extensions::new(),
            ),
        }
    }
}

impl From<InnerTask> for InnerRequest {
    fn from(task: InnerTask) -> Self {
        Self {
            uri: task.uri,
            headers: task.headers,
            method: task.method,
            version: task.version,
            extensions: Exts(
                task.extensions,
                Extensions::new(),
                Extensions::new(),
                Extensions::new(),
            ),
        }
    }
}

impl Default for MetaRequest {
    fn default() -> Self {
        Self {
            info: Info::default(),
            parser: 0 as *const (),
            body_fn: None,
            err_parser: None,
            exts: Exts::default(),
        }
    }
}

impl fmt::Debug for MetaRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parser = "Unknow";
        let mut err_parser = None;
        if let Some((n, _)) = serde_fn::query(None, Some(self.parser)) {
            parser = n;
        }
        if self.err_parser.is_some() {
            if let Some((n, _)) = serde_fn::query(None, self.err_parser) {
                err_parser = Some(n);
            }
        }
        f.debug_struct("MetaRequest")
            .field("info", &self.info)
            .field("parser", &parser)
            .field("err_parser", &err_parser)
            .field("exts", &self.exts)
            .finish()
    }
}

impl Request {
    /// Create an instance of `RequestBuilder` that used to build a `Request`  
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::request::*;
    /// let request = Request::default();
    ///     .body(());
    /// ```
    pub fn builder() -> RequestBuilder {
        RequestBuilder {
            inner: InnerRequest::default(),
            meta: MetaRequest::default(),
        }
    }

    // /// get shared reference to header of `RequestBuilder`
    // ///
    // /// # Examples
    // ///
    // /// ```rust
    // /// # use dyer::request::*;
    // /// let request = Request::default();
    // /// assert!(request.task().is_none());
    // /// ```
    // pub fn task(&self) -> &Task {
    //     &self.inner.task
    // }

    // /// get mutable reference to header of `RequestBuilder`
    // ///
    // /// # Examples
    // ///
    // /// ```rust
    // /// # use dyer::request::*;
    // /// let request = Request::default();
    // /// request.task_mut().unwrap().body = 3;
    // /// assert_eq!(request.task().unwrap().body, 3);
    // /// ```
    // pub fn task_mut(&mut self) -> &mut Task {
    //     self.inner.task
    // }

    // /// get shared reference to affix of `RequestBuilder`
    // ///
    // /// # Examples
    // ///
    // /// ```rust
    // /// # use dyer::request::*;
    // /// let request = Request::default();
    // /// assert!(request.affix().is_none());
    // /// ```
    // pub fn affix(&self) -> Option<&Affix> {
    //     if self.inner.affix.is_none() {
    //         return None;
    //     }
    //     self.inner.affix.as_ref()
    // }

    // /// get mutable reference to affix of `RequestBuilder`
    // ///
    // /// # Examples
    // ///
    // /// ```rust
    // /// # use dyer::request::*;
    // /// let request = Request::default();
    // /// request.affix_mut().body = Some(3);
    // /// assert_eq!(request.affix().body, Some(3));
    // /// ```
    // pub fn affix_mut(&mut self) -> Option<&mut Affix> {
    //     if self.inner.affix.is_none() {
    //         return None;
    //     }
    //     self.inner.affix.as_mut()
    // }

    /// get shared reference to body of `RequestBuilder`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::request::*;
    /// let request = Request::default();
    /// assert!(request.body().is_none());
    /// ```
    pub fn body(&self) -> &Body {
        &self.body
    }

    /// get mutable reference to body of `RequestBuilder`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::request::*;
    /// let request = Request::default();
    /// request.body_mut() = 3;
    /// assert!(request.body_mut().is_none());
    /// ```
    pub fn body_mut(&mut self) -> &mut Body {
        &mut self.body
    }

    /// get shared reference to extensions of `RequestBuilder`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::request::*;
    /// # use http::*;
    /// struct S {}
    /// let request = Request::default();
    ///     .body(());
    /// let s = S {};
    /// request.extensions_mut.insert(s);
    /// assert_eq!(request.extensions().get::<S>(), &s);
    /// ```
    pub fn extensions(&self) -> &Extensions {
        &self.inner.extensions.2
    }

    /// get mutable reference to extensions of `RequestBuilder`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::Request::*;
    /// # use http::*;
    /// let request = Request::default();
    ///     .body(());
    /// request.extensions_mut().insert(vec![1,2,3]);
    /// assert_eq!(request.extensions().get::<Vec<_>>(), &vec![1,2,3]);
    /// ```
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.inner.extensions.2
    }

    /// get shared reference to exts of `RequestBuilder`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::request::*;
    /// # use http::*;
    /// struct S {}
    /// let request = Request::default();
    ///     .body(());
    /// let s = S {};
    /// request.exts_mut.insert(s);
    /// assert_eq!(request.exts().get::<S>(), &s);
    /// ```
    pub fn exts(&self) -> &Extensions {
        &self.metar.exts.2
    }

    /// get mutable reference to exts of `RequestBuilder`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::Request::*;
    /// # use http::*;
    /// let request = Request::default();
    ///     .body(());
    /// request.exts_mut().insert(vec![1,2,3]);
    /// assert_eq!(request.exts().get::<Vec<_>>(), &vec![1,2,3]);
    /// ```
    pub fn exts_mut(&mut self) -> &mut Extensions {
        &mut self.metar.exts.2
    }

    /// get shared reference to body_fn of `Task`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::request::*;
    /// # fn body_fn(t: T, p: P) -> Body { todo!() }
    /// let reqeust = Request::default();
    ///     .body_fn(body_fn)
    ///     .body(());
    /// assert_eq!(*reqeust.body_fn(), body_fn);
    /// ```
    pub fn body_fn(&self) -> Option<fn(Body) -> Body> {
        if self.metar.body_fn.is_none() {
            return None;
        }
        let f = unsafe {
            std::mem::transmute::<*const (), fn(Body) -> Body>(self.metar.body_fn.unwrap())
        };
        Some(f)
    }

    /// set the body_fn of `Request`,
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::Request::*;
    /// # use http::*;
    /// # fn body_fn(_: Body) -> Body { todo!() }
    /// let task = Request::default();
    ///     .as_mut()
    ///     .body_fn_mut(body_fn);
    /// assert_eq!(*Request.body_fn(), body_fn);
    /// ```
    pub fn body_fn_mut(&mut self, body_fn: fn(Body) -> Body) {
        let body_fn = body_fn as *const ();
        self.metar.body_fn = Some(body_fn);
    }

    /// get shared reference to info of `Request`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// let request = request::default();
    /// assert_eq!(request.info().used, 0);
    /// ```
    pub fn info(&self) -> &Info {
        &self.metar.info
    }

    /// get mutable reference to info of `Task`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// let request = request::default();
    /// task.info_mut().unique = false;
    /// assert_eq!(*task.info_ref().unique, false);
    /// ```
    pub fn info_mut(&mut self) -> &mut Info {
        &mut self.metar.info
    }

    /// Consume the request and obtain the body
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::request::*;
    /// let request = request::default();
    /// assert!(request.into_body().is_empty());
    /// ```
    pub fn into_body(self) -> Body {
        self.body
    }

    /// Convert the body of the `request` with function
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::request::*;
    /// let request = request::default();
    /// let new= request.map(|v| v + 1 );
    /// assert_eq!(new.body, vec![2,3,4]);
    /// ```
    pub fn map<F>(self, f: F) -> Request
    where
        F: FnOnce(Body) -> Body,
    {
        Request {
            body: f(self.body),
            metar: self.metar,
            inner: self.inner,
        }
    }

    /// Create new `Request` directly with body, inner data
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::request::*;
    /// let request = request::default();
    /// let ( mut inner, body, meta ) = request.into_parts();
    /// let _ = request::from_parts(inner, body, meta);
    /// ```
    pub fn from_parts(inner: InnerRequest, body: Body, metar: MetaRequest) -> Self {
        Self { inner, body, metar }
    }

    /// split `request` into body, inner data
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::request::*;
    /// let request = request::default();
    /// let (_inner, _body, _meta ) = request.into_parts();
    /// ```
    pub fn into_parts(self) -> (InnerRequest, Body, MetaRequest) {
        (self.inner, self.body, self.metar)
    }

    /// Create new `Request` directly with Task and Affix(Optional)
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::request::*;
    /// let request = request::default();
    /// let ( mut inner, body, meta ) = request.into_parts();
    /// let _ = request::from_parts(inner, body, meta);
    /// ```
    pub fn from_couple(
        couple: &Couple,
        f: Option<&Box<dyn for<'c, 'd> Fn(&'c Body, Option<&'d Body>) -> Body + Send>>,
    ) -> Self {
        match couple.affix {
            Some(ref affix) => {
                let item = Some(&affix.body);
                let body = match f {
                    Some(ff) => ff(&couple.task.body, item),
                    None => Body::get_merged(&couple.task.body, item),
                };
                let mut ext_t = Extensions::new();
                unsafe {
                    std::ptr::copy(
                        &couple.task.inner.extensions as *const _,
                        &mut ext_t as *mut _,
                        1,
                    )
                };
                let ext_p = if couple.affix.is_none() {
                    Extensions::new()
                } else {
                    let mut ext_p = Extensions::new();
                    unsafe {
                        std::ptr::copy(
                            &couple.affix.as_ref().unwrap().metap.exts as *const _,
                            &mut ext_p as *mut _,
                            1,
                        )
                    };
                    ext_p
                };
                let inner = InnerRequest {
                    uri: couple.task.inner.uri.clone(),
                    method: couple.task.inner.method.clone(),
                    version: couple.task.inner.version.clone(),
                    headers: couple.task.inner.headers.clone(),
                    extensions: Exts(
                        //std::ptr::read(&couple.task.inner.extensions),
                        //std::ptr::read(&affix.inner.extensions),
                        ext_t,
                        ext_p,
                        Extensions::new(),
                        Extensions::new(),
                    ),
                };
                let mut info = couple.task.metat.info.clone();
                info.able = f64::max(info.able, affix.metap.info.able);
                info.id = couple.id;
                let mut ext_t = Extensions::new();
                unsafe {
                    std::ptr::copy(&couple.task.metat.exts as *const _, &mut ext_t as *mut _, 1)
                };
                let ext_p = if couple.affix.is_none() {
                    Extensions::new()
                } else {
                    let mut ext_p = Extensions::new();
                    unsafe {
                        std::ptr::copy(
                            &couple.affix.as_ref().unwrap().metap.exts as *const _,
                            &mut ext_p as *mut _,
                            1,
                        )
                    };
                    ext_p
                };
                let metar = MetaRequest {
                    info: info,
                    parser: couple.task.metat.parser.clone(),
                    err_parser: couple.task.metat.err_parser.clone(),
                    body_fn: None,
                    exts: Exts(
                        //std::ptr::read(&couple.task.metat.exts),
                        //std::ptr::read(&affix.metap.exts),
                        ext_t,
                        ext_p,
                        Extensions::new(),
                        Extensions::new(),
                    ),
                };
                Self { inner, body, metar }
            }
            None => {
                let body = match f {
                    // concat the body with the function
                    Some(ff) => ff(&couple.task.body, None),
                    // concat the body directly, the layout:
                    // - task body + affix body
                    None => Body::get_merged(&couple.task.body, None),
                };
                let mut ext = Extensions::new();
                unsafe {
                    std::ptr::copy(
                        &couple.task.inner.extensions as *const _,
                        &mut ext as *mut _,
                        1,
                    )
                };
                let inner = InnerRequest {
                    uri: couple.task.inner.uri.clone(),
                    method: couple.task.inner.method.clone(),
                    version: couple.task.inner.version,
                    headers: couple.task.inner.headers.clone(),
                    extensions: Exts(ext, Extensions::new(), Extensions::new(), Extensions::new()),
                };
                let mut info = couple.task.metat.info.clone();
                info.id = couple.id;
                let mut ext = Extensions::new();
                unsafe {
                    std::ptr::copy(&couple.task.metat.exts as *const _, &mut ext as *mut _, 1)
                };
                let metar = MetaRequest {
                    info: info,
                    parser: couple.task.metat.parser,
                    err_parser: couple.task.metat.err_parser,
                    body_fn: None,
                    exts: Exts(ext, Extensions::new(), Extensions::new(), Extensions::new()),
                };
                Self { inner, body, metar }
            }
        }
    }
}

impl
    Into<(
        MetaRequest,
        hyper::Request<hyper::Body>,
        Extensions,
        Extensions,
    )> for Request
{
    /// transform a `Request` into `hyper::Request`
    fn into(
        mut self,
    ) -> (
        MetaRequest,
        hyper::Request<hyper::Body>,
        Extensions,
        Extensions,
    ) {
        let (ext_t, ext_p) = (self.inner.extensions.0, self.inner.extensions.1);
        let mut r = hyper::Request::builder()
            .uri(self.inner.uri)
            .method(self.inner.method)
            //.header(self.inner.headers)
            .version(self.inner.version)
            .extension(self.inner.extensions.2)
            .body(hyper::body::Body::from(self.body.to_bytes()))
            .unwrap();
        *r.headers_mut() = self.inner.headers;
        (self.metar, r, ext_t, ext_p)
    }
}

/// An Wrapper of [http::request::Builder]
///
/// An HTTP request builder
///
/// This type can be used to construct an instance or `Request`
/// through a builder-like pattern.
pub struct RequestBuilder {
    inner: InnerRequest,
    meta: MetaRequest,
}

impl RequestBuilder {
    /// Create an instance of `RequestBuilder` that used to build a `Request`  
    /// Same as `Request::builder()`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::request::*;
    /// let request = RequestBuilder::new()
    ///     .body(());
    /// ```
    pub fn new() -> Self {
        Self {
            inner: InnerRequest::default(),
            meta: MetaRequest::default(),
        }
    }

    /// set the uri of `Task`, if not called, the default value is "/"
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = TaskBuilder::new()
    ///     .uri("https://example.com")
    ///     .parser(parser_fn)
    ///     .body(());
    /// ```
    pub fn uri<S>(mut self, uri: S) -> Self
    where
        Uri: TryFrom<S>,
        <Uri as TryFrom<S>>::Error: Into<http::Error>,
    {
        self.inner.uri = TryFrom::try_from(uri)
            .map_err(Into::into)
            .expect("Set Uri Failed");
        self
    }

    /// get shared reference to uri of `TaskBuilder`
    /// Same as `Task::uri(...)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let uri = "https://example.com";
    /// let task = TaskBuilder::new()
    ///     .uri(uri)
    ///     .parser(parser_fn)
    ///     .body(());
    /// assert_eq!(task.uri_ref(), uri);
    /// ```
    pub fn uri_ref(&self) -> &Uri {
        &self.inner.uri
    }

    /// set the method of `Task`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # use http::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let method = Method::POST;
    /// let task = TaskBuilder::new()
    ///     .method(method)
    ///     .parser(parser_fn)
    ///     .body(());
    /// assert_eq!(task.method_ref(), method);
    /// ```
    pub fn method<S>(mut self, method: S) -> Self
    where
        Method: TryFrom<S>,
        <Method as TryFrom<S>>::Error: Into<http::Error>,
    {
        self.inner.method = TryFrom::try_from(method)
            .map_err(Into::into)
            .expect("Set Method Failed");
        self
    }

    /// get shared reference to method of `TaskBuilder`
    /// Same as `Task::method(...)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let method = Method::POST;
    /// let task = TaskBuilder::new()
    ///     .method(method)
    ///     .parser(parser_fn)
    ///     .body(());
    /// assert_eq!(task.method_ref(), method);
    /// ```
    pub fn method_ref(&self) -> &Method {
        &self.inner.method
    }

    /// get shared reference to header of `TaskBuilder`
    /// Same as `Task::headers(...)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = TaskBuilder::new()
    ///     .header("accept", "*/*")
    ///     .parser(parser_fn)
    ///     .body(());
    /// assert_eq!(task.header_ref()["accept"], "*/*");
    /// ```
    pub fn header_ref(&self) -> &HeaderMap<HeaderValue> {
        &self.inner.headers
    }

    /// get mutable reference to header of `TaskBuilder`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = TaskBuilder::new()
    ///     .header("accept", "*/*")
    ///     .parser(parser_fn)
    ///     .body(());
    /// task.header_mut().insert("accept", "text/html");
    /// assert_eq!(task.header_ref()["accept"], "text/html");
    /// ```
    pub fn header_mut(&mut self) -> &mut HeaderMap<HeaderValue> {
        &mut self.inner.headers
    }

    /// set the headers of `Task`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # use http::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = TaskBuilder::new()
    ///     .header("accept", "*/*")
    ///     .parser(parser_fn)
    ///     .body(());
    /// assert_eq!(task.header_ref()["accept"], "*/*");
    /// ```
    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        let k: HeaderName = TryFrom::try_from(key)
            .map_err(Into::into)
            .expect("Invalid Key When Setting Header");
        let v: HeaderValue = TryFrom::try_from(value)
            .map_err(Into::into)
            .expect("Invalid Value When Setting Header");
        self.inner.headers.append(k, v);
        self
    }

    /// set the version of `Task`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # use http::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = TaskBuilder::new()
    ///     .version(Version::HTTP_10)
    ///     .parser(parser_fn)
    ///     .body(());
    /// assert_eq!(task.version_ref(), Version::HTTP_10);
    /// ```
    pub fn version(mut self, version: Version) -> Self {
        self.inner.version = version;
        self
    }

    /// get shared reference to version of `TaskBuilder`
    /// Same as `Task::version(...)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # use http::Version;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let version = Version::HTTP_10;
    /// let task = TaskBuilder::new()
    ///     .version(version)
    ///     .parser(parser_fn)
    ///     .body(());
    /// assert_eq!(task.version_ref(), version);
    /// ```
    pub fn version_ref(&self) -> &Version {
        &self.inner.version
    }

    /// Take this `RequestBuilder` and combine the body to create a `Request`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::request::*;
    /// let _ = RequestBuilder::new()
    ///     .body(());
    /// ```
    pub fn body<'r, R>(self, body: Body) -> http::Result<Request> {
        Ok(Request {
            inner: self.inner,
            metar: self.meta,
            body,
        })
    }

    /// get shared reference to extensions of `RequestBuilder`
    /// Same as `Request::extensions(...)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::request::*;
    /// # use http::*;
    /// struct S {}
    /// let request = RequestBuilder::new()
    ///     .body(());
    /// let s = S {};
    /// request.extensions_mut.insert(s);
    /// assert_eq!(request.extensions_ref().get::<S>(), &s);
    /// ```
    pub fn extensions_ref(&self) -> &Extensions {
        &self.inner.extensions.2
    }

    /// get mutable reference to extensions of `RequestBuilder`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::Request::*;
    /// # use http::*;
    /// let request = RequestBuilder::new()
    ///     .body(());
    /// request.extensions_mut().insert(vec![1,2,3]);
    /// assert_eq!(request.extensions_ref().get::<Vec<_>>(), &vec![1,2,3]);
    /// ```
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.inner.extensions.2
    }

    /// set the exts of `Request`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::Request::*;
    /// let Request = RequestBuilder::new()
    ///     .extensions(vec![1,2,3])
    ///     .body(());
    /// assert_eq!(Request.extensions_ref(), &vec![1,2,3]);
    /// ```
    pub fn extensions<S>(mut self, extensions: S) -> Self
    where
        S: std::any::Any + Send + Sync + 'static,
    {
        self.inner.extensions.2.insert(extensions);
        self
    }

    /// get shared reference to exts of `RequestBuilder`
    /// Same as `Request::exts(...)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::request::*;
    /// # use http::*;
    /// struct S {}
    /// let request = RequestBuilder::new()
    ///     .body(());
    /// let s = S {};
    /// request.exts_mut.insert(s);
    /// assert_eq!(request.exts_ref().get::<S>(), &s);
    /// ```
    pub fn exts_ref(&self) -> &Extensions {
        &self.meta.exts.2
    }

    /// get mutable reference to exts of `RequestBuilder`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::Request::*;
    /// # use http::*;
    /// let request = RequestBuilder::new()
    ///     .body(());
    /// request.exts_mut().insert(vec![1,2,3]);
    /// assert_eq!(request.exts_ref().get::<Vec<_>>(), &vec![1,2,3]);
    /// ```
    pub fn exts_mut(&mut self) -> &mut Extensions {
        &mut self.meta.exts.2
    }

    /// set the exts of `Request`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::Request::*;
    /// let Request = RequestBuilder::new()
    ///     .exts(vec![1,2,3])
    ///     .body(());
    /// assert_eq!(Request.exts_ref(), &vec![1,2,3]);
    /// ```
    pub fn exts<S>(mut self, exts: S) -> Self
    where
        S: std::any::Any + Send + Sync + 'static,
    {
        self.meta.exts.2.insert(exts);
        self
    }

    /// set the body_fn of `Request`,
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::Request::*;
    /// # use http::*;
    /// # fn body_fn(t: T, p: P) -> Body { todo!() }
    /// let task = RequestBuilder::new()
    ///     .body_fn(body_fn)
    ///     .body(());
    /// assert_eq!(*Request.body_fn_ref(), body_fn);
    /// ```
    pub fn body_fn(&mut self, body_fn: fn(Body) -> Body) -> &mut Self {
        let body_fn = body_fn as *const ();
        self.meta.body_fn = Some(body_fn);
        self
    }
}
