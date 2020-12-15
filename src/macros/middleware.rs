use crate::component::{Profile, Request, Response, Task};
use log::error;

pub trait MiddleWare<T> {
    ///handle status code non-[200, 299]
    ///if the error is not remendable then recycle the the Request
    ///into Task and Profile
    fn hand_err(
        &self,
        res: Response,
    ) -> Option<(Option<Vec<Task>>, Option<Profile>, Option<Request>)>;

    ///handle extracted item from parser
    fn hand_item(&self, items: &mut Vec<T>);

    ///handle constructed Request if necessary
    fn hand_req(&self, req: &mut Request);

    ///handle downloader Response if necessary
    fn hand_res(&self, res: &mut Response);
}

///impl Default for object that implementes MiddleWare 
///if user not manually impl MiddleWare, then this actively used
///basically, just do nothing except print out
pub struct MiddleWareDefault<T>{
    data: std::marker::PhantomData<T>
}

impl<T> MiddleWareDefault<T> {
    pub fn new() -> Self {
        MiddleWareDefault{
            data: std::marker::PhantomData::<T>
        }
    }
}

impl<T> MiddleWare<T> for MiddleWareDefault<T> {
    fn hand_err(&self, res: Response) -> Option<(Option<Vec<Task>>, Option<Profile>, Option<Request>)> {
        error!("response error: {}, uri: {}", res.status, res.uri);
        Some( (None, None, None) )
    }

    fn hand_item(&self, _items: &mut Vec<T>) {}

    fn hand_req(&self, _req: &mut Request) {}

    fn hand_res(&self, _res: &mut Response) {}
}
