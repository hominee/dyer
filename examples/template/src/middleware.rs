// Middleware that processes the data before reaching PipeLine
// including dealing with errors, data structures, runtime modification

use crate::entity::{Entities, Parg, Targ};
use dyer::{plug, App, FutureExt, MiddleWare};

// there are 7 methods availible:
//     1. handle_profile
//     2. handle_task
//     3. handle_req
//     4. handle_res
//     5. handle_item
//     6. handle_err
//     7. handle_yerr
// you can specify some of them if necessary, others are assigned as default
// More details in https://docs.rs/dyer/plugin/middleware/struct.MiddleWare.html

// process Entities if necessary
pub async fn handle_item(_items: &mut Vec<Entities>, _app: &mut App<Entities, Targ, Parg>) {}

pub fn make_middleware<'md>() -> MiddleWare<'md, Entities, Targ, Parg> {
    plug!( MiddleWare<Entities, Targ, Parg> {
        handle_item: handle_item,
    })
}
