use crate::entity::{Entities, Parg, Targ};
use dyer::{ParseResult, Response};
use dyer::dyer_macros::parser;

/* note that call this function to parse via specifying task.parser:
 *     let task = Task::new();
 *     ...
 *     task.parser = "parse_func".into();
 * that means function `parse_func` is called to parse the Response.
 * attribute #[parser] mark the method and use it extract entities from `Response` marked with
 * string "parse_func"
 */
#[parser]
pub fn parse_func(_res: Response<Targ, Parg>) -> ParseResult<Entities, Targ, Parg> {
    ParseResult::new()
}