use crate::entity::*;
use dyer::*;

/* note that call this function to parse via specifying task.parser:
 *     let task = Task::builder();
 *         ...
 *         .parser(parse_func)
 *         .body(Body::empty(), "actor_marker".into())
 *         .unwrap();
 * that means function `parse_func` is called to parse the Response.
 * attribute #[dyer::parser] mark the method and use it extract entities from `Response` whose
 * parser is  parse_func
 */
#[dyer::parser]
pub fn parse_func(_res: Response) -> Parsed<Entities> {
    Parsed::new()
}