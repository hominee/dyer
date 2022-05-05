//! the [Client] that asynchronously executes [Request], with specified `connect-timeout`, `read-timeout` and
//! `write-timeout`.
//!
//! Note that polling the `[Request]`s requires `tokio::runtime`.

use crate::component::Body;
use crate::component::{utils, Request, Response};
use crate::request::Exts;
use crate::response::InnerResponse;
use crate::response::MetaResponse;
use bytes::buf::ext::BufExt;
//use futures_executor::block_on;
use futures_util::{future::join_all, Future};
use http::Extensions;
use hyper::client::HttpConnector;
use hyper_timeout::TimeoutConnector;
use hyper_tls::HttpsConnector;
use std::io::{BufReader, Read};
use std::sync::Once;

type MClient = hyper::Client<TimeoutConnector<HttpsConnector<HttpConnector>>>;

// TODO add proxy support
/// Client that take [Request] and execute, return [Response]
///
/// NOTE that not all `content-encoding` supported, it only supports as following
/// - plain-text (not compressed)
/// - gzip
/// - br
/// - deflate
pub struct Client;

impl Client {
    /// new static client
    fn new(con: u64, read: u64, write: u64) -> &'static MClient {
        static INIT: Once = Once::new();
        static mut VAL: Option<MClient> = None;
        unsafe {
            INIT.call_once(|| {
                let https: HttpsConnector<HttpConnector> = HttpsConnector::new();
                let mut conn = hyper_timeout::TimeoutConnector::new(https);
                conn.set_connect_timeout(Some(std::time::Duration::from_secs(con)));
                conn.set_read_timeout(Some(std::time::Duration::from_secs(read)));
                conn.set_write_timeout(Some(std::time::Duration::from_secs(write)));
                let item = hyper::Client::builder().build::<_, hyper::Body>(conn);
                VAL = Some(item);
            });
            VAL.as_ref().unwrap()
        }
    }

    /// this function requires a `hyper::Request` and `hyper::Client` to return the Response
    /// Poll the `Request`, and asynchronously aggregate data from
    /// server.
    pub async fn exec(
        req: hyper::Request<hyper::Body>,
    ) -> Result<(hyper::Response<Body>, f64), Box<dyn std::error::Error>> {
        let client = Client::new(7, 23, 7);
        let tic = utils::now();
        match client.request(req).await {
            Ok(response) => {
                let toc = utils::now();
                let gap = toc - tic;
                let (header, bd) = response.into_parts();
                let bod = hyper::body::aggregate(bd).await;
                match bod {
                    Ok(body) => {
                        //let mut data = Bytes::from(body.bytes());
                        let mut reader = BufReader::new(body.reader());
                        // Response Content
                        let mut data = String::new();
                        //let mut data = Chunk::new();
                        let encodings = header.headers.get("content-encoding");
                        let mut encode = 2;
                        if let Some(t) = encodings {
                            let t = t.to_str().unwrap();
                            if ["gzip", "deflate"].contains(&t) {
                                encode = 0;
                            } else if t == "br" {
                                encode = 1;
                            }
                        }

                        if encode == 0 {
                            let mut gz = flate2::read::GzDecoder::new(reader);
                            gz.read_to_string(&mut data).unwrap();
                            //gz.read(&mut *data).unwrap();
                        } else if encode == 1 {
                            let mut br = brotli2::read::BrotliDecoder::new(reader);
                            br.read_to_string(&mut data).unwrap();
                            //br.read(&mut *data).unwrap();
                        } else {
                            reader.read_to_string(&mut data).unwrap();
                            //reader.read(&mut *data).unwrap();
                        }
                        let body = Body::from(data);
                        let res = hyper::Response::from_parts(header, body);
                        Ok((res, gap))
                    }
                    Err(e) => Err(e.into()),
                }
            }
            Err(e) => {
                log::error!("Failed request: {:?}", e);
                Err(e.into())
            }
        }
    }

    /// execute only one `Request` for common use.
    pub async fn exec_one(req: Request) -> Result<Response, MetaResponse> {
        let (mta, request, ext_t, ext_p) = req.into();
        let mut mta = MetaResponse::from(mta);
        let response = Client::exec(request).await;
        match response {
            Ok(data) => {
                let (parts, body) = data.0.into_parts();
                let inn = InnerResponse {
                    status: parts.status,
                    version: parts.version,
                    headers: parts.headers,
                    extensions: Exts(ext_t, ext_p, Extensions::new(), parts.extensions),
                };
                mta.info.gap = data.1;
                let ret = Response::from_parts(inn, body, mta);
                Ok(ret)
            }
            Err(_) => Err(mta),
        }
    }

    /// A wrapper of futures's function block_on
    ///
    /// blocking the current thread and execute the future
    ///
    /// NOTE that avoid using this if not necessary
    /// spawn a task or use join_all instead
    ///
    /*
     *pub fn block_exec<F: Future>(f: F) -> F::Output {
     *    block_on(f)
     *}
     */

    /// A wrapper of futures's function join_all
    ///
    /// execute multiple `Request` for common use.
    ///
    pub async fn join_all<I>(i: I) -> Vec<<<I as IntoIterator>::Item as Future>::Output>
    where
        I: IntoIterator,
        I::Item: Future,
    {
        join_all(i).await
    }
}
