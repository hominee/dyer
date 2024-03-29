use crate::entity::*;
use dyer::*;

/* attribute #[dyer::middleware(attr)] mark the method and use it as that in `MiddleWare`
 * attr could be :
 *    handle_entity/handle_req/handle_task/handle_affix
 *    /handle_res/handle_err/handle_yerr
 */
#[dyer::middleware(handle_entity)]
pub async fn handle_entities(_items: &mut Vec<Entities>, _app: &mut App<Entities>) {}
