use crate::component::{Client, Profile, Request, Response, Task};

pub trait MiddleWare<U, T, P>
where
    T: serde::Serialize + for<'a> serde::Deserialize<'a> + std::fmt::Debug + Clone,
    P: serde::Serialize + for<'a> serde::Deserialize<'a> + std::fmt::Debug + Clone,
{
    ///handle status code non-[200, 299]
    ///if the error is not remendable then recycle the the Request
    ///into Task and Profile
    fn hand_err(
        &self,
        res: Response<T, P>,
    ) -> Option<(
        Option<Vec<Task<T>>>,
        Option<Profile<P>>,
        Option<Request<T, P>>,
        Option<String>,
        bool,
    )>;

    ///handle extracted item from parser
    fn hand_item(&self, items: &mut Vec<U>);

    ///handle constructed Request if necessary
    fn hand_req(
        &self,
        req: Request<T, P>,
    ) -> (Option<Request<T, P>>, Option<Profile<P>>, Option<Task<T>>);

    ///handle downloader Response if necessary
    fn hand_res(&self, res: &mut Response<T, P>);
}

///impl Default for object that implementes MiddleWare
///if user not manually impl MiddleWare, then this actively used
///basically, just do nothing except print out
pub struct MiddleWareDefault<U> {
    data: std::marker::PhantomData<U>,
}

impl<U> MiddleWareDefault<U> {
    pub fn new() -> Self {
        MiddleWareDefault {
            data: std::marker::PhantomData::<U>,
        }
    }
}

impl<U, T, P> MiddleWare<U, T, P> for MiddleWareDefault<U>
where
    T: std::fmt::Debug + serde::Serialize + for<'de> serde::Deserialize<'de> + Clone,
    P: std::fmt::Debug + serde::Serialize + for<'de> serde::Deserialize<'de> + Clone,
{
    fn hand_err(
        &self,
        res: Response<T, P>,
    ) -> Option<(
        Option<Vec<Task<T>>>,
        Option<Profile<P>>,
        Option<Request<T, P>>,
        Option<String>,
        bool,
    )> {
        log::error!("response error: {}, uri: {}", res.status, res.uri);
        let mut redirect = false;
        if let Some(content) = res.content.as_ref() {
            redirect = content.contains("unhuman?type=unhuman")
                || content.contains("need_login")
                || content.contains("anticrawl/captcha_appeal");
            if redirect {
                let client = Client::new(7, 23, 7).as_ref().unwrap();
                let req = hyper::Request::builder()
                    .uri("http://127.0.0.1:5000")
                    .body(hyper::Body::default())
                    .unwrap();
                match Client::block_exec(client.request(req)) {
                    Ok(_) => {
                        log::info!("unlock the zhihu captcha successfully.");
                    }
                    Err(_) => {
                        log::error!("cannot execute the unlock operation or operation failed.");
                        let duration = std::time::Duration::from_secs(30);
                        std::thread::sleep(duration);
                        return self.hand_err(res);
                    }
                }
            }
        }
        let (mut tsk, pfile) = res.into1(3).unwrap();
        if tsk.trys >= 2 {
            let yield_err = format!(
                "status: {}\turi: {}\tcontent: {}",
                &res.status,
                &res.uri,
                res.content.as_ref().unwrap_or(&"".to_string())
            );
            log::error!("this task fails 3+ times. drop it.");
            Some((None, Some(pfile), None, Some(yield_err), redirect))
        } else {
            log::error!("{} times failure, reuse this task.", tsk.trys);
            tsk.trys += 1;
            Some((Some(vec![tsk]), Some(pfile), None, None, redirect))
        }
    }

    fn hand_item(&self, _items: &mut Vec<U>) {}

    fn hand_req(
        &self,
        req: Request<T, P>,
    ) -> (Option<Request<T, P>>, Option<Profile<P>>, Option<Task<T>>) {
        let cookie = req.cookie.as_ref().unwrap();
        let v1 = cookie.contains_key("cap_id")
            && cookie.contains_key("r_cap_id")
            && cookie.contains_key("l_cap_id");
        let mut v2: bool = true;
        if req.parser == "parse_dis".to_string() || req.parser == "parse_essence".to_string() {
            v2 = req.targs.is_some();
        }
        if v1 && v2 {
            return (Some(req), None, None);
        } else {
            let (pfile, tsk) = Request::into1(req);
            if !v1 && v2 {
                log::warn!("invalid profile in request");
                return (None, None, Some(tsk));
            } else if v1 && !v2 {
                log::warn!("invalid task in request");
                return (None, Some(pfile), None);
            } else {
                log::error!("both task and profile are invalid in request");
                return (None, None, None);
            }
        }
    }

    fn hand_res(&self, _res: &mut Response<T, P>) {}
}
