// Parsers that extract entities from Response
// external tool may be used to achieve that

use crate::entity::{Entities, Parg, Targ};
use dyer::{ParseResult, Response};

// note that call this function to parse via specifying:
//     let task = Task::default();
//     ...
//     task.parser = "parse_index".to_string();
// that means function `parse_index` is called once the task being executed successfully and
// becoming response.
pub fn parse_index(_res: Response<Targ, Parg>) -> ParseResult<Entities, Targ, Parg> {
    ParseResult::default()
}