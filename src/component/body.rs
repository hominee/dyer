//! Contains some data structure that related to the content of HTTP
//!
//! NOTE that it is [Bytes] based, not all data structure supported
//!
use crate::component::utils;
use bytes::{Buf, Bytes};
use core::ops::Deref;
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{ser::SerializeSeq, Deserialize, Deserializer, Serialize, Serializer};
use std::collections::vec_deque::{Iter, IterMut};
use std::io::IoSlice;
use std::marker::PhantomData;
use std::ops::DerefMut;
use std::{borrow::Cow, collections::VecDeque, str};
use std::{
    fmt,
    hash::{Hash, Hasher},
};

/// series of [Bytes] that receives from network or to be sent
///
/// NOTE that it is not continous buffer
///
#[derive(Serialize, Clone, Default, Deserialize, fmt::Debug)]
pub struct Body {
    pub(crate) inner: VecDeque<Chunk>,
}

/// the types that make a [Chunk]
#[derive(fmt::Debug, Clone)]
pub enum Kind {
    Byte,
    Str,
    String,
    Ref8,
    Vec8,
    Custom,
}

/// A [Bytes] that with its origin type
///
/// `Kind` is used when serializing
///
#[derive(fmt::Debug, Clone)]
pub struct Chunk(Bytes, Kind);

impl Chunk {
    pub fn new() -> Self {
        Self(Bytes::new(), Kind::Byte)
    }
}

impl Deref for Chunk {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Chunk {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::slice::from_raw_parts_mut(self.0.as_ptr() as *mut u8, self.0.len()) }
    }
}

impl Buf for Body {
    #[inline]
    fn remaining(&self) -> usize {
        self.inner.iter().map(|buf| buf.0.remaining()).sum()
    }

    #[inline]
    fn bytes(&self) -> &[u8] {
        self.inner
            .front()
            .map(|b| Buf::bytes(&b.0))
            .unwrap_or_default()
    }

    #[inline]
    fn advance(&mut self, mut cnt: usize) {
        while cnt > 0 {
            {
                let front = &mut self.inner[0].0;
                let rem = front.remaining();
                if rem > cnt {
                    front.advance(cnt);
                    return;
                } else {
                    front.advance(rem);
                    cnt -= rem;
                }
            }
            self.inner.pop_front();
        }
    }

    #[inline]
    fn bytes_vectored<'t>(&'t self, dst: &mut [IoSlice<'t>]) -> usize {
        if dst.is_empty() {
            return 0;
        }
        let mut vecs = 0;
        for buf in &self.inner {
            vecs += buf.0.bytes_vectored(&mut dst[vecs..]);
            if vecs == dst.len() {
                break;
            }
        }
        vecs
    }
}

impl Hash for Body {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for bt in &self.inner {
            bt.0.hash(state);
        }
    }
}

impl Body {
    pub fn new(chunk: Bytes) -> Self {
        let chunk = Chunk(chunk, Kind::Byte);
        let mut inner = VecDeque::new();
        inner.push_back(chunk);
        Self { inner }
    }

    pub fn empty() -> Self {
        Self {
            inner: VecDeque::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn push_back(&mut self, chunk: Chunk) {
        self.inner.push_back(chunk);
    }

    pub fn push_front(&mut self, chunk: Chunk) {
        self.inner.push_front(chunk);
    }

    pub fn map<F: FnMut(&mut VecDeque<Chunk>)>(&mut self, mut f: F) {
        f(&mut self.inner)
    }

    pub fn merge<F: FnMut(&mut VecDeque<Chunk>) -> Bytes>(&mut self, mut f: F) -> Bytes {
        f(&mut self.inner)
    }

    /// consume the body and extend the body with other body then return the result
    pub fn get_merged(&self, other: Option<&'_ Body>) -> Body {
        let l = if other.is_some() {
            other.as_ref().unwrap().len()
        } else {
            0
        };
        let len = self.len() + l;
        let mut v = VecDeque::with_capacity(len);
        for item in self.inner.iter() {
            v.push_back(Chunk::from(&item.0));
        }
        if other.is_some() {
            for item in other.unwrap().inner.iter() {
                v.push_back(Chunk::from(&item.0));
            }
        }
        Body { inner: v }
    }

    pub fn extend<I: IntoIterator<Item = Chunk>>(&mut self, iter: I) {
        self.inner.extend(iter);
    }

    pub fn iter(&self) -> Iter<'_, Chunk> {
        self.inner.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, Chunk> {
        self.inner.iter_mut()
    }

    pub fn get(&self, index: usize) -> Option<&Chunk> {
        self.inner.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Chunk> {
        self.inner.get_mut(index)
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        self.inner.swap(i, j);
    }
}

/*
 *use std::iter::FromIterator;
 *
 *impl<'b, I> From<I> for Body
 *where
 *    I: std::iter::IntoIterator<Item = Chunk>,
 *{
 *    fn from(iter: I) -> Body {
 *        Body {
 *            inner: VecDeque::from_iter(iter),
 *        }
 *    }
 *}
 */

impl From<Bytes> for Body {
    fn from(chunk: Bytes) -> Body {
        if chunk.is_empty() {
            Body::empty()
        } else {
            Body::new(chunk)
        }
    }
}

/// Customized Visitor when deserializing
///
/// Note that only a few data type supported
/////////////////// Chunk Deserializer ///////////////////
pub struct ChunkVisitor {
    _t: PhantomData<fn() -> Chunk>,
}

impl ChunkVisitor {
    pub fn new() -> Self {
        ChunkVisitor { _t: PhantomData }
    }
}

impl<'de> Visitor<'de> for ChunkVisitor {
    type Value = Chunk;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Chunk")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        //let b = unsafe { v as *const [u8] as *const Bytes };
        let s = utils::slice(v.as_ptr(), v.len());
        Ok(Chunk(Bytes::from(s), Kind::Byte))
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_bytes(v)
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let s = utils::slice(v.as_ptr(), v.len());
        Ok(Chunk(Bytes::from(s), Kind::Vec8))
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let s = utils::slice(s.as_ptr(), s.len());
        Ok(Chunk(Bytes::from(s), Kind::Str))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let s = utils::slice(v.as_ptr(), v.len());
        Ok(Chunk(Bytes::from(s), Kind::String))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut buf: Vec<u8> = Vec::with_capacity(seq.size_hint().unwrap_or(0));
        while let Some(e) = seq.next_element()? {
            //debug_assert!(e as u8);
            buf.push(e);
        }
        Ok(Chunk(Bytes::from(buf), Kind::Vec8))

        //Err(de::Error::invalid_type(de::Unexpected::Seq, &self))
    }

    fn visit_map<M>(self, access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let _ = access;
        Err(de::Error::invalid_type(de::Unexpected::Map, &self))
    }
}

impl<'de, 'b> Deserialize<'de> for Chunk {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        d.deserialize_any(ChunkVisitor::new())
    }
}

impl Serialize for Chunk {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.1 {
            Kind::Str | Kind::String => {
                let v = str::from_utf8(&self.0[..]).unwrap();

                s.serialize_str(v)
            }
            Kind::Ref8 | Kind::Vec8 => {
                let mut ser = s.serialize_seq(Some(self.0.len())).unwrap();
                for e in self.0.iter() {
                    ser.serialize_element(e)?;
                }
                ser.end()
            }
            Kind::Byte => s.serialize_bytes(&self.0),
            // Kind::Custom is not available so far
            _ => unreachable!(),
        }
    }
}

/*
 *impl<S: Serialize> From<S> for Chunk {
 *    fn from(s: S) -> Chunk {
 *        let ss = serde_json::to_vec(&s).unwrap();
 *        Chunk::from(&ss)
 *    }
 *}
 *
 *impl<S: for<'d> Deserialize<'d>> Into<S> for Chunk {
 *    fn into(self) -> S {
 *        serde_json::from_slice::<S>(&self.0).unwrap()
 *    }
 *}
 */

use std::io::BufReader;
impl<R> From<BufReader<R>> for Chunk {
    fn from(buf: BufReader<R>) -> Chunk {
        let buf = buf.buffer();
        let s = utils::slice(buf.as_ptr(), buf.len());
        Chunk(Bytes::from(s), Kind::Byte)
    }
}

impl From<&Bytes> for Chunk {
    fn from(buf: &Bytes) -> Chunk {
        let s = utils::slice(buf.as_ptr(), buf.len());
        Chunk(Bytes::from(s), Kind::Byte)
    }
}

impl From<Vec<u8>> for Chunk {
    fn from(u8s: Vec<u8>) -> Chunk {
        Chunk(Bytes::from(u8s), Kind::Vec8)
    }
}

impl From<&[u8]> for Chunk {
    fn from(u8s: &[u8]) -> Chunk {
        let s = utils::slice(u8s.as_ptr(), u8s.len());
        Chunk(Bytes::from(s), Kind::Ref8)
    }
}

impl<'b> From<Cow<'b, [u8]>> for Chunk {
    fn from(cow: Cow<'b, [u8]>) -> Chunk {
        match cow {
            Cow::Borrowed(e) => Chunk(Bytes::from(utils::slice(e.as_ptr(), e.len())), Kind::Ref8),
            Cow::Owned(e) => Chunk(Bytes::from(e), Kind::Ref8),
        }
    }
}

impl From<String> for Chunk {
    fn from(s: String) -> Chunk {
        Chunk(Bytes::from(s), Kind::String)
    }
}

impl<'b> From<&'b str> for Chunk {
    fn from(slice: &'b str) -> Chunk {
        Chunk(
            Bytes::from(utils::slice(slice.as_ptr(), slice.len())),
            Kind::Str,
        )
    }
}

impl<'b> From<Cow<'b, str>> for Chunk {
    fn from(cow: Cow<'b, str>) -> Chunk {
        match cow {
            Cow::Borrowed(e) => Chunk(Bytes::from(utils::slice(e.as_ptr(), e.len())), Kind::Ref8),
            Cow::Owned(e) => Chunk(Bytes::from(e), Kind::Ref8),
        }
    }
}

impl From<Vec<u8>> for Body {
    fn from(u8s: Vec<u8>) -> Body {
        let mut inner = VecDeque::new();
        inner.push_back(Chunk(Bytes::from(u8s), Kind::Vec8));
        Body { inner }
    }
}

impl<'b> From<&'b [u8]> for Body {
    fn from(u8s: &'b [u8]) -> Body {
        let mut inner = VecDeque::new();
        inner.push_back(Chunk(
            Bytes::from(utils::slice(u8s.as_ptr(), u8s.len())),
            Kind::Ref8,
        ));
        Body { inner }
    }
}

impl<'b> From<Cow<'b, [u8]>> for Body {
    fn from(cow: Cow<'b, [u8]>) -> Body {
        let mut inner = VecDeque::new();
        match cow {
            Cow::Borrowed(e) => {
                inner.push_back(Chunk(
                    Bytes::from(utils::slice(e.as_ptr(), e.len())),
                    Kind::Ref8,
                ));
            }
            Cow::Owned(e) => {
                inner.push_back(Chunk(Bytes::from(e), Kind::Ref8));
            }
        }
        Body { inner }
    }
}

impl From<String> for Body {
    fn from(s: String) -> Body {
        let mut inner = VecDeque::new();
        inner.push_back(Chunk(Bytes::from(s), Kind::String));
        Body { inner }
    }
}

impl<'b> From<&'b str> for Body {
    fn from(slice: &'b str) -> Body {
        let mut inner = VecDeque::new();
        inner.push_back(Chunk(
            Bytes::from(utils::slice(slice.as_ptr(), slice.len())),
            Kind::Str,
        ));
        Body { inner }
    }
}

impl<'b> From<Cow<'b, str>> for Body {
    fn from(cow: Cow<'b, str>) -> Body {
        match cow {
            Cow::Borrowed(e) => Body::from(e),
            Cow::Owned(e) => Body::from(e),
        }
    }
}
