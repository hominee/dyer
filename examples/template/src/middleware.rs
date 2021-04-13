// Middleware that processes the data before reaching PipeLine
// including dealing with errors, data structures, runtime modification

use crate::entity::{Entities, Parg, Targ};
use dyer::{plug, App, FutureExt, MiddleWare};

// there are 7 methods availible:
//     1. hand_profile
//     2. hand_task
//     3. hand_req
//     4. hand_res
//     5. hand_item
//     6. hand_err
//     7. hand_yerr
// you can specify some of them if necessary, others are assigned as default
// More details in https://docs.rs/dyer/plugin/middleware/struct.MiddleWare.html

// process Entities if necessary
pub async fn hand_item(_items: &mut Vec<Entities>, _app: &mut App<Entities, Targ, Parg>) {}

pub fn get_middleware<'md>() -> MiddleWare<'md, Entities, Targ, Parg> {
    plug!( MiddleWare<Entities, Targ, Parg> {
        hand_item: hand_item,
    })
}