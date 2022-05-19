//! some utilities that useful and convenience for dealing with data flow.
//!
use crate::{component::Poly, engine::vault::Vault};
use std::collections::hash_map::DefaultHasher;
use std::convert::TryInto;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::io::{BufRead, BufReader};

/// get the function name of the function
/// empty string returned if not works correctly
pub fn function_name<T>(_: T) -> &'static str {
    let name = std::any::type_name::<T>();
    let segs = name.rsplitn(2, "::").collect::<Vec<_>>();
    if !segs.is_empty() {
        segs[0]
    } else {
        ""
    }
}

/// load unfinished or extra data
pub fn load<T>(
    path: &str,
    f: Option<&Box<dyn Fn(&str) -> Poly + Send>>,
    //f: Option<&'a dyn Fn(&'a str) -> T>,
) -> Vec<T>
where
    Poly: TryInto<T>,
{
    let mut data = Vec::new();
    if let Some(ff) = f {
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .unwrap();
        for line in BufReader::new(&file).lines() {
            let s = line.unwrap().to_string();
            if let Ok(item) = ff(&s).try_into() {
                data.push(item);
            }
        }
    } else {
        log::error!("Session Loader Not Provided");
    }
    data
}

/// store unfinished or extra data,
pub(crate) fn stored<I, T>(
    path: &str,
    ens: &mut Vault<I>,
    f: Option<&Box<dyn for<'a> Fn(Poly, &'a ()) -> &'a str + Send>>,
    //f: Option<&'a dyn Fn(I::Item) -> &'a str>,
) where
    I: std::iter::IntoIterator<Item = T> + Default,
    Poly: From<T>,
{
    if let Some(ff) = f {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .unwrap();
        let buf = ens.take();
        let mut cows: Vec<std::borrow::Cow<str>> = Vec::new();
        for en in buf.into_iter() {
            let item = (ff)(en.into(), &());
            cows.push(std::borrow::Cow::Borrowed(item));
            cows.push(std::borrow::Cow::Borrowed("\n"));
        }

        cows.into_iter().for_each(|cow| match cow {
            std::borrow::Cow::Owned(s) => {
                file.write(s.as_bytes()).unwrap();
            }
            std::borrow::Cow::Borrowed(s) => {
                file.write(s.as_bytes()).unwrap();
            }
        });
    } else {
        log::error!("Session Storer Not Provided");
    }
}

/// handy tool to get the instant time of system time
pub fn now() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
}

/// generate hash via Iterator
pub fn hash_iter<I>(salt: I) -> u64
where
    I: std::iter::IntoIterator,
    I::Item: Hash,
{
    let mut hasher = DefaultHasher::new();
    salt.into_iter().for_each(|ele| ele.hash(&mut hasher));
    hasher.finish()
}
/// hash a hash-able object
pub fn hash<I>(salt: I) -> u64
where
    I: Hash,
{
    let mut hasher = DefaultHasher::new();
    salt.hash(&mut hasher);
    hasher.finish()
}

/// basically re-construct a slice from ptr
///
/// generally speaking, it is used to manipulate the lifetime
pub fn slice<'a, T: 'a>(ptr: *const T, len: usize) -> &'a [T] {
    unsafe { std::slice::from_raw_parts(ptr, len) }
}
