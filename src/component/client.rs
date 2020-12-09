use crate::component::{Request, ResError, Response};
use bytes::buf::ext::BufExt;
use futures::future::join_all;
use hyper::{client::HttpConnector, Body as hBody, Client as hClient, Request as hRequest};
use hyper_timeout::TimeoutConnector;
use hyper_tls::HttpsConnector;
use std::collections::HashMap;
use std::io::{BufReader, Read};
use std::sync::{Arc, Mutex, Once};
use tokio::task;

pub type MClient = hClient<TimeoutConnector<HttpsConnector<HttpConnector>>>;

pub struct Client;

impl Client {
    /// new static client
    pub fn new(con: u64, read: u64, write: u64) -> &'static Vec<MClient> {
        static INIT: Once = Once::new();
        static mut VAL: Vec<MClient> = Vec::new();
        unsafe {
            INIT.call_once(|| {
                let https: HttpsConnector<HttpConnector> = HttpsConnector::new();
                let mut conn = hyper_timeout::TimeoutConnector::new(https);
                conn.set_connect_timeout(Some(std::time::Duration::from_secs(con)));
                conn.set_read_timeout(Some(std::time::Duration::from_secs(read)));
                conn.set_write_timeout(Some(std::time::Duration::from_secs(write)));
                let clt = hClient::builder().build::<_, hBody>(conn);
                VAL.push(clt);
            });
            &VAL
        }
    }
}

impl Client {
    ///this function require a `hyper::Request` and `hyper::Client` to return the Response

    pub async fn exec(
        req: hRequest<hBody>,
    ) -> Result<(Option<String>, HashMap<String, String>, usize), ResError> {
        let client = &Client::new(7, 23, 7)[0];
        let response = client.request(req).await.unwrap();
        let (header, bd) = response.into_parts();
        let bod = hyper::body::aggregate(bd).await;
        match bod {
            Ok(body) => {
                let mut reader = BufReader::new(body.reader());
                let status = header.status.as_u16() as usize;
                let mut headers: HashMap<String, String> = HashMap::new();
                header.headers.into_iter().for_each(|(key, value)| {
                    headers.insert(
                        key.unwrap().to_string(),
                        value.to_str().unwrap().to_string(),
                    );
                });

                // Response Content
                let mut data = String::new();
                reader.read_to_string(&mut data).unwrap();

                Ok((Some(data), headers, status))
            }
            Err(e) => Err(ResError {
                desc: e.into_cause().unwrap().source().unwrap().to_string(),
            }),
        }
    }

    pub async fn exec_one(req: Request) -> Result<Response, ResError> {
        let mut r = Response::default(Some(&req));
        let req = req.init().unwrap();
        let response = Client::exec(req).await;

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
    pub async fn exec_all(reqs: Vec<Request>, result: Arc<Mutex<Vec<Response>>>) {
        let mut v = Vec::new();
        let mut rs = Vec::new();
        reqs.into_iter().for_each(|req| {
            rs.push(Response::default(Some(&req)));
            if let Some(r) = req.init() {
                v.push(r);
            }
        });

        let mut futs = Vec::new();
        v.into_iter().for_each(|req| {
            let fut = Client::exec(req);
            futs.push(fut);
        });
        let mut res = join_all(futs).await;
        for _ in 0..rs.len() {
            let mut r = rs.pop().unwrap();
            let d = res.pop().unwrap();
            match d {
                Ok(da) => {
                    r.content = da.0;
                    r.headers = da.1;
                    r.status = da.2;
                }
                Err(e) => {
                    r.msg = Some(e.desc);
                }
            }
        }
        result.lock().unwrap().extend(rs);
    }

    ///join spawned tokio-task
    pub async fn join(
        res: Arc<Mutex<Vec<(u64, task::JoinHandle<()>)>>>,
        pfile: Arc<Mutex<Vec<(u64, task::JoinHandle<()>)>>>,
    ) {
        let mut ind_r: Vec<usize> = Vec::new();
        let mut handle_r = Vec::new();
        let mut j = 0;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u64;
        res.lock().unwrap().iter().enumerate().for_each(|(ind, r)| {
            if now - r.0 >= 30 {
                ind_r.push(ind - j);
                j += 1;
            }
        });
        ind_r.into_iter().for_each(|ind| {
            let (_, handle) = res.lock().unwrap().remove(ind);
            handle_r.push(handle)
        });

        let mut ind_p: Vec<usize> = Vec::new();
        let mut j = 0;
        pfile
            .lock()
            .unwrap()
            .iter()
            .enumerate()
            .for_each(|(ind, r)| {
                if now - r.0 >= 30 {
                    ind_p.push(ind - j);
                    j += 1;
                }
            });
        ind_p.into_iter().for_each(|ind| {
            let (_, handle) = pfile.lock().unwrap().remove(ind);
            handle_r.push(handle)
        });
        join_all(handle_r).await;
    }
}
