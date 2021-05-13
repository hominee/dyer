use crate::entity::{Entities, Parg, Targ};
use dyer::App;
use dyer::dyer_macros::middleware;

/* attribute #[middleware(attr)] mark the method and use it as that in `MiddleWare`
 * attr could be :
 *    handle_entity/handle_req/handle_task/handle_profile
 *    /handle_res/handle_err/handle_yerr
 */
#[middleware(handle_entity)]
pub async fn handle_entities(_items: &mut Vec<Entities>, _app: &mut App<Entities, Targ, Parg>) {}
