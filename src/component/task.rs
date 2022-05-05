//! This module contains structs related to HTTP requests,
//!
//! notably the Request type itself as well
//! as a builder to create requests. Typically you’ll import the http::Request type rather than
//! reaching into this module itself.
//!
use crate::plugin::deser::*;
use crate::utils;
use crate::{
    component::{Body, Info, Parsed},
    Response,
};
use http::{
    header::HeaderName, method::Method, version::Version, Extensions, HeaderMap, HeaderValue, Uri,
};
use serde::{Deserialize, Serialize};
use std::collections::{hash_map::DefaultHasher, BinaryHeap};
use std::hash::{Hash, Hasher};

use std::convert::TryFrom;
use std::{fmt, mem};

/// An `Task` consists of a head and a potentially optional body. The body component is
/// generic, enabling arbitrary types to represent the HTTP body. For example, the body could be
/// Vec<u8>, a Stream of byte chunks, or a value that has been deserialized.
#[derive(Deserialize, Default, Serialize)]
pub struct Task {
    /// main infomation that represents a `Task`
    pub(crate) inner: InnerTask,
    /// Formdata, files or other request parameters stored here
    pub(crate) body: Body,
    // TODO: add metadata that controls this `Task`
    //pub meta: Meta,
    /// some metadata about this Task,
    pub(crate) metat: MetaTask,
}

impl fmt::Debug for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Task")
            .field("uri", self.uri())
            .field("method", self.method())
            .field("version", &self.version())
            .field("headers", &self.headers())
            .field("body", &self.body())
            .field("metat", &self.metat)
            .finish()
    }
}

/// An Wrapper of [http::request::Parts]
///
/// Component parts of an HTTP `Request`
///
/// The HTTP request head consists of a method, uri, version, and a set of
/// header fields.
#[derive(Deserialize, Default, fmt::Debug, Serialize)]
pub struct InnerTask {
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
    pub extensions: Extensions,
}

impl Hash for InnerTask {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut heap: BinaryHeap<(&str, &str)> = BinaryHeap::new();
        self.uri.hash(state);
        self.method.hash(state);
        self.headers.iter().for_each(|(k, v)| {
            heap.push((k.as_str(), v.to_str().unwrap()));
        });
        while let Some((k, v)) = heap.pop() {
            k.hash(state);
            v.hash(state);
        }
    }
}

// Safety: since *const () is a static function pointer(a usize that indicating hardware address)
// which is `Copy` so it owns the data, and no one else has it, the data can be safely transfered
// to another thread
unsafe impl Send for Task {}
unsafe impl Sync for Task {}

impl InnerTask {
    pub fn new() -> Self {
        Self {
            uri: Uri::default(),
            method: Method::default(),
            version: Version::default(),
            headers: HeaderMap::default(),
            extensions: Extensions::default(),
        }
    }
}

/// Meta Data of the Task
///
/// `MetaTask` can be used to carry infomation about the worlflow and beyond
#[derive(Deserialize, Serialize)]
pub struct MetaTask {
    /// info about this task
    pub info: Info,
    /// parsing the `Response` when it's done
    #[serde(serialize_with = "serde_fn::serfn")]
    #[serde(deserialize_with = "serde_fn::defn")]
    pub(crate) parser: *const (),
    /// parsing the `Response` when it failed
    #[serde(serialize_with = "serde_fn::serfn_op")]
    #[serde(deserialize_with = "serde_fn::defn_op")]
    pub(crate) err_parser: Option<*const ()>,
    /// additional arguments for extensive application
    #[serde(skip)]
    pub exts: Extensions,
}

impl Hash for MetaTask {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.info.used.hash(state);
        self.info.marker.hash(state);
        self.parser.hash(state);
        self.err_parser.hash(state);
    }
}

impl MetaTask {
    pub fn new() -> Self {
        Self {
            info: Info::default(),
            parser: 0 as *const (),
            err_parser: None,
            exts: Extensions::new(),
        }
    }
}

impl fmt::Debug for MetaTask {
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
        f.debug_struct("MetaTask")
            .field("info", &self.info)
            .field("parser", &parser)
            .field("err_parser", &err_parser)
            .field("exts", &self.exts)
            .finish()
    }
}

impl Default for MetaTask {
    fn default() -> Self {
        Self {
            info: Info::default(),
            parser: 0 as *const (),
            err_parser: None,
            exts: Extensions::new(),
        }
    }
}

impl Task {
    /// create an uninitalized Task instance
    /// without set the parser as well
    /// NOTE valid only in this crate when recycling failed response
    /*
     *pub(crate) fn new_uninit() -> mem::MaybeUninit<Task> {
     *    let task = Task {
     *        inner: InnerTask::new(),
     *        body: Body::empty(),
     *        metat: MetaTask::new(),
     *    };
     *    mem::MaybeUninit::new(task)
     *}
     */

    /// Create an instance of `TaskBuilder` that used to build a `Task`  
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = Task::builder()
    ///     .method("GET")
    ///     .uri("https://example.com/")
    ///     .header("accept", "*/*")
    ///     .parser(parser_fn)
    ///     .body(())
    ///     .unwrap();
    /// ```
    pub fn builder() -> TaskBuilder {
        TaskBuilder::new()
    }
}

impl Task {
    /// creates a new `TaskBuilder` initialized with a POST method and `URI`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = Task::get("https://example.com/")
    ///     .parser(parser_fn)
    ///     .body(())
    ///     .unwrap();
    /// ```
    pub fn get<'a, S>(uri: S) -> TaskBuilder
    where
        Uri: TryFrom<S>,
        <Uri as TryFrom<S>>::Error: Into<http::Error>,
    {
        TaskBuilder::new().method(Method::GET).uri(uri)
    }

    /// creates a new `TaskBuilder` initialized with a POST method and `URI`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = Task::post("https://example.com/")
    ///     .parser(parser_fn)
    ///     .body(());
    /// ```
    pub fn post<'a, S>(uri: S) -> TaskBuilder
    where
        Uri: TryFrom<S>,
        <Uri as TryFrom<S>>::Error: Into<http::Error>,
    {
        TaskBuilder::new().method(Method::POST).uri(uri)
    }

    /// creates a new `TaskBuilder` initialized with a PUT method and `URI`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = Task::put("https://example.com/")
    ///     .parser(parser_fn)
    ///     .body(());
    /// ```
    pub fn put<'a, S>(uri: S) -> TaskBuilder
    where
        Uri: TryFrom<S>,
        <Uri as TryFrom<S>>::Error: Into<http::Error>,
    {
        TaskBuilder::new().method(Method::PUT).uri(uri)
    }

    /// creates a new `TaskBuilder` initialized with a CONNECT method and `URI`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = Task::connect("https://example.com/")
    ///     .parser(parser_fn)
    ///     .body(());
    /// ```
    pub fn connect<'a, S>(uri: S) -> TaskBuilder
    where
        Uri: TryFrom<S>,
        <Uri as TryFrom<S>>::Error: Into<http::Error>,
    {
        TaskBuilder::new().method(Method::CONNECT).uri(uri)
    }

    /// creates a new `TaskBuilder` initialized with a DELETE method and `URI`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let request = Task::delete("https://example.com/")
    ///     .parser(parser_fn)
    ///     .body(());
    /// ```
    pub fn delete<'a, S>(uri: S) -> TaskBuilder
    where
        Uri: TryFrom<S>,
        <Uri as TryFrom<S>>::Error: Into<http::Error>,
    {
        TaskBuilder::new().method(Method::DELETE).uri(uri)
    }

    /// creates a new `TaskBuilder` initialized with a HEAD method and `URI`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = Task::head("https://example.com/")
    ///     .parser(parser_fn)
    ///     .body(());
    /// ```
    pub fn head<'a, S>(uri: S) -> TaskBuilder
    where
        Uri: TryFrom<S>,
        <Uri as TryFrom<S>>::Error: Into<http::Error>,
    {
        TaskBuilder::new().method(Method::HEAD).uri(uri)
    }

    /// creates a new `TaskBuilder` initialized with a OPTIONS method and `URI`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = Task::options("https://example.com/")
    ///     .parser(parser_fn)
    ///     .body(());
    /// ```
    pub fn options<'a, S>(uri: S) -> TaskBuilder
    where
        Uri: TryFrom<S>,
        <Uri as TryFrom<S>>::Error: Into<http::Error>,
    {
        TaskBuilder::new().method(Method::OPTIONS).uri(uri)
    }

    /// creates a new `TaskBuilder` initialized with a PATCH method and `URI`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = Task::patch("https://example.com/")
    ///     .parser(parser_fn)
    ///     .body(());
    /// ```
    pub fn patch<S>(uri: S) -> TaskBuilder
    where
        Uri: TryFrom<S>,
        <Uri as TryFrom<S>>::Error: Into<http::Error>,
    {
        TaskBuilder::new().method(Method::PATCH).uri(uri)
    }

    /// creates a new `TaskBuilder` initialized with a TRACE method and `URI`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = Task::trace("https://example.com/")
    ///     .parser(parser_fn)
    ///     .body(());
    /// ```
    pub fn trace<'a, S>(uri: S) -> TaskBuilder
    where
        Uri: TryFrom<S>,
        <Uri as TryFrom<S>>::Error: Into<http::Error>,
    {
        TaskBuilder::new().method(Method::TRACE).uri(uri)
    }
}

impl Task {
    /// get shared reference to uri of `Task`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = Task::builder()
    ///     .parser(parser_fn)
    ///     .body(());
    /// assert_eq!(*task.uri(), *"/");
    /// ```
    pub fn uri(&self) -> &Uri {
        &self.inner.uri
    }

    /// get shared reference to method of `Task`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = Task::builder()
    ///     .parser(parser_fn)
    ///     .body(());
    /// assert_eq!(task.method(), Method::GET);
    /// ```
    pub fn method(&self) -> &Method {
        &self.inner.method
    }

    /// get shared reference to headers of `Task`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = Task::builder()
    ///     .parser(parser_fn)
    ///     .body(());
    /// assert!(task.headers().is_empty());
    /// ```
    pub fn headers(&self) -> &HeaderMap<HeaderValue> {
        &self.inner.headers
    }

    /// get headers of `Task`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = Task::builder()
    ///     .parser(parser_fn)
    ///     .body(());
    /// assert_eq!(task.version(), Version::HTTP_11);
    /// ```
    pub fn version(&self) -> Version {
        self.inner.version
    }

    /// get shared reference to exts of `Task`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = Task::builder()
    ///     .parser(parser_fn)
    ///     .body(());
    /// assert!(task.exts().is_empty());
    /// ```
    pub fn exts(&self) -> &Extensions {
        &self.metat.exts
    }

    /// get shared reference to extensions of `Task`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// struct S {}
    /// let task = Task::builder()
    ///     .parser(parser_fn)
    ///     .body(());
    /// assert!(task.extensions().get::<S>().is_none());
    /// ```
    pub fn extensions(&self) -> &Extensions {
        &self.inner.extensions
    }

    /// get shared reference to parser of `Task`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E> { todo!() }
    /// let task = Task::builder()
    ///     .parser(parser_fn)
    ///     .body(());
    /// assert_eq!(*task.parser(), parser_fn);
    /// ```
    pub fn parser<E>(&self) -> fn(Response) -> Parsed<E>
    where
        E: Serialize + Clone,
    {
        let f =
            unsafe { mem::transmute::<*const (), fn(Response) -> Parsed<E>>(self.metat.parser) };
        f
    }

    /// get shared reference to err_parser of `Task`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E> { todo!() }
    /// # fn err_parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = Task::builder()
    ///     .parser(parser_fn)
    ///     .err_parser(parser_fn)
    ///     .body(());
    /// assert_eq!(*task.err_parser(), err_parser_fn);
    /// ```
    pub fn err_parser<E>(&self) -> fn(Response) -> Parsed<E>
    where
        E: Serialize + Clone,
    {
        let f =
            unsafe { mem::transmute::<*const (), fn(Response) -> Parsed<E>>(self.metat.parser) };
        f
    }

    /// get the rank of `Task`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::Task::*;
    /// let task = Task::builder()
    ///     .body(());
    /// assert_eq!(task.rank(), 0);
    /// ```
    pub fn rank(&self) -> i16 {
        self.metat.info.rank
    }

    /// get mutable reference to rank of `Task`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::Task::*;
    /// let task = Task::builder()
    ///     .body(());
    /// task.rank_mut() = 3;
    /// assert_eq!(*task.rank_mut(), 3);
    /// ```
    pub fn rank_mut(&mut self) -> &mut i16 {
        &mut self.metat.info.rank
    }

    /// get shared reference to info of `Task`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = Task::builder()
    ///     .parser(parser_fn)
    ///     .body(());
    /// assert_eq!(task.info().used, 0);
    /// ```
    pub fn info(&self) -> &Info {
        &self.metat.info
    }

    /// get shared reference to body of `Task`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E> { todo!() }
    /// let task = Task::builder()
    ///     .parser(parser_fn)
    ///     .body(());
    /// assert!(task.body().is_empty());
    /// ```
    pub fn body(&self) -> &Body {
        &self.body
    }

    /// get mutable reference to uri of `Task`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = Task::builder()
    ///     .parser(parser_fn)
    ///     .body(());
    /// let uri = "https://example.com".Into();
    /// task.uri_mut() = uri;
    /// assert_eq!(*task.uri(), uri);
    /// ```
    pub fn uri_mut(&mut self) -> &mut Uri {
        &mut self.inner.uri
    }

    /// get mutable reference to method of `Task`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = Task::builder()
    ///     .parser(parser_fn)
    ///     .body(());
    /// let method = Method::POST;
    /// task.method_mut() = method;
    /// assert_eq!(*task.method(), method);
    /// ```
    pub fn method_mut(&mut self) -> &mut Method {
        &mut self.inner.method
    }

    /// get mutable reference to headers of `Task`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = Task::builder()
    ///     .parser(parser_fn)
    ///     .body(());
    /// task.headers_mut().insert(Method::ACCEPT, "*/*".into());
    /// assert!(!task.headers().is_empty());
    /// ```
    pub fn headers_mut(&mut self) -> &mut HeaderMap<HeaderValue> {
        &mut self.inner.headers
    }

    /// get mutable reference to version of `Task`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = Task::builder()
    ///     .parser(parser_fn)
    ///     .body(());
    /// let version = Version::HTTP_3;
    /// task.version_mut() = version;
    /// assert_eq!(task.version(), version);
    /// ```
    pub fn version_mut(&mut self) -> &mut Version {
        &mut self.inner.version
    }

    /// get mutable reference to exts of `Task`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// struct S {}
    /// let task = Task::builder()
    ///     .parser(parser_fn)
    ///     .body(());
    /// let s = S {};
    /// task.exts_mut().insert(s);
    /// assert_eq!(task.exts().get::<S>(), Some(&s));
    /// ```
    pub fn exts_mut(&mut self) -> &mut Extensions {
        &mut self.metat.exts
    }

    /// get mutable reference to extensions of `Task`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// struct S {}
    /// let task = Task::builder()
    ///     .parser(parser_fn)
    ///     .body(());
    /// let s = S {};
    /// task.extensions_mut().insert(s);
    /// assert_eq!(task.extensions().get::<S>(), Some(&s));
    /// ```
    pub fn extension_mut(&mut self) -> &mut Extensions {
        &mut self.inner.extensions
    }

    /// get mutable reference to parser of `Task`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// # fn new_parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = Task::builder()
    ///     .parser(parser_fn)
    ///     .body(());
    /// task.parser_mut() = new_parser_fn;
    /// assert_eq!(*task.parser(), new_parser_fn);
    /// ```
    pub fn parser_mut<E>(&mut self, parser: fn(Response) -> Parsed<E>)
    where
        E: Serialize + Clone,
    {
        let parser = parser as *const ();
        assert_ne!(0 as *const (), parser, "the parser cannot be NULL!");
        self.metat.parser = parser;
    }

    /// get mutable reference to err_parser of `Task`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// # fn new_parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = Task::builder()
    ///     .parser(parser_fn)
    ///     .body(());
    /// task.err_parser_mut(new_parser_fn);
    /// assert_eq!(*task.parser(), new_parser_fn);
    /// ```
    pub fn err_parser_mut<E>(&mut self, parser: fn(Response) -> Parsed<E>)
    where
        E: Serialize + Clone,
    {
        let parser = parser as *const ();
        assert_ne!(0 as *const (), parser, "the error parser cannot be NULL!");
        self.metat.err_parser = Some(parser);
    }

    /// get mutable reference to info of `Task`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = Task::builder()
    ///     .parser(parser_fn)
    ///     .body(());
    /// task.info_mut().unique = false;
    /// assert_eq!(*task.info_ref().unique, false);
    /// ```
    pub fn info_mut(&mut self) -> &mut Info {
        &mut self.metat.info
    }

    /// get mutable reference to body of `Task`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = Task::builder()
    ///     .parser(parser_fn)
    ///     .body(Vec::new());
    /// task.body_mut().extend(vec![1,2,3]);
    /// assert_eq!(*task.body().get::<Vec<i32>>, Some(&vec![1,2,3]));
    /// ```
    pub fn body_mut(&mut self) -> &mut Body {
        &mut self.body
    }

    /// Consume the task and obtain the body
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = Task::builder()
    ///     .parser(parser_fn)
    ///     .body(Vec::new());
    /// let body = task.into_body()
    /// assert_eq!(body, vec::new());
    /// ```
    pub fn into_body(self) -> Body {
        self.body
    }

    /// Convert the body of the `Task` with function
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = Task::builder()
    ///     .parser(parser_fn)
    ///     .body(vec![1,2,3]);
    /// let new_task = task.map(|v| v + 1 );
    /// assert_eq!(new_task.body, vec![2,3,4]);
    /// ```
    pub fn map<F>(self, f: F) -> Task
    where
        F: FnOnce(Body) -> Body,
    {
        Task {
            body: f(self.body),
            metat: self.metat,
            inner: self.inner,
        }
    }

    /// Create new `Task` directly with body, inner data
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = Task::builder()
    ///     .get("https://example.com")
    ///     .parser(parser_fn)
    ///     .body(vec![1,2,3]);
    /// let ( mut inner, body, meta ) = task.into_parts();
    /// inner.version = Version::HTTP_3;
    /// let new_task = Task::from_parts(inner, body, meta);
    /// ```
    pub fn from_parts(inner: InnerTask, body: Body, metat: MetaTask) -> Self {
        Self { inner, body, metat }
    }

    /// split `Task` into body, inner data
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = Task::builder()
    ///     .get("https://example.com")
    ///     .parser(parser_fn)
    ///     .body(vec![1,2,3]);
    /// let (_inner, _body, _meta ) = task.into_parts();
    /// ```
    pub fn into_parts(self) -> (InnerTask, Body, MetaTask) {
        (self.inner, self.body, self.metat)
    }
}

impl Hash for Task {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
        self.metat.hash(state);
        self.body.hash(state);
    }
}

/// An Wrapper of [http::request::Builder]
/// Serve as an medium to create an instance of `Task`
///
/// This type can be used to construct an instance or `Request`
/// through a builder-like pattern.
pub struct TaskBuilder {
    inner: InnerTask,
    meta: MetaTask,
    parser_set: bool,
}

impl TaskBuilder {
    /// Create an instance of `TaskBuilder` that used to build a `Task`  
    /// Same as `Task::builder()`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = TaskBuilder::new()
    ///     .method("GET")
    ///     .uri("https://example.com/")
    ///     .header("accept", "*/*")
    ///     .parser(parser_fn)
    ///     .body(());
    /// ```
    pub fn new() -> Self {
        Self {
            inner: InnerTask::new(),
            meta: MetaTask::new(),
            parser_set: false,
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

    /// get shared reference to exts of `TaskBuilder`
    /// Same as `Task::exts(...)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # use http::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// struct S {}
    /// let task = TaskBuilder::new()
    ///     .parser(parser_fn)
    ///     .body(());
    /// let s = S {};
    /// task.exts_mut.insert(s);
    /// assert_eq!(task.exts_ref().get::<S>(), &s);
    /// ```
    pub fn exts_ref(&self) -> &Extensions {
        &self.meta.exts
    }

    /// get mutable reference to exts of `TaskBuilder`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # use http::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = TaskBuilder::new()
    ///     .parser(parser_fn)
    ///     .body(());
    /// task.exts_mut().insert(vec![1,2,3]);
    /// assert_eq!(task.exts_ref().get::<Vec<_>>(), vec![1,2,3]);
    /// ```
    pub fn exts_mut(&mut self) -> &mut Extensions {
        &mut self.meta.exts
    }

    /// set the exts of `Task`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # use http::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = TaskBuilder::new()
    ///     .exts(vec![1,2,3])
    ///     .parser(parser_fn)
    ///     .body(());
    /// assert_eq!(task.exts_ref(), &vec![1,2,3]);
    /// ```
    pub fn exts<S>(mut self, exts: S) -> Self
    where
        S: std::any::Any + Send + Sync + 'static,
    {
        self.meta.exts.insert(exts);
        self
    }

    /// get shared reference to extensions of `Task`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// struct S {}
    /// let task = TaskBuilder::new()
    ///     .parser(parser_fn)
    ///     .body(());
    /// assert!(task.extensions().get::<S>().is_none());
    /// ```
    pub fn extensions_ref(&self) -> &Extensions {
        &self.inner.extensions
    }

    /// set the extensions of `Task`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # use http::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = TaskBuilder::new()
    ///     .extensions(vec![1,2,3])
    ///     .parser(parser_fn)
    ///     .body(());
    /// assert_eq!(task.extensions_ref(), &vec![1,2,3]);
    /// ```
    pub fn extensions<S>(mut self, extension: S) -> Self
    where
        S: std::any::Any + Send + Sync + 'static,
    {
        self.inner.extensions.insert(extension);
        self
    }

    /// get shared reference to parser of `Task`
    /// Same as `Task::parser(...)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = Task::builder()
    ///     .parser(parser_fn)
    ///     .body(());
    /// assert_eq!(*task.parser_ref(), parser_fn);
    /// ```
    pub fn parser_ref<E>(&self) -> fn(Response) -> Parsed<E>
    where
        E: Serialize + Clone,
    {
        let f = unsafe { mem::transmute::<*const (), fn(Response) -> Parsed<E>>(self.meta.parser) };
        f
    }

    /// set the parser of `Task`, Note that a task must has a parser before initialized!
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # use http::*;
    /// fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = TaskBuilder::new()
    ///     .parser(parser_fn)
    ///     .body(());
    /// assert_eq!(*task.parser_ref(), parser_fn);
    /// ```
    pub fn parser<E>(
        mut self,
        parser: fn(Response) -> Parsed<E>,
        parser_marker: &'static str,
    ) -> Self {
        let parser = parser as *const ();
        use crate::plugin::deser;
        assert_ne!(0 as *const (), parser, "the parser cannot be NULL!");
        if deser::serde_fn::query(Some(parser_marker), None).is_none()
            && deser::serde_fn::query(None, Some(parser)).is_none()
        {
            log::debug!("set parser with marker: {}", parser_marker);
            unsafe {
                deser::FNMAP.push((parser_marker, parser));
            }
        }
        self.meta.parser = parser;
        self.parser_set = true;
        self
    }

    /// get shared reference to err_parser of `Task`
    /// Same as `Task::err_parser(...)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// # fn err_parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = Task::builder()
    ///     .parser(parser_fn)
    ///     .err_parser(err_parser_fn)
    ///     .body(());
    /// assert_eq!(*task.err_parser_ref(), err_parser_fn);
    /// ```
    pub fn err_parser_ref<E>(&self) -> Option<fn(Response) -> Parsed<E>> {
        if self.meta.err_parser.is_none() {
            return None;
        }
        let f = unsafe { mem::transmute::<*const (), fn(Response) -> Parsed<E>>(self.meta.parser) };
        Some(f)
    }

    /// set the err_parser of `Task`,
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # use http::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// fn err_parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = TaskBuilder::new()
    ///     .err_parser(err_parser_fn)
    ///     .parser(parser_fn)
    ///     .body(());
    /// assert_eq!(*task.err_parser_ref(), err_parser_fn);
    /// ```
    pub fn err_parser<E>(mut self, parser: fn(Response) -> Parsed<E>) -> Self {
        let parser = parser as *const ();
        assert_ne!(0 as *const (), parser, "the parser cannot be NULL!");
        self.meta.err_parser = Some(parser);
        self
    }

    /// get shared reference to info of `Task`
    /// same as `Task::info(...)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = Task::builder()
    ///     .parser(parser_fn)
    ///     .body(());
    /// assert_eq!(task.info_ref().used, &0);
    /// ```
    pub fn info_ref(&self) -> &Info {
        &self.meta.info
    }

    /// set the info of `Task`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = TaskBuilder::new()
    ///     .parser(parser_fn)
    ///     .body(());
    /// task.info_mut().id = 18908209022;
    /// assert_eq!(task.info_ref().id, 18908209022);
    /// ```
    pub fn info_mut(&mut self) -> &mut Info {
        &mut self.meta.info
    }

    /// set the info of `Task`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let info = Info::default();
    /// let task = TaskBuilder::new()
    ///     .info(info)
    ///     .parser(parser_fn)
    ///     .body(());
    /// assert_eq!(task.info_ref(), &info);
    /// ```
    pub fn info(mut self, info: Info) -> Self {
        self.meta.info = info;
        self
    }

    /// Take this `TaskBuilder` and combine the body to create a `Task`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let _ = TaskBuilder::new()
    ///     .parser(parser_fn)
    ///     .body(());
    /// ```
    pub fn body(mut self, body: Body, marker: String) -> http::Result<Task> {
        assert!(
            self.parser_set,
            "set parser is required before building the Task"
        );
        if self.meta.info.id == 0 {
            let mut hasher = DefaultHasher::new();
            self.inner.hash(&mut hasher);
            self.meta.hash(&mut hasher);
            body.hash(&mut hasher);
            self.meta.info.id = hasher.finish();
        }
        if self.meta.info.from.path() == "/" {
            self.meta.info.from = self.inner.uri.clone();
        }
        if self.meta.info.created == 0.0 {
            self.meta.info.created = utils::now();
        }
        self.meta.info.marker = marker;
        Ok(Task {
            inner: self.inner,
            metat: self.meta,
            body,
        })
    }

    /// get shared reference to meta of `Task`,
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let task = Task::builder()
    ///     .parser(parser_fn)
    ///     .body(());
    /// assert!(!task.meta_ref().is_empty());
    /// ```
    pub fn meta_ref(&self) -> &MetaTask {
        &self.meta
    }

    /// set the meta of `Task`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::task::*;
    /// # fn parser_fn(_: Response ) -> Parsed<E,> { todo!() }
    /// let meta = MetaTask::new();
    /// let task = TaskBuilder::new()
    ///     .meta(meta)
    ///     .parser(parser_fn)
    ///     .body(());
    /// ```
    pub fn meta(mut self, meta: MetaTask) -> Self {
        self.meta = meta;
        self
    }
}
