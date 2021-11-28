//! This module contains structs that adjusts [Task] before consume it
//!
//! Generally, it is not necessary,
//! Check the [examples] in case you need it
//!
//! [Task]: crate::component::task::Task
//! [examples]: <https://github.com/HomelyGuy/dyer/tree/master/examples/>
//!
use std::collections::hash_map::DefaultHasher;
use std::collections::BinaryHeap;
use std::convert::TryFrom;
use std::hash::{Hash, Hasher};

use crate::component::{body::Body, info::Info};
use crate::plugin::deser::*;
use http::{header::HeaderName, Extensions, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::fmt;

/// generally, it provide extra infomation , meta data required by server,
/// including `User-Agent`, `Accept-Encoding` and so on.
#[derive(Deserialize, Default, Serialize)]
pub struct Affix {
    pub(crate) inner: InnerAffix,
    /// Formdata, files or other request parameters stored here
    pub(crate) body: Body,
    /// some metadata about this Affix,
    pub(crate) metap: MetaAffix,
}

/// Meta Data of the Affix
///
/// `MetaAffix` can be used to carry infomation about the worlflow and beyond
#[derive(Deserialize, Default, Serialize)]
pub struct MetaAffix {
    /// info about the Affix
    pub info: Info,
    /// additional arguments for extensive application
    #[serde(skip)]
    pub exts: Extensions,
}

impl Hash for MetaAffix {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.info.used.hash(state);
        self.info.marker.hash(state);
    }
}

impl fmt::Debug for Affix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Affix")
            .field("headers", &self.inner.headers)
            .field("body", &self.body)
            .field("metap", &self.metap)
            // omit extensions
            .finish()
    }
}

impl fmt::Debug for MetaAffix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MetaAffix")
            .field("info", &self.info)
            .finish()
    }
}

impl Affix {
    /// Create an instance of `AffixBuilder` that used to build a `Affix`  
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::affix::*;
    /// let affix = affix::builder()
    ///     .header("accept", "*/*")    
    ///     .body(())
    ///     .unwrap();
    /// ```
    pub fn builder() -> AffixBuilder {
        AffixBuilder::new()
    }

    /// get shared reference to headers of `Affix`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::affix::*;
    /// let affix = Affix::builder()    
    ///     .body(());
    /// assert!(affix.headers().is_empty());
    /// ```
    pub fn headers(&self) -> &HeaderMap<HeaderValue> {
        &self.inner.headers
    }

    /// get mutable reference to headers of `Affix`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::affix::*;
    /// let affix = Affix::builder()    
    ///     .body(());
    /// affix.headers_mut().insert(Method::ACCEPT, "*/*".into());
    /// assert!(!affix.headers().is_empty());
    /// ```
    pub fn headers_mut(&mut self) -> &mut HeaderMap<HeaderValue> {
        &mut self.inner.headers
    }

    /// get shared reference to exts of `Affix`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::affix::*;
    /// let affix = Affix::builder()    
    ///     .body(());
    /// assert!(affix.exts().is_empty());
    /// ```
    pub fn exts(&self) -> &Extensions {
        &self.metap.exts
    }

    /// get mutable reference to exts of `Affix`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::affix::*;
    /// struct S {}
    /// let affix = Affix::builder()    
    ///     .body(());
    /// let s = S {};
    /// affix.exts_mut().insert(s);
    /// assert_eq!(affix.exts().get::<S>(), Some(&s));
    /// ```
    pub fn exts_mut(&mut self) -> &mut Extensions {
        &mut self.metap.exts
    }

    /// get the rank of `Affix`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::affix::*;
    /// let affix = Affix::builder()
    ///     .body(());
    /// assert_eq!(affix.rank(), 0);
    /// ```
    pub fn rank(&self) -> i16 {
        self.metap.info.rank
    }

    /// get mutable reference to rank of `Affix`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::affix::*;
    /// let affix = Affix::builder()
    ///     .body(());
    /// affix.rank_mut() = 3;
    /// assert_eq!(*affix.rank(), 3);
    /// ```
    pub fn rank_mut(&mut self) -> &mut i16 {
        &mut self.metap.info.rank
    }

    /// get mutable reference to body of `Affix`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::affix::*;
    /// let affix = Affix::builder()
    ///     .body(Vec::new());    
    /// affix.body_mut().extend(vec![1,2,3]);    
    /// assert_eq!(*affix.body().get::<Vec<i32>>, Some(&vec![1,2,3]));
    /// ```
    pub fn body_mut(&mut self) -> &mut Body {
        &mut self.body
    }
    /// get shared reference to body of `Affix`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::affix::*;
    /// let affix = Affix::builder()
    ///     .body(());    
    /// assert!(affix.body().is_empty());
    /// ```
    pub fn body(&self) -> &Body {
        &self.body
    }
    /// Consume the affix and obtain the body
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::affix::*;
    /// let affix = Affix::builder()
    ///     .body(Vec::new());    
    /// let body = affix.into_body()    
    /// assert_eq!(body, vec::new());
    /// ```
    pub fn into_body(self) -> Body {
        self.body
    }
    /// Convert the body of the `Affix` with function
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::affix::*;
    /// let affix = Affix::builder()
    ///     .body(vec![1,2,3]);
    /// let new_affix = affix.map(|v| v + 1 );    
    /// assert_eq!(new_affix.body, vec![2,3,4]);
    /// ```
    pub fn map<F>(self, f: F) -> Affix
    where
        F: FnOnce(Body) -> Body,
    {
        Affix {
            body: f(self.body),
            metap: self.metap,
            inner: self.inner,
        }
    }
    /// Create new `affix` directly with body, inner data
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::affix::*;
    /// let affix = affix::builder()
    ///     .get("https://example.com")
    ///     .body(vec![1,2,3]);    
    /// let ( mut inner, body, meta ) = affix.into_parts();
    /// inner.version = Version::HTTP_3;    
    /// let new_affix = affix::from_parts(inner, body, meta);
    /// ```
    pub fn from_parts(headers: HeaderMap<HeaderValue>, body: Body, metap: MetaAffix) -> Self {
        let inn = InnerAffix {
            headers,
            extensions: Extensions::new(),
        };
        Self {
            inner: inn,
            body,
            metap,
        }
    }
    /// split `affix` into body, inner data
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::affix::*;
    /// let affix = affix::builder()
    ///     .get("https://example.com")
    ///     .body(vec![1,2,3]);    
    /// let (_inner, _body, _meta ) = affix.into_parts();
    /// ```
    pub fn into_parts(self) -> (InnerAffix, Body, MetaAffix) {
        (self.inner, self.body, self.metap)
    }
}

impl Hash for Affix {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
        self.metap.hash(state);
        self.body.hash(state);
    }
}

/// Partial component of an HTTP `Request`
///
/// it provides additional infomation
#[derive(Serialize, Deserialize, Default)]
pub struct InnerAffix {
    /// represents a headers
    #[serde(with = "serde_headermap")]
    pub headers: HeaderMap<HeaderValue>,
    /// additional arguments for extensive application
    #[serde(skip)]
    pub extensions: Extensions,
}

impl Hash for InnerAffix {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut heap: BinaryHeap<(&str, &str)> = BinaryHeap::new();
        self.headers.iter().for_each(|(k, v)| {
            heap.push((k.as_str(), v.to_str().unwrap()));
        });
        while let Some((k, v)) = heap.pop() {
            k.hash(state);
            v.hash(state);
        }
    }
}

/// Serve as an media to create an instance of `Affix`
///
/// This type can be used to construct an instance or `Affix`
/// through a builder-like pattern.
pub struct AffixBuilder {
    inner: InnerAffix,
    meta: MetaAffix,
}

impl AffixBuilder {
    /// Create an instance of `AffixBuilder` that used to build a `Affix`  
    /// Same as `Affix::builder()`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::affix::*;
    /// let affix = AffixBuilder::new()
    ///     .body(());
    /// ```
    pub fn new() -> Self {
        Self {
            inner: InnerAffix::default(),
            meta: MetaAffix::default(),
        }
    }

    /// get shared reference to header of `AffixBuilder`
    /// Same as `Affix::headers(...)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::affix::*;
    /// let affix = AffixBuilder::new()
    ///     .header("accept", "*/*")    
    ///     .body(());
    /// assert_eq!(affix.header_ref()["accept"], "*/*");
    /// ```
    pub fn header_ref(&self) -> &HeaderMap<HeaderValue> {
        &self.inner.headers
    }

    /// get mutable reference to header of `AffixBuilder`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::affix::*;
    /// let affix = AffixBuilder::new()
    ///     .header("accept", "*/*")    
    ///     .body(());
    /// affix.header_mut().insert("accept", "text/html");
    /// assert_eq!(affix.header_ref()["accept"], "text/html");
    /// ```
    pub fn header_mut(&mut self) -> &mut HeaderMap<HeaderValue> {
        &mut self.inner.headers
    }

    /// set the headers of `Affix`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::affix::*;
    /// # use http::*;
    /// let affix = AffixBuilder::new()
    ///     .header("accept", "*/*")
    ///     .body(());
    /// assert_eq!(affix.header_ref()["accept"], "*/*");
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

    /// get shared reference to meta extensions of `Affix`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::affix::*;
    /// struct S {}
    /// let affix = AffixBuilder::new()
    ///     .body(());
    /// assert!(affix.extensions().get::<S>().is_none());
    /// ```
    pub fn extensions_ref(&self) -> &Extensions {
        &self.inner.extensions
    }

    /// get mutable reference to extensions of `Affix`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::affix::*;
    /// struct S {}
    /// let affix = Affix::builder()
    ///     .body(());
    /// let s = S {};
    /// affix.extensions_mut().insert(s);
    /// assert_eq!(affix.extensions().get::<S>(), Some(&s));
    /// ```
    pub fn extension_mut(&mut self) -> &mut Extensions {
        &mut self.inner.extensions
    }

    /// set the extensions of `Affix`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::affix::*;
    /// # use http::*;
    /// let affix = AffixBuilder::new()
    ///     .extensions(vec![1,2,3])
    ///     .body(());
    /// assert_eq!(affix.extensions_ref(), &vec![1,2,3]);
    /// ```
    pub fn extensions<S>(mut self, extension: S) -> Self
    where
        S: std::any::Any + Send + Sync + 'static,
    {
        self.inner.extensions.insert(extension);
        self
    }

    /// get shared reference to exts of `AffixBuilder`
    /// Same as `Affix::exts(...)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::affix::*;
    /// # use http::*;
    /// struct S {}
    /// let affix = AffixBuilder::new()
    ///     .parser(parser_fn)
    ///     .body(());
    /// let s = S {};
    /// affix.exts_mut.insert(s);
    /// assert_eq!(affix.exts_ref().get::<S>(), &s);
    /// ```
    pub fn exts_ref(&self) -> &Extensions {
        &self.meta.exts
    }

    /// get mutable reference to exts of `AffixBuilder`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::Affix::*;
    /// # use http::*;
    /// let affix = AffixBuilder::new()
    ///     .body(());
    /// affix.exts_mut().insert(vec![1,2,3]);
    /// assert_eq!(affix.exts_ref().get::<Vec<_>>(), vec![1,2,3]);
    /// ```
    pub fn exts_mut(&mut self) -> &mut Extensions {
        &mut self.meta.exts
    }

    /// set the exts of `Affix`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::Affix::*;
    /// let Affix = AffixBuilder::new()
    ///     .exts(vec![1,2,3])
    ///     .body(());
    /// assert_eq!(Affix.exts_ref(), &vec![1,2,3]);
    /// ```
    pub fn exts<S>(mut self, exts: S) -> Self
    where
        S: std::any::Any + Send + Sync + 'static,
    {
        self.meta.exts.insert(exts);
        self
    }

    /// get shared reference to info of `Affix`
    /// same as `Affix::info(...)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::affix::*;
    /// let affix = Affix::builder()
    ///     .body(());
    /// assert_eq!(affix.info_ref().unique, true);
    /// ```
    pub fn info_ref(&self) -> &Info {
        &self.meta.info
    }

    /// get mutable reference to info of `Affix`
    /// same as `Affix::info_mut(...)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::affix::*;
    /// let info = Info::default();
    /// info.unique = false;
    /// let affix = Affix::builder()
    ///     .body(());
    /// affix.info_mut() = info;
    /// assert_eq!(affix.info_ref().unique, false);
    /// ```
    pub fn info_mut(&mut self) -> &mut Info {
        &mut self.meta.info
    }

    /// set the info of `Affix`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::affix::*;
    /// let affix = affixBuilder::new()
    ///     .body(());
    /// assert_eq!(affix.info_ref().unique, true);
    /// ```
    pub fn info<S>(mut self, info: Info) -> Self {
        self.meta.info = info;
        self
    }

    /// Take this `AffixBuilder` and combine the body to create a `Affix`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::affix::*;
    /// let _ = AffixBuilder::new()
    ///     .body(());
    /// ```
    pub fn body(mut self, body: Body) -> http::Result<Affix> {
        if self.meta.info.id == 0 {
            let mut hasher = DefaultHasher::new();
            self.inner.hash(&mut hasher);
            self.meta.hash(&mut hasher);
            body.hash(&mut hasher);
            self.meta.info.id = hasher.finish();
        }
        Ok(Affix {
            inner: self.inner,
            metap: self.meta,
            body,
        })
    }

    /// get shared reference to meta of `Affix`,
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::affix::*;
    /// let affix = Affix::builder()    
    ///     .body(());
    /// assert_eq!(affix.meta_ref().rank, 0);
    /// ```
    pub fn meta_ref(&self) -> &MetaAffix {
        &self.meta
    }

    /// set the meta of `Affix`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::affix::*;
    /// let meta = MetaAffix::new();
    /// let affix = AffixBuilder::new()
    ///     .meta(meta)    
    ///     .body(());
    /// ```
    pub fn meta(mut self, meta: MetaAffix) -> Self {
        self.meta = meta;
        self
    }
}
