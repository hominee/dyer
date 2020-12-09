use crate::component::{Profile, Request, Response, Task};

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
