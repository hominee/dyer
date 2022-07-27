//! the [Client] that asynchronously executes [Request], with specified `connect-timeout`, `read-timeout` and
//! `write-timeout`.
//!
//! Note that polling the `[Request]`s requires `tokio::runtime`.

use crate::component::Body;
use crate::component::{utils, Request, Response};
use crate::request::Exts;
use crate::response::InnerResponse;
use crate::response::MetaResponse;
use futures_util::{future::join_all, Future};
use http::Extensions;
use hyper::body::Buf;
use hyper::client::HttpConnector;
#[cfg(feature = "proxy")]
use hyper_proxy::ProxyConnector;
use hyper_tls::HttpsConnector;
use std::collections::HashMap;
use std::io::{BufReader, Read};

//type MClient = hyper::Client<HttpsConnector<HttpConnector>>;
type ClientPlain = hyper::Client<HttpsConnector<HttpConnector>>;
#[cfg(feature = "proxy")]
type ClientProxy = hyper::Client<ProxyConnector<HttpConnector>>;

pub enum ClientType {
    Plain(ClientPlain),
    #[cfg(feature = "proxy")]
    Proxy(ClientProxy),
}

pub static mut CLIENTPOOL: Option<HashMap<u64, Client>> = None;

// TODO add proxy support
/// Client that take [Request] and execute, return [Response]
///
/// NOTE that not all `content-encoding` supported, it only supports as following
/// - plain-text (not compressed)
/// - gzip
/// - br
/// - deflate
pub struct Client {
    pub id: u64,
    pub(crate) inner: ClientType,
}

impl Client {
    pub fn new_plain() -> &'static Client {
        let id = 0;
        unsafe {
            if let Some(d) = CLIENTPOOL.as_ref().and_then(|pool| pool.get(&id)) {
                return d;
            }
        }
        let https = HttpsConnector::new();
        let client = hyper::Client::builder().build::<_, hyper::Body>(https);
        let downloader = Client {
            id,
            inner: ClientType::Plain(client),
        };
        unsafe {
            match CLIENTPOOL {
                None => {
                    let mut pool = HashMap::new();
                    pool.insert(id, downloader);
                    CLIENTPOOL = Some(pool);
                }
                Some(ref mut pool) => {
                    pool.insert(id, downloader);
                }
            }
            CLIENTPOOL.as_ref().unwrap().get(&id).unwrap()
        }
    }

    /*
     * /// new static client
     * fn new() -> &'static MClient {
     *    static INIT: Once = Once::new();
     *    static mut VAL: Option<MClient> = None;
     *    unsafe {
     *        INIT.call_once(|| {
     *            let https: HttpsConnector<HttpConnector> = HttpsConnector::new();
     *            let item = hyper::Client::builder().build::<_, hyper::Body>(https);
     *            VAL = Some(item);
     *        });
     *        VAL.as_ref().unwrap()
     *    }
     *}
     */

    /*
     *    /// this function requires a `hyper::Request` and `hyper::Client` to return the Response
     *    /// Poll the `Request`, and asynchronously aggregate data from
     *    /// server.
     *    pub async fn exec(
     *        req: hyper::Request<hyper::Body>,
     *    ) -> Result<(hyper::Response<Body>, f64), Box<dyn std::error::Error>> {
     *        let client = Client::new();
     *        let tic = utils::now();
     *        match client.request(req).await {
     *            Ok(response) => {
     *                let toc = utils::now();
     *                let gap = toc - tic;
     *                let (header, bd) = response.into_parts();
     *                let bod = hyper::body::aggregate(bd).await;
     *                match bod {
     *                    Ok(body) => {
     *                        //let mut data = Bytes::from(body.bytes());
     *                        let mut reader = BufReader::new(body.reader());
     *                        // Response Content
     *                        let mut data = Vec::new();
     *                        if let Some(t) = header.headers.get("content-encoding") {
     *                            match t.to_str() {
     *                                #[cfg(feature = "compression")]
     *                                Ok("gzip") | Ok("deflate") => {
     *                                    let mut gz = flate2::read::GzDecoder::new(reader);
     *                                    gz.read_to_end(&mut data).unwrap();
     *                                }
     *                                #[cfg(feature = "compression")]
     *                                Ok("br") => {
     *                                    let mut br = brotli2::read::BrotliDecoder::new(reader);
     *                                    br.read_to_end(&mut data).unwrap();
     *                                }
     *                                _ => {
     *                                    reader.read_to_end(&mut data).unwrap();
     *                                }
     *                            }
     *                        } else {
     *                            reader.read_to_end(&mut data).unwrap();
     *                        }
     *
     *                        let body = Body::from(data);
     *                        let res = hyper::Response::from_parts(header, body);
     *                        Ok((res, gap))
     *                    }
     *                    Err(e) => Err(e.into()),
     *                }
     *            }
     *            Err(e) => {
     *                if e.is_canceled() {
     *                    log::error!("Timeout request: {:?}", e);
     *                } else {
     *                    log::error!("Failed request: {:?}", e);
     *                }
     *                Err(e.into())
     *            }
     *        }
     *    }
     */

    /// this function requires a `Request` and `hyper::Client` to return the Response
    /// Poll the `Request`, and asynchronously aggregate data from
    /// server.
    pub async fn request(&self, req: Request) -> Result<Response, MetaResponse> {
        let (mta, req, ext_t, ext_p) = req.into();
        let mut mta = MetaResponse::from(mta);
        let tic = utils::now();
        let result = match self.inner {
            ClientType::Plain(ref client) => client.request(req).await,
            #[cfg(feature = "proxy")]
            ClientType::Proxy(ref client) => client.request(req).await,
        };
        let toc = utils::now();
        match result {
            Ok(response) => {
                let (parts, body_future) = response.into_parts();
                let bod = hyper::body::aggregate(body_future).await;
                match bod {
                    Ok(body) => {
                        //let mut data = Bytes::from(body.bytes());
                        let mut reader = BufReader::new(body.reader());
                        // Response Content
                        let mut data = Vec::new();
                        if let Some(t) = parts.headers.get("content-encoding") {
                            match t.to_str() {
                                #[cfg(feature = "compression")]
                                Ok("gzip") | Ok("deflate") => {
                                    let mut gz = flate2::read::GzDecoder::new(reader);
                                    gz.read_to_end(&mut data).unwrap();
                                }
                                #[cfg(feature = "compression")]
                                Ok("br") => {
                                    let mut br = brotli2::read::BrotliDecoder::new(reader);
                                    br.read_to_end(&mut data).unwrap();
                                }
                                _ => {
                                    reader.read_to_end(&mut data).unwrap();
                                }
                            }
                        } else {
                            reader.read_to_end(&mut data).unwrap();
                        }

                        let body = Body::from(data);
                        let inn = InnerResponse {
                            status: parts.status,
                            version: parts.version,
                            headers: parts.headers,
                            extensions: Exts(ext_t, ext_p, Extensions::new(), parts.extensions),
                        };
                        mta.info.gap = toc - tic;
                        let ret = Response::from_parts(inn, body, mta);
                        Ok(ret)
                    }
                    Err(_) => Err(mta),
                    //Err(e) => Err(e.into()),
                }
            }
            Err(e) => {
                if format!("{:?}", e).contains("Cancelled") {
                    log::error!("Timeout request: {:?}", e);
                } else {
                    log::error!("Failed request: {:?}", e);
                }
                Err(mta)
            }
        }
    }

    /*
     * /// execute only one `Request` for common use.
     *pub async fn exec_one(req: Request) -> Result<Response, MetaResponse> {
     *    let (mta, request, ext_t, ext_p) = req.into();
     *    let mut mta = MetaResponse::from(mta);
     *    let response = Client::exec(request).await;
     *    match response {
     *        Ok(data) => {
     *            let (parts, body) = data.0.into_parts();
     *            let inn = InnerResponse {
     *                status: parts.status,
     *                version: parts.version,
     *                headers: parts.headers,
     *                extensions: Exts(ext_t, ext_p, Extensions::new(), parts.extensions),
     *            };
     *            mta.info.gap = data.1;
     *            let ret = Response::from_parts(inn, body, mta);
     *            Ok(ret)
     *        }
     *        Err(_) => Err(mta),
     *    }
     *}
     */

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
