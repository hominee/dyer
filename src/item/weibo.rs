use crate::spider::Entity;
use crate::{Profile, Request, Response, Task};

pub struct PaerseResult {
    pub req: Option<Request>,
    pub res: Option<Response>,
    pub task: Option<Task>,
    pub profile: Option<Profile>,
    pub entities: Option<Vec<Entity>>,
}
