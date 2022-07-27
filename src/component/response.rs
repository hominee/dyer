//! HTTP response types.
//!
//! This module contains structs related to HTTP responses, notably the
//! `Response` type itself as well as a builder to create responses. Typically
//! you'll import the `http::Response` type rather than reaching into this
//! module itself.
//!
use crate::component::{body::Body, info::Info, request::MetaRequest, utils};
use crate::plugin::deser::*;
use crate::request::Exts;
use http::{header::HeaderName, Extensions, HeaderMap, HeaderValue, StatusCode, Version};
#[cfg(feature = "xpath")]
use libxml::{tree::Document, xpath::Context};
use std::{convert::TryFrom, fmt};

/// An Wrapper of [http::Response]
///
/// Represents an HTTP response
///
/// An HTTP response consists of a head and a potentially optional body. The body component is
/// generic, enabling arbitrary types to represent the HTTP body. For example, the body could be
/// Vec<u8>, a Stream of byte chunks, or a value that has been deserialized.
///
/// Typically you’ll work with responses on the client side as the result of sending a Request and
/// on the server you’ll be generating a Response to send back to the client.
#[derive(Default)]
pub struct Response {
    /// inner parts of Response
    pub inner: InnerResponse,
    /// body
    pub body: Body,
    /// the metadata of the response
    pub metas: MetaResponse,
    /// xpath related dom and context
    #[cfg(feature = "xpath")]
    pub(crate) context: (Option<Document>, Option<Context>),
}

/// Safety:
/// the safety of InnerResponse and MetaResponse is addressed repectively,
/// the body is obviously Send and Sync
/// xpath related dom and context can only be called when Response
/// is successful and step into parse
/// Since we parse the response's body in main thread, it should be Send and Sync
unsafe impl Send for Response {}
unsafe impl Sync for Response {}

/// An Wrapper of [http::response::Parts]
///
/// Component parts of an HTTP `Response`
/// ///
/// /// The HTTP response head consists of a status, version, and a set of
/// /// header fields.
#[derive(Default)]
pub struct InnerResponse {
    /// The response's status
    pub status: StatusCode,
    /// The response's version
    pub version: Version,
    /// The response's headers
    pub headers: HeaderMap<HeaderValue>,
    /// additional arguments for extensive application
    pub extensions: Exts,
}

/// Meta Data of the Response
///
/// `MetaResponse` can be used to carry infomation about the worlflow and beyond
pub struct MetaResponse {
    /// identifier of the entity
    pub info: Info,
    /// parsing the `Response` when it's done
    pub parser: *const (),
    /// parsing the `Response` when it failed
    pub err_parser: Option<*const ()>,
    /// convert the `Body`s from `Task` and `Affix` to make a new `Body`
    pub body_fn: Option<*const ()>,
    /// Whether a redirection happens or not
    pub redirected: bool,
    /// additional arguments for extensive application
    pub exts: Exts,
}

// Safety: since *const () is a static function pointer(a usize that indicating hardware address)
// which is `Copy` so it owns the data, and no one else has it, the data can be safely transfered
// to another thread
unsafe impl Send for MetaResponse {}
unsafe impl Sync for MetaResponse {}

impl Default for MetaResponse {
    fn default() -> Self {
        Self {
            info: Info::default(),
            parser: 0 as *const (),
            body_fn: None,
            err_parser: None,
            redirected: false,
            exts: Exts::default(),
        }
    }
}

impl fmt::Debug for MetaResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parser = "Unknow";
        let mut err_parser = None;
        let mut body_fn = None;
        if let Some((n, _)) = serde_fn::query(None, Some(self.parser)) {
            parser = n;
        }
        if self.err_parser.is_some() {
            if let Some((n, _)) = serde_fn::query(None, self.err_parser) {
                err_parser = Some(n);
            }
        }
        if self.err_parser.is_some() {
            if let Some((n, _)) = serde_fn::query(None, self.body_fn) {
                body_fn = Some(n);
            }
        }
        f.debug_struct("MetaTask")
            .field("info", &self.info)
            .field("parser", &parser)
            .field("err_parser", &err_parser)
            .field("body_fn", &body_fn)
            .field("redirected", &self.redirected)
            .field("exts", &self.exts)
            .finish()
    }
}

impl From<MetaRequest> for MetaResponse {
    fn from(mut m: MetaRequest) -> Self {
        m.info.created = utils::now();
        MetaResponse {
            info: m.info,
            parser: m.parser,
            err_parser: m.err_parser,
            body_fn: m.body_fn,
            redirected: false,
            exts: m.exts,
        }
    }
}

/// An Wrapper of [http::response::Builder]
///
/// An HTTP response builder
/// ///
/// /// This type can be used to construct an instance of `Response` through a
/// /// builder-like pattern.
#[derive(Default)]
pub struct ResponseBuilder {
    /// Inner parts
    pub inner: InnerResponse,
    /// The meta-response's extensions
    pub meta: MetaResponse,
}

impl Response {
    /// Creates a new builder-style object to manufacture a `Response`
    ///
    /// This method returns an instance of `ResponseBuilder` which can be used to
    /// create a `Response`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dyer::response::*;
    /// let response = Response::builder()
    ///     .status(200)
    ///     .header("accept", "*/*")
    ///     .body(Body::empty())
    ///     .unwrap();
    /// ```
    #[inline]
    pub fn builder() -> ResponseBuilder {
        ResponseBuilder::new()
    }

    /// Creates a new blank `Response` with the body
    ///
    /// The component ports of this response will be set to their default, e.g.
    /// the ok status, no headers, etc.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dyer::response::*;
    /// let response = Response::new("hello world");
    ///
    /// assert_eq!(response.status(), StatusCode::OK);
    /// assert_eq!(response.body(), &Body::from("hello world"));
    /// ```
    #[inline]
    pub fn new<T>(body: T) -> Response
    where
        Body: From<T>,
    {
        Response {
            inner: InnerResponse::default(),
            body: Body::from(body),
            metas: MetaResponse::default(),
            #[cfg(feature = "xpath")]
            context: (None, None),
        }
    }

    /// Creates a new `Response` with the given head and body
    ///
    /// # Examples
    ///
    /// ```
    /// # use dyer::response::*;
    /// let response = Response::new("hello world");
    /// let (mut inner, body, meta) = response.into_parts();
    ///
    /// inner.status = StatusCode::BAD_REQUEST;
    /// let response = Response::from_parts(inner, body, meta);
    ///
    /// assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    /// ```
    #[inline]
    pub fn from_parts(inner: InnerResponse, body: Body, meta: MetaResponse) -> Response {
        Response {
            inner,
            body: body,
            metas: meta,
            #[cfg(feature = "xpath")]
            context: (None, None),
        }
    }

    /// Returns the `StatusCode`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dyer::response::*;
    /// let response: Response<()> = Response::default();
    /// assert_eq!(response.status(), StatusCode::OK);
    /// ```
    #[inline]
    pub fn status(&self) -> StatusCode {
        self.inner.status
    }

    /// Returns a mutable reference to the associated `StatusCode`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dyer::response::*;
    /// let mut response = Response::default();
    /// *response.status_mut() = StatusCode::CREATED;
    /// assert_eq!(response.status(), StatusCode::CREATED);
    /// ```
    #[inline]
    pub fn status_mut(&mut self) -> &mut StatusCode {
        &mut self.inner.status
    }

    /// Returns a reference to the associated version.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dyer::response::*;
    /// let response: Response = Response::default();
    /// assert_eq!(response.version(), Version::HTTP_11);
    /// ```
    #[inline]
    pub fn version(&self) -> Version {
        self.inner.version
    }

    /// Returns a mutable reference to the associated version.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dyer::response::*;
    /// let mut response: Response = Response::default();
    /// *response.version_mut() = Version::HTTP_2;
    /// assert_eq!(response.version(), Version::HTTP_2);
    /// ```
    #[inline]
    pub fn version_mut(&mut self) -> &mut Version {
        &mut self.inner.version
    }

    /// Get a share reference to the header of HeaderMap.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dyer::response::*;
    /// let response: Response = Response::default();
    /// assert!(response.headers().is_empty());
    /// ```
    #[inline]
    pub fn headers(&self) -> &HeaderMap<HeaderValue> {
        &self.inner.headers
    }

    /// Returns a mutable reference to the associated header field map.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dyer::response::*;
    /// # use http::header::*;
    /// let mut response: Response = Response::default();
    /// response.headers_mut().insert(HOST, HeaderValue::from_static("world"));
    /// assert!(!response.headers().is_empty());
    /// ```
    #[inline]
    pub fn headers_mut(&mut self) -> &mut HeaderMap<HeaderValue> {
        &mut self.inner.headers
    }

    /// Returns a reference to the associated extensions.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dyer::response::*;
    /// let response: Response = Response::default();
    /// assert!(response.extensions().get::<i32>().is_none());
    /// ```
    #[inline]
    pub fn extensions(&self) -> &Extensions {
        &self.inner.extensions.3
    }

    /// Returns a mutable reference to the associated extensions.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dyer::response::*;
    /// # use http::header::*;
    /// let mut response: Response<()> = Response::default();
    /// response.extensions_mut().insert("hello");
    /// assert_eq!(response.extensions().get::<&'static str>(), Some(&"hello"));
    /// ```
    #[inline]
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.inner.extensions.3
    }

    /// Returns a reference to the associated extensions of metadata.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dyer::response::*;
    /// let response: Response<()> = Response::default();
    /// assert!(response.exts().get::<i32>().is_none());
    /// ```
    #[inline]
    pub fn exts(&self) -> &Extensions {
        &self.metas.exts.3
    }

    /// Returns a mutable reference to the associated extensions of metadata.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dyer::response::*;
    /// # use http::header::*;
    /// let mut response: Response<()> = Response::default();
    /// response.extensions_mut().insert("hello");
    /// assert_eq!(response.exts().get(), Some(&"hello"));
    /// ```
    #[inline]
    pub fn exts_mut(&mut self) -> &mut Extensions {
        &mut self.metas.exts.3
    }

    /// Returns a reference to the associated HTTP body.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dyer::response::*;
    /// let response: Response = Response::default();
    /// assert!(response.body().is_empty());
    /// ```
    #[inline]
    pub fn body(&self) -> &Body {
        &self.body
    }

    /// Returns a mutable reference to the associated HTTP body.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dyer::response::*;
    /// let mut response: Response<String> = Response::default();
    /// response.body_mut() = Body::from("hello world");
    /// assert!(!response.body().is_empty());
    /// ```
    #[inline]
    pub fn body_mut(&mut self) -> &mut Body {
        &mut self.body
    }

    /// Consumes the response, returning just the body.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dyer::response::Response;
    /// let response = Response::new(10);
    /// let body = response.into_body();
    /// assert_eq!(body, Body::from(10));
    /// ```
    #[inline]
    pub fn into_body(self) -> Body {
        self.body
    }

    /// Consumes the response returning the head, metadata and body parts.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dyer::response::*;
    /// let response: Response = Response::default();
    /// let (parts, body, meta) = response.into_parts();
    /// assert_eq!(parts.status, StatusCode::OK);
    /// ```
    #[inline]
    pub fn into_parts(self) -> (InnerResponse, Body, MetaResponse) {
        (self.inner, self.body, self.metas)
    }

    /// Consumes the response returning a new response with body mapped to the
    /// return type of the passed in function.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dyer::response::*;
    /// let response = Response::builder().body("some string").unwrap();
    /// let mapped_response: Response = response.map(|b| {
    ///   Body::from(b.bytes())
    /// });
    /// assert_eq!(mapped_response.body().bytes(), "some string".as_bytes());
    /// ```
    #[inline]
    pub fn map<F>(self, f: F) -> Response
    where
        F: FnOnce(Body) -> Body,
    {
        Response {
            body: f(self.body),
            inner: self.inner,
            metas: self.metas,
            #[cfg(feature = "xpath")]
            context: self.context,
        }
    }
}

impl ResponseBuilder {
    /// Creates a new default instance of `ResponseBuilder` to construct either a
    /// `Head` or a `Response`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dyer::response::*;
    /// let response = response::ResponseBuilder::new()
    ///     .status(200)
    ///     .body(Body::empty())
    ///     .unwrap();
    /// ```
    #[inline]
    pub fn new() -> ResponseBuilder {
        ResponseBuilder::default()
    }

    /// Set the HTTP status for this response.
    ///
    /// This function will configure the HTTP status code of the `Response` that
    /// will be returned from `ResponseBuilder::build`.
    ///
    /// By default this is `200`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dyer::response::*;
    ///
    /// let response = Response::builder()
    ///     .status(200)
    ///     .body(Body::empty())
    ///     .unwrap();
    /// ```
    pub fn status<T>(mut self, status: T) -> ResponseBuilder
    where
        StatusCode: TryFrom<T>,
        <StatusCode as TryFrom<T>>::Error: Into<http::Error>,
    {
        self.inner.status = TryFrom::try_from(status).map_err(Into::into).unwrap();
        self
    }

    /// Set the HTTP version for this response.
    ///
    /// This function will configure the HTTP version of the `Response` that
    /// will be returned from `ResponseBuilder::build`.
    ///
    /// By default this is HTTP/1.1
    ///
    /// # Examples
    ///
    /// ```
    /// # use dyer::response::*;
    ///
    /// let response = Response::builder()
    ///     .version(Version::HTTP_2)
    ///     .body(Body::empty())
    ///     .unwrap();
    /// ```
    pub fn version(mut self, version: Version) -> ResponseBuilder {
        self.inner.version = version;
        self
    }

    /// Appends a header to this response builder.
    ///
    /// This function will append the provided key/value as a header to the
    /// internal `HeaderMap` being constructed. Essentially this is equivalent
    /// to calling `HeaderMap::append`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dyer::response::*;
    /// # use http::header::HeaderValue;
    ///
    /// let response = Response::builder()
    ///     .header("Content-Type", "text/html")
    ///     .header("X-Custom-Foo", "bar")
    ///     .header("content-length", 0)
    ///     .body(Body::empty())
    ///     .unwrap();
    /// ```
    pub fn header<K, V>(mut self, key: K, value: V) -> ResponseBuilder
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        let name = <HeaderName as TryFrom<K>>::try_from(key)
            .map_err(Into::into)
            .unwrap();
        let value = <HeaderValue as TryFrom<V>>::try_from(value)
            .map_err(Into::into)
            .unwrap();
        self.inner.headers.append(name, value);
        self
    }

    /// Get header on this response builder.
    ///
    /// When builder has error returns None.
    ///
    /// # Example
    ///
    /// ```
    /// # use dyer::response::Response;
    /// # use http::header::HeaderValue;
    /// let res = Response::builder()
    ///     .header("Accept", "text/html")
    ///     .header("X-Custom-Foo", "bar");
    /// let headers = res.headers_ref().unwrap();
    /// assert_eq!( headers["Accept"], "text/html" );
    /// assert_eq!( headers["X-Custom-Foo"], "bar" );
    /// ```
    pub fn headers_ref(&self) -> &HeaderMap<HeaderValue> {
        &self.inner.headers
    }

    /// Get header on this response builder.
    /// when builder has error returns None
    ///
    /// # Example
    ///
    /// ```
    /// # use dyer::response::*;
    /// # use dyer::response::header::HeaderValue;
    /// # use dyer::response::response::ResponseBuilder;
    /// let mut res = Response::builder();
    /// {
    ///   let headers = res.headers_mut().unwrap();
    ///   headers.insert("Accept", HeaderValue::from_static("text/html"));
    ///   headers.insert("X-Custom-Foo", HeaderValue::from_static("bar"));
    /// }
    /// let headers = res.headers_ref().unwrap();
    /// assert_eq!( headers["Accept"], "text/html" );
    /// assert_eq!( headers["X-Custom-Foo"], "bar" );
    /// ```
    pub fn headers_mut(&mut self) -> &mut HeaderMap<HeaderValue> {
        &mut self.inner.headers
    }

    /// Adds an extension to this builder
    ///
    /// # Examples
    ///
    /// ```
    /// # use dyer::response::*;
    ///
    /// let response = Response::builder()
    ///     .extension("My Extension")
    ///     .body(Body::empty())
    ///     .unwrap();
    ///
    /// assert_eq!(response.extensions().get::<&'static str>(),
    ///            Some(&"My Extension"));
    /// ```
    pub fn extension<S>(mut self, extension: S) -> ResponseBuilder
    where
        S: std::any::Any + Send + Sync + 'static,
    {
        self.inner.extensions.3.insert(extension);
        self
    }

    /// Get a reference to the extensions for this response builder.
    ///
    /// If the builder has an error, this returns `None`.
    ///
    /// # Example
    ///
    /// ```
    /// # use dyer::response::Response;
    /// let res = Response::builder().extension(5u32);
    /// let extensions = res.extensions_ref().unwrap();
    /// assert_eq!(extensions.get::<u32>(), &5u32);
    /// ```
    pub fn extensions_ref(&self) -> &Extensions {
        &self.inner.extensions.3
    }

    /// Get a mutable reference to the extensions for this response builder.
    ///
    /// If the builder has an error, this returns `None`.
    ///
    /// # Example
    ///
    /// ```
    /// # use dyer::response::Response;
    /// let res = Response::builder().extension(5u32);
    /// let mut extensions = res.extensions_mut().unwrap();
    /// assert_eq!(extensions.get::<u32>(), Some(&5u32));
    /// extensions.insert(3u32);
    /// assert_eq!(extensions.get::<u32>(), Some(&3u32));
    /// ```
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.inner.extensions.3
    }

    /// Adds an extension to this builder
    ///
    /// # Examples
    ///
    /// ```
    /// # use dyer::response::*;
    /// let response = Response::builder()
    ///     .exts(1i32)
    ///     .body(Body::empty())
    ///     .unwrap();
    /// ```
    #[inline]
    pub fn exts<S>(mut self, extension: S) -> Self
    where
        S: std::any::Any + Send + Sync + 'static,
    {
        self.meta.exts.3.insert(extension);
        self
    }

    /// Get a mutable reference to the extensions for this response builder.
    ///
    /// If the builder has an error, this returns `None`.
    ///
    /// # Example
    ///
    /// ```
    /// # use dyer::response::Response;
    /// let res = Response::builder().extension(5u32);
    /// let mut exts = res.exts_mut().unwrap();
    /// assert_eq!(exts.get::<u32>(), Some(&5u32));
    /// exts.insert(3u32);
    /// assert_eq!(exts.get::<u32>(), Some(&3u32));
    /// ```
    #[inline]
    pub fn exts_mut(&mut self) -> &mut Extensions {
        &mut self.meta.exts.3
    }

    /// Get a reference to the extensions for the meta data of response builder.
    ///
    /// # Example
    ///
    /// ```
    /// # use dyer::response::Response;
    /// let res = Response::builder().extension(5u32);
    /// let exts = res.exts_ref().unwrap();
    /// assert_eq!(exts.get::<u32>(), &5u32);
    /// ```
    pub fn exts_ref(&self) -> &Extensions {
        &self.meta.exts.3
    }

    /// Consumes this builder, using the provided `body` to return a
    /// constructed `Response`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dyer::response::*;
    ///
    /// let response = Response::builder()
    ///     .body(Body::empty())
    ///     .unwrap();
    /// ```
    pub fn body<T>(self, body: T) -> Response
    where
        Body: From<T>,
    {
        Response {
            inner: self.inner,
            body: Body::from(body),
            metas: self.meta,
            #[cfg(feature = "xpath")]
            context: (None, None),
        }
    }
}
