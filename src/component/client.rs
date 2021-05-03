extern crate brotli2;
extern crate flate2;

use crate::component::{Request, ResError, Response};
use crate::engine::ArgRate;
use bytes::buf::ext::BufExt;
use futures::{executor::block_on, future::join_all, Future};
use hyper::{client::HttpConnector, Body as hBody, Client as hClient, Request as hRequest};
use hyper_timeout::TimeoutConnector;
use hyper_tls::HttpsConnector;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufReader, Read};
use std::sync::{Arc, Mutex, Once};
use std::{time, time::UNIX_EPOCH};

pub type MClient = hClient<TimeoutConnector<HttpsConnector<HttpConnector>>>;

// FIXME add proxy support
/// the `Client` that asynchronously executes `Request`s, with specified `connect-timeout`, `read-timeout` and
/// `write-timeout`. Note that polling the `Request`s requires `tokio::runtime` (other asynchronous
/// runtime, proxy will work in the future).
pub struct Client;

impl Client {
    /// new static client
    pub fn new(con: u64, read: u64, write: u64) -> &'static Option<MClient> {
        static INIT: Once = Once::new();
        static mut VAL: Option<MClient> = None;
        unsafe {
            INIT.call_once(|| {
                let https: HttpsConnector<HttpConnector> = HttpsConnector::new();
                let mut conn = hyper_timeout::TimeoutConnector::new(https);
                conn.set_connect_timeout(Some(std::time::Duration::from_secs(con)));
                conn.set_read_timeout(Some(std::time::Duration::from_secs(read)));
                conn.set_write_timeout(Some(std::time::Duration::from_secs(write)));
                let clt = hClient::builder().build::<_, hBody>(conn);
                VAL = Some(clt);
            });
            &VAL
        }
    }
}

impl Client {
    ///this function require a `hyper::Request` and `hyper::Client` to return the Response
    pub fn block_exec<F: Future>(f: F) -> F::Output {
        block_on(f)
    }

    /// for the sake of convenience, polling the `Request` in no time, is designed for in creating `Spider.entry_profile` or `Spider.entry_task`  Note that:  DO NOT use it in most
    /// of your code, cz it will slow your whole program down.
    pub async fn request<T, P>(req: Request<T, P>) -> Response<T, P>
    where
        T: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + Clone,
        P: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + Clone,
    {
        let mut r = Response::default(Some(&req));
        match req.init() {
            None => r,
            Some(request) => match Client::exec(request, None).await {
                Ok(res) => {
                    r.content = res.0;
                    r.headers = res.1;
                    r.status = res.2;
                    r
                }
                Err(e) => {
                    log::error!("failed Request: {:?}", e.desc);
                    r.msg = Some(e.desc);
                    r
                }
            },
        }
    }

    /// the core part of `Client`, as to poll the `Request`, and asynchronously aggregate data from
    /// server.
    pub async fn exec(
        req: hRequest<hBody>,
        args: Option<bool>,
    ) -> Result<(Option<String>, HashMap<String, String>, usize, f64), ResError> {
        let client = &Client::new(7, 23, 7).as_ref().unwrap();
        let tic = time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        match client.request(req).await {
            Ok(response) => {
                let toc = time::SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64();
                let gap = toc - tic;
                match args {
                    Some(true) | None => {
                        let (header, bd) = response.into_parts();
                        let bod = hyper::body::aggregate(bd).await;
                        match bod {
                            Ok(body) => {
                                let mut encoding = "".to_string();
                                let mut reader = BufReader::new(body.reader());
                                let status = header.status.as_u16() as usize;
                                let mut headers: HashMap<String, String> = HashMap::new();
                                let cookies: Vec<&str> = header
                                    .headers
                                    .get_all("set-cookie")
                                    .iter()
                                    .map(|s| s.to_str().unwrap())
                                    .collect();
                                let cookie = cookies.join("::");
                                headers.insert("set-cookie".to_string(), cookie);
                                header
                                    .headers
                                    .into_iter()
                                    .for_each(|(key, value)| match key {
                                        None => {}
                                        Some(k) => {
                                            let kk = k.to_string();
                                            if kk.to_lowercase() == "content-encoding".to_string() {
                                                encoding = value.to_str().unwrap().to_string();
                                            }
                                            if &kk != "set-cookie" {
                                                headers.insert(
                                                    kk,
                                                    value.to_str().unwrap().to_string(),
                                                );
                                            }
                                        }
                                    });

                                // Response Content
                                let mut data = String::new();
                                if encoding == "gzip".to_string()
                                    || encoding == "deflate".to_string()
                                {
                                    let mut gz = flate2::read::GzDecoder::new(reader);
                                    gz.read_to_string(&mut data).unwrap();
                                } else if encoding == "br".to_string() {
                                    let mut br = brotli2::read::BrotliDecoder::new(reader);
                                    br.read_to_string(&mut data).unwrap();
                                } else {
                                    reader.read_to_string(&mut data).unwrap();
                                }

                                Ok((Some(data), headers, status, gap))
                            }
                            Err(e) => Err(ResError {
                                desc: e.into_cause().unwrap().to_string(),
                            }),
                        }
                    }
                    Some(false) => {
                        let (header, _) = response.into_parts();
                        let mut encoding = "".to_string();
                        let status = header.status.as_u16() as usize;
                        let mut headers: HashMap<String, String> = HashMap::new();
                        let cookies: Vec<&str> = header
                            .headers
                            .get_all("set-cookie")
                            .iter()
                            .map(|s| s.to_str().unwrap())
                            .collect();
                        let cookie = cookies.join("::");
                        headers.insert("set-cookie".to_string(), cookie);
                        header
                            .headers
                            .into_iter()
                            .for_each(|(key, value)| match key {
                                None => {}
                                Some(k) => {
                                    let kk = k.to_string();
                                    if kk.to_lowercase() == "content-encoding".to_string() {
                                        encoding = value.to_str().unwrap().to_string();
                                    }
                                    if kk != "set-cookie".to_string() {
                                        headers.insert(kk, value.to_str().unwrap().to_string());
                                    }
                                }
                            });

                        Ok((None, headers, status, gap))
                    }
                }
            }
            Err(e) => {
                let err = if let Some(msg) = e.into_cause() {
                    msg.to_string()
                } else {
                    "Unknow Error".to_string()
                };
                log::error!("cannot exec the request caused by {}.", err);
                Err(ResError { desc: err })
            }
        }
    }

    /// execute only one `Request` for common use.
    pub async fn exec_one<T, P>(req: Request<T, P>) -> Result<Response<T, P>, ResError>
    where
        T: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + Clone,
        P: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + Clone,
    {
        let mut r = Response::default(Some(&req));
        let request = req.init().unwrap();
        let response = Client::exec(request, None).await;

        match response {
            Ok(data) => {
                r.headers.extend(data.1);
                r.content = data.0;
                r.status = data.2;
            }
            Err(e) => {
                r.msg = Some(e.desc);
            }
        }
        Ok(r)
    }

    // FIXME it's not necessary to return Result, Vec<> will be fine.
    /// execute multiple `Request` for common use.
    pub async fn exec_all<T, P>(
        mut reqs: Vec<Request<T, P>>,
        result: Arc<Mutex<Vec<Response<T, P>>>>,
        rate: Arc<Mutex<ArgRate>>,
    ) where
        T: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + Clone,
        P: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + Clone,
    {
        let mut rs = Vec::new();
        let mut ress = Vec::new();
        let mut futs = Vec::new();
        let len_reqs = reqs.len();
        for _ in 0..len_reqs {
            let req = reqs.pop().unwrap();
            rs.push(Response::default(Some(&req)));
            match req.init() {
                Some(r) => futs.push(Client::exec(r, None)),
                None => {
                    rs.remove(0);
                    log::error!("cannot init Request into hyper::Request");
                }
            }
        }

        let mut res = join_all(futs).await;
        let len_rs = rs.len();
        for _ in 0..len_rs {
            let mut r = rs.pop().unwrap();
            let d = res.pop().unwrap();
            match d {
                Ok(da) => {
                    r.content = da.0;
                    r.headers = da.1;
                    r.status = da.2;
                    ress.push(r);
                    rate.lock().unwrap().stamps.push(da.3);
                }
                Err(e) => {
                    r.msg = Some(e.desc);
                    rate.lock().unwrap().err += 1;
                    ress.push(r);
                }
            }
        }
        result.lock().unwrap().extend(ress);
    }

    /// wrapper of futures::future::join_all
    pub async fn join_all<I>(i: I) -> Vec<<<I as IntoIterator>::Item as Future>::Output>
    where
        I: IntoIterator,
        I::Item: Future,
    {
        join_all(i).await
    }
}
