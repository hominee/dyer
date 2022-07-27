//! This module contains structs related to HTTP requests, notably the Request type itself as well
//! as a builder to create requests. Typically you’ll import the http::Request type rather than
//! reaching into this module itself.
//!
use crate::component::client::Client;
use crate::component::couple::Couple;
#[cfg(feature = "proxy")]
use crate::component::proxy::Proxy;
use crate::task::InnerTask;
use crate::task::MetaTask;
use crate::{
    component::{body::Body, info::Info},
    plugin::deser::*,
};
use http::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Extensions, Method, Uri, Version,
};
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt, io};

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
    #[cfg(feature = "proxy")]
    proxy: Option<Proxy>,
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
    pub Extensions,
    pub Extensions,
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
            #[cfg(feature = "proxy")]
            proxy: None,
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

    /// get shared reference to body_fn of `Request`
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
    /// let req = Request::default();
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
    /// # use dyer::request::*;
    /// let request = request::default();
    /// assert_eq!(request.info().used, 0);
    /// ```
    pub fn info(&self) -> &Info {
        &self.metar.info
    }

    /// get mutable reference to info of `Request`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::request::*;
    /// let request = request::default();
    /// request.info_mut().unique = false;
    /// assert_eq!(*request.info_ref().unique, false);
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
            #[cfg(feature = "proxy")]
            proxy: self.proxy,
        }
    }

    /// Create new `Request` directly with body, inner data (require feature `proxy` enabled)
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::request::*;
    /// let request = request::default();
    /// let ( mut inner, body, meta ) = request.into_parts();
    /// let _ = request::from_parts(inner, body, meta);
    /// ```
    pub fn from_parts(
        inner: InnerRequest,
        body: Body,
        metar: MetaRequest,
        #[cfg(feature = "proxy")] proxy: Option<Proxy>,
    ) -> Self {
        Self {
            inner,
            body,
            metar,
            #[cfg(feature = "proxy")]
            proxy,
        }
    }

    /// split `request` into body, inner data, (require feature `proxy` enabled)
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::request::*;
    /// let request = request::default();
    /// let (_inner, _body, _meta ) = request.into_parts();
    /// ```
    #[cfg(feature = "proxy")]
    pub fn into_parts(self) -> (InnerRequest, Body, MetaRequest, Option<Proxy>) {
        (self.inner, self.body, self.metar, self.proxy)
    }
    #[cfg(not(feature = "proxy"))]
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
        exts_t_fn: Option<
            &Box<
                dyn for<'c, 'd> Fn(&'c Extensions, &'d Extensions) -> (Extensions, Extensions)
                    + Send,
            >,
        >,
        exts_p_fn: Option<
            &Box<
                dyn for<'c, 'd> Fn(&'c Extensions, &'d Extensions) -> (Extensions, Extensions)
                    + Send,
            >,
        >,
    ) -> Self {
        match couple.affix {
            Some(ref affix) => {
                let item = Some(&affix.body);
                let body = match f {
                    Some(ff) => ff(&couple.task.body, item),
                    None => Body::get_merged(&couple.task.body, item),
                };
                let (inner_t, exts_t) = match exts_t_fn {
                    None => (Extensions::new(), Extensions::new()),
                    Some(ff) => ff(&couple.task.inner.extensions, &couple.task.metat.exts),
                };
                let (inner_p, exts_p) = match exts_p_fn {
                    None => (Extensions::new(), Extensions::new()),
                    Some(ff) => ff(&affix.inner.extensions, &affix.metap.exts),
                };
                let mut headers = couple.task.inner.headers.clone();
                couple
                    .affix
                    .as_ref()
                    .and_then(|item| {
                        headers.extend(item.inner.headers.clone());
                        Some(0)
                    })
                    .unwrap();
                let inner = InnerRequest {
                    uri: couple.task.inner.uri.clone(),
                    method: couple.task.inner.method.clone(),
                    version: couple.task.inner.version.clone(),
                    headers,
                    extensions: Exts(inner_t, inner_p, Extensions::new(), Extensions::new()),
                };
                let mut info = couple.task.metat.info.clone();
                info.able = f64::max(info.able, affix.metap.info.able);
                info.id = couple.id;
                let metar = MetaRequest {
                    info: info,
                    parser: couple.task.metat.parser.clone(),
                    err_parser: couple.task.metat.err_parser.clone(),
                    body_fn: None,
                    exts: Exts(exts_t, exts_p, Extensions::new(), Extensions::new()),
                };
                #[cfg(feature = "proxy")]
                let proxy = match affix.proxy {
                    Some(ref prox) => Some(prox.clone()),
                    None => match couple.task.proxy {
                        Some(ref prx) => Some(prx.clone()),
                        None => None,
                    },
                };
                Self {
                    inner,
                    body,
                    metar,
                    #[cfg(feature = "proxy")]
                    proxy,
                }
            }

            None => {
                let body = match f {
                    // concat the body with the function
                    Some(ff) => ff(&couple.task.body, None),
                    // concat the body directly, the layout:
                    // - task body + affix body
                    None => Body::get_merged(&couple.task.body, None),
                };
                let (inner_t, exts_t) = match exts_t_fn {
                    None => (Extensions::new(), Extensions::new()),
                    Some(ff) => ff(&couple.task.inner.extensions, &couple.task.metat.exts),
                };
                let inner = InnerRequest {
                    uri: couple.task.inner.uri.clone(),
                    method: couple.task.inner.method.clone(),
                    version: couple.task.inner.version,
                    headers: couple.task.inner.headers.clone(),
                    extensions: Exts(
                        inner_t,
                        Extensions::new(),
                        Extensions::new(),
                        Extensions::new(),
                    ),
                };
                let mut info = couple.task.metat.info.clone();
                info.id = couple.id;
                let metar = MetaRequest {
                    info: info,
                    parser: couple.task.metat.parser,
                    err_parser: couple.task.metat.err_parser,
                    body_fn: None,
                    exts: Exts(
                        exts_t,
                        Extensions::new(),
                        Extensions::new(),
                        Extensions::new(),
                    ),
                };
                #[cfg(feature = "proxy")]
                let proxy = match couple.task.proxy {
                    Some(ref prx) => Some(prx.clone()),
                    None => None,
                };
                Self {
                    inner,
                    body,
                    metar,
                    #[cfg(feature = "proxy")]
                    proxy,
                }
            }
        }
    }
}

impl Request {
    #[cfg_attr(docsrs, doc(cfg(feature = "proxy")))]
    /// get the unique client id that will execute the request
    pub fn get_id(&self) -> u64 {
        #[cfg(feature = "proxy")]
        match self.proxy {
            None => 0,
            Some(ref prx) => crate::utils::hash(prx),
        }
        #[cfg(not(feature = "proxy"))]
        0u64
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "proxy")))]
    /// get the client to execute the request
    pub fn get_client(&mut self) -> &'static Client {
        use crate::client::CLIENTPOOL;
        let id = self.get_id();
        unsafe {
            match CLIENTPOOL.as_ref().and_then(|pool| pool.get(&id)) {
                Some(ref downloader) => {
                    #[cfg(feature = "proxy")]
                    if let Some(ref prx) = self.proxy {
                        if let Some("http") = self.inner.uri.scheme_str() {
                            prx.auth
                                .as_ref()
                                .and_then(|au| Some(au.encode()))
                                .and_then(|token| {
                                    let val = HeaderValue::from_str(&token).unwrap();
                                    self.inner.headers.insert("Authorization", val.clone());
                                    self.inner.headers.insert("Proxy-Authorization", val);
                                    Some(())
                                });
                        }
                    }
                    downloader
                }
                None => {
                    #[cfg(feature = "proxy")]
                    match self.proxy {
                        None => {
                            // non proxy client
                            Client::new_plain()
                        }
                        Some(ref prx) => {
                            let client = prx.build();
                            if let Some("http") = self.inner.uri.scheme_str() {
                                prx.auth.as_ref().and_then(|au| Some(au.encode())).and_then(
                                    |token| {
                                        let val = HeaderValue::from_str(&token).unwrap();
                                        self.inner.headers.insert("Authorization", val.clone());
                                        self.inner.headers.insert("Proxy-Authorization", val);
                                        Some(())
                                    },
                                );
                            }
                            client
                        }
                    }
                    #[cfg(not(feature = "proxy"))]
                    Client::new_plain()
                }
            }
        }
    }
}

#[cfg(feature = "proxy")]
#[test]
fn test_get_client() {
    use super::*;
    use crate::component::client::CLIENTPOOL;
    fn parse(_: Response) -> Parsed<()> {
        todo!()
    }

    let task = Task::get("https://example.com")
        .proxy("http://127.0.0.1:1088")
        .parser(parse)
        .body(Body::empty(), "marker")
        .unwrap();
    let cp = Couple::new(task, None);
    let req = Request::from_couple(&cp, None, None, None);
    let id = req.get_client().id;
    unsafe {
        assert_eq!(CLIENTPOOL.as_ref().unwrap().len(), 1);
    }
    let task = Task::get("https://example.com")
        .parser(parse)
        .body(Body::empty(), "marker")
        .unwrap();
    let cp = Couple::new(task, None);
    let req2 = Request::from_couple(&cp, None, None, None);
    let id2 = req2.get_client().id;
    unsafe {
        assert_eq!(CLIENTPOOL.as_ref().unwrap().len(), 2);
        assert_eq!(id2, 0);
    }
    let task = Task::get("https://example.com")
        .parser(parse)
        .proxy("http://127.0.0.1:1088")
        .body(Body::empty(), "marker")
        .unwrap();
    let cp = Couple::new(task, None);
    let req3 = Request::from_couple(&cp, None, None, None);
    let id3 = req3.get_client().id;
    unsafe {
        assert_eq!(CLIENTPOOL.as_ref().unwrap().len(), 2);
    }
    assert_eq!(id, id3);
    assert!(id2 != id3);
    let task = Task::get("https://example.com")
        .parser(parse)
        .proxy("http://127.0.0.1:1080")
        .body(Body::empty(), "marker")
        .unwrap();
    let cp = Couple::new(task, None);
    let req4 = Request::from_couple(&cp, None, None, None);
    let _ = req4.get_client().id;
    unsafe {
        assert_eq!(CLIENTPOOL.as_ref().unwrap().len(), 3);
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
        self,
    ) -> (
        MetaRequest,
        hyper::Request<hyper::Body>,
        Extensions,
        Extensions,
    ) {
        let (ext_t, ext_p) = (self.inner.extensions.0, self.inner.extensions.1);
        let stream = futures_util::stream::iter(
            self.body
                .inner
                .into_iter()
                .map(|chunk| Ok::<_, io::Error>(chunk.inner())),
        );
        let mut r = hyper::Request::builder()
            .uri(self.inner.uri)
            .method(self.inner.method)
            //.header(self.inner.headers)
            .version(self.inner.version)
            .extension(self.inner.extensions.2)
            .body(hyper::body::Body::wrap_stream(stream))
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
    #[cfg(feature = "proxy")]
    proxy: Option<Proxy>,
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
            #[cfg(feature = "proxy")]
            proxy: None,
        }
    }

    /// set the uri of `request`, if not called, the default value is "/"
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::request::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let request = RequestBuilder::new()
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

    /// get shared reference to uri of `RequestBuilder`
    /// Same as `request.uri(...)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::request::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let uri = "https://example.com";
    /// let request = RequestBuilder::new()
    ///     .uri(uri)
    ///     .parser(parser_fn)
    ///     .body(());
    /// assert_eq!(task.uri_ref(), uri);
    /// ```
    pub fn uri_ref(&self) -> &Uri {
        &self.inner.uri
    }

    /// set the method of `Request`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::request::*;
    /// # use http::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let method = Method::POST;
    /// let request = RequestBuilder::new()
    ///     .method(method)
    ///     .parser(parser_fn)
    ///     .body(());
    /// assert_eq!(request.method_ref(), method);
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

    /// get shared reference to method of `RequestBuilder`
    /// Same as `request.method(...)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::request::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let method = Method::POST;
    /// let request = RequestBuilder::new()
    ///     .method(method)
    ///     .parser(parser_fn)
    ///     .body(());
    /// assert_eq!(request.method_ref(), method);
    /// ```
    pub fn method_ref(&self) -> &Method {
        &self.inner.method
    }

    /// get shared reference to header of `RequestBuilder`
    /// Same as `Request::headers(...)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::request::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let request = RequestBuilder::new()
    ///     .header("accept", "*/*")
    ///     .parser(parser_fn)
    ///     .body(());
    /// assert_eq!(task.header_ref()["accept"], "*/*");
    /// ```
    pub fn header_ref(&self) -> &HeaderMap<HeaderValue> {
        &self.inner.headers
    }

    /// get mutable reference to header of `RequestBuilder`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::request::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let request = RequestBuilder::new()
    ///     .header("accept", "*/*")
    ///     .parser(parser_fn)
    ///     .body(());
    /// task.header_mut().insert("accept", "text/html");
    /// assert_eq!(task.header_ref()["accept"], "text/html");
    /// ```
    pub fn header_mut(&mut self) -> &mut HeaderMap<HeaderValue> {
        &mut self.inner.headers
    }

    /// set the headers of `Request`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::request::*;
    /// # use http::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let request = RequestBuilder::new()
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

    /// set the version of `Request`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::request::*;
    /// # use http::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let request = RequestBuilder::new()
    ///     .version(Version::HTTP_10)
    ///     .parser(parser_fn)
    ///     .body(());
    /// assert_eq!(task.version_ref(), Version::HTTP_10);
    /// ```
    pub fn version(mut self, version: Version) -> Self {
        self.inner.version = version;
        self
    }

    /// get shared reference to version of `RequestBuilder`
    /// Same as `Request::version(...)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::request::*;
    /// # use http::Version;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let version = Version::HTTP_10;
    /// let request = RequestBuilder::new()
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
            #[cfg(feature = "proxy")]
            proxy: self.proxy,
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
    /// # fn body_fn(_: Body) -> Body { todo!() }
    /// let request = RequestBuilder::new()
    ///     .body_fn(body_fn)
    ///     .body(());
    /// assert_eq!(Request.body_fn_ref(), body_fn);
    /// ```
    pub fn body_fn(mut self, body_fn: fn(Body) -> Body) -> Self {
        let body_fn = body_fn as *const ();
        self.meta.body_fn = Some(body_fn);
        self
    }

    /// get the the body_fn of `Request`,
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::Request::*;
    /// # use http::*;
    /// # fn body_fn(_: Body) -> Body { todo!() }
    /// let request = RequestBuilder::new()
    ///     .body_fn(body_fn)
    ///     .body(());
    /// assert_eq!(Request.body_fn_ref(), body_fn);
    /// ```
    pub fn body_fn_ref(&self) -> fn(Body) -> Body {
        let ptr = self.meta.body_fn.unwrap();
        unsafe { std::mem::transmute::<*const (), fn(Body) -> Body>(ptr) }
    }

    /// change the the body_fn of `Request`,
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::Request::*;
    /// # use http::*;
    /// # fn body_fn(_: Body) -> Body { todo!() }
    /// let request = RequestBuilder::new()
    ///     .body_fn(body_fn)
    ///     .body(());
    /// assert_eq!(Request.body_fn_ref(), body_fn);
    /// ```
    pub fn body_fn_mut(&mut self, body_fn: fn(Body) -> Body) {
        let body_fn = body_fn as *const ();
        self.meta.body_fn = Some(body_fn);
    }
}
