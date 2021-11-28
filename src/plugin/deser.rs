//! Some useful tools to serialize and deserialize components,
//!
use http::{header::HeaderName, method::Method, version::Version, HeaderMap, HeaderValue, Uri};
use serde::ser::{self, SerializeMap};
use serde::{Deserialize, Deserializer, Serializer};
use std::str::FromStr;

/// A vector of tuple which contians function name(&str) and function pointer (*const ()),
///
/// it should initialize before the program starts
pub static mut FNMAP: Vec<(&'static str, *const ())> = vec![];

/// mod that contains serialize funtion and deserialize funtion to actor parser funciton
/// it deserializes string into actor parser when loading and
/// serialize the parser function to string when storing, debug, or stdout
///
pub mod serde_fn {
    use super::*;
    pub(crate) fn query(name: Option<&str>, ptr: Option<*const ()>) -> Option<(&str, *const ())> {
        match (name, ptr) {
            (None, None) | (Some(_), Some(_)) => unreachable!(),
            // Safety: only read the data in single thread and no mutating happens
            (Some(n), None) => unsafe {
                for (k, v) in FNMAP.iter() {
                    if *k == n {
                        return Some((k, *v));
                    }
                }
                None
            },
            (None, Some(p)) => unsafe {
                for (k, v) in FNMAP.iter() {
                    if std::ptr::eq(p, *v) {
                        return Some((k, *v));
                    }
                }
                None
            },
        }
    }

    /// serfn function that serializes the function pointer to function name, if the coresponding
    /// function is not found then `Unknow` is the default.  
    pub(crate) fn serfn<S>(t: &*const (), s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some((k, _)) = query(None, Some(*t)) {
            return s.serialize_str(k);
        }
        Err(ser::Error::custom(format!(
            "Failed Serialize Function Pointer"
        )))
    }

    /// deserialize function that deseralizes the function string nane to fucntion coresponding
    /// function pointer, if the function pointer is not found then panic happens.
    pub(crate) fn defn<'de, D>(d: D) -> Result<*const (), D::Error>
    where
        D: Deserializer<'de>,
    {
        let index = <&str>::deserialize(d)?;
        if let Some((_, v)) = query(Some(index), None) {
            return Ok(v);
        }
        Err(serde::de::Error::custom(format!(
            "Function Pointer {:?} Not Found",
            index,
        )))
    }

    /// serfn function that serializes the function pointer to function name, if the coresponding
    /// function is not found then `None` is the default.  
    pub(crate) fn serfn_op<S>(arg: &Option<*const ()>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some((k, _)) = query(None, *arg) {
            return s.serialize_some(k);
        }
        s.serialize_none()
    }

    /// deserialize function that deseralizes the function string nane to fucntion coresponding
    /// function pointer, if the function pointer is not found then panic happens.
    pub(crate) fn defn_op<'de, D>(d: D) -> Result<Option<*const ()>, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        if let Ok(index) = <&str>::deserialize(d) {
            if let Some((_, v)) = query(Some(index), None) {
                return Ok(Some(v));
            }
        }
        Ok(None)
    }
}

/// mod that contains serialize funtion and deserialize funtion to ser-de [http::Version]
///
pub mod serde_version {
    use super::*;
    /// deserialize [http::Version]
    pub fn serialize<S>(arg: &Version, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *arg {
            Version::HTTP_09 => serializer.serialize_str("HTTP/0.9"),
            Version::HTTP_10 => serializer.serialize_str("HTTP/10"),
            Version::HTTP_11 => serializer.serialize_str("HTTP/11"),
            Version::HTTP_2 => serializer.serialize_str("HTTP/2.0"),
            Version::HTTP_3 => serializer.serialize_str("HTTP/3.0"),
            _ => Err(serde::ser::Error::custom("Unknow Version")),
        }
    }

    /// serialize [http::Version]
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Version, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = <&str>::deserialize(deserializer)?;
        match raw {
            "HTTP/0.9" => Ok(Version::HTTP_09),
            "HTTP/10" => Ok(Version::HTTP_10),
            "HTTP/11" => Ok(Version::HTTP_11),
            "HTTP/2.0" => Ok(Version::HTTP_2),
            "HTTP/3.0" => Ok(Version::HTTP_3),
            _ => Err(serde::de::Error::custom(format!(
                "Invalid Input As Version"
            ))),
        }
    }
}

/// mod that contains serialize funtion and deserialize funtion to ser-de [http::Method]
///
pub mod serde_method {
    use super::*;
    pub fn serialize<S>(arg: &Method, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(arg.as_str())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Method, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = <&str>::deserialize(deserializer)?;
        Ok(Method::from_str(raw).unwrap())
    }
}

/// mod that contains serialize funtion and deserialize funtion to ser-de [http::Uri]
///
pub mod serde_uri {
    use super::*;
    pub fn serialize<S>(arg: &Uri, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&arg.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Uri, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = <&str>::deserialize(deserializer)?;
        Ok(Uri::from_str(raw).unwrap())
    }
}

/// mod that contains serialize funtion and deserialize funtion to ser-de [http::header::HeaderName]
///
pub mod serde_headername {
    use super::*;
    pub fn serialize<S>(arg: &HeaderName, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(arg.as_str())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HeaderName, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = <&str>::deserialize(deserializer)?;
        Ok(HeaderName::from_str(raw).unwrap())
    }
}

/// mod that contains serialize funtion and deserialize funtion to ser-de [http::HeaderValue]
///
pub mod serde_headervalue {
    use super::*;
    pub fn serialize<S>(arg: &HeaderValue, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(arg.to_str().unwrap())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HeaderValue, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = <&str>::deserialize(deserializer)?;
        Ok(HeaderValue::from_str(raw).unwrap())
    }
}

/// mod that contains serialize funtion and deserialize funtion to ser-de [http::HeaderMap]
///
pub mod serde_headermap {
    use super::serde_headermapop::*;
    use super::*;
    /////////////////// HeaderMap Deserializer ///////////////////

    pub fn serialize<S>(arg: &HeaderMap, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = serializer.serialize_map(Some(arg.len()))?;
        for (k, v) in arg.iter() {
            ser.serialize_entry(k.as_str(), v.to_str().unwrap())?;
        }
        ser.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HeaderMap, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(HeaderMapVisitor::new())
    }
}

/// mod that contains serialize funtion and deserialize funtion to ser-de Option variant of [http::HeaderMap]
///
pub mod serde_headermapop {
    use http::header::*;
    use serde::{
        de::{MapAccess, Visitor},
        ser::SerializeMap,
        Deserializer, Serializer,
    };
    use std::marker::PhantomData;
    use std::{fmt, str::FromStr};
    /////////////////// HeaderMap Deserializer ///////////////////
    pub struct HeaderMapVisitor<HeaderValue> {
        marker: PhantomData<fn() -> HeaderMap<HeaderValue>>,
    }

    struct OptionHeaderMapVisitor<HeaderValue> {
        marker: PhantomData<fn() -> Option<HeaderMap<HeaderValue>>>,
    }

    impl HeaderMapVisitor<HeaderValue> {
        pub fn new() -> Self {
            HeaderMapVisitor {
                marker: PhantomData,
            }
        }
    }

    impl OptionHeaderMapVisitor<HeaderValue> {
        fn new() -> Self {
            OptionHeaderMapVisitor {
                marker: PhantomData,
            }
        }
    }

    impl<'de> Visitor<'de> for HeaderMapVisitor<HeaderValue> {
        type Value = HeaderMap<HeaderValue>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("Header Map")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut map = HeaderMap::with_capacity(access.size_hint().unwrap_or(0));
            while let Some((key, value)) = access.next_entry()? {
                map.append(
                    HeaderName::from_str(key).unwrap(),
                    HeaderValue::from_str(value).unwrap(),
                );
            }
            Ok(map)
        }
    }

    impl<'de> Visitor<'de> for OptionHeaderMapVisitor<HeaderValue> {
        type Value = Option<HeaderMap<HeaderValue>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("Option Header Map")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            Ok(Some(deserializer.deserialize_map(HeaderMapVisitor::new())?))
        }
    }

    pub fn serialize<S>(arg: &Option<HeaderMap>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if arg.is_none() {
            return serializer.serialize_none();
        }

        let item = arg.as_ref().unwrap();
        let mut ser = serializer.serialize_map(Some(item.len()))?;
        for (k, v) in item.iter() {
            ser.serialize_entry(k.as_str(), v.to_str().unwrap())?;
        }
        ser.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<HeaderMap>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_option(OptionHeaderMapVisitor::new())
    }
}
