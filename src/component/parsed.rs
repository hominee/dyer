//! The result of parser when parsing [Response]
//!
//! [Response]: crate::component::Response
use crate::component::MetaResponse;
use crate::component::{Affix, Request, Task};
use crate::Response;

/// the parsed result returned by `parser`.
pub struct Parsed<E> {
    /// a vector of `Request`
    pub req: Vec<Request>,
    /// a vector of `Task`
    pub task: Vec<Task>,
    /// a vector of `Affix`
    pub affix: Vec<Affix>,
    /// a vector of customized `Entity`
    pub entities: Vec<E>,
    /// a vector of record for failed `Response`, for the use of debug.
    pub errs: Vec<Result<Response, MetaResponse>>,
}

impl<E> Parsed<E> {
    pub fn new() -> Self {
        Parsed {
            task: Vec::new(),
            affix: Vec::new(),
            req: Vec::new(),
            entities: Vec::new(),
            errs: Vec::new(),
        }
    }
}
