use crate::item::{Profile, Request, Response, Task};

///handle status code between 100-199
///if the error is not remendable then recycle the the Request
///into Task and Profile
pub fn hand100(_res: Response) -> Option<Request> {
    Some(Request::default())
}

///handle status code within 300 - 399
pub fn hand300(_res: Response) -> Option<(Task, Profile)> {
    None
}

///handle status code within 400 - 499
pub fn hand400(_res: Response) -> Option<(Task, Profile)> {
    None
}

///handle status code within 500 - 599
pub fn hand500(_res: Response) -> Option<(Task, Profile)> {
    None
}

///handle status code within 500 - 599
pub fn hand0(_res: Response) -> Option<(Task, Profile)> {
    None
}
