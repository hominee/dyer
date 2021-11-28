use dyer::*;
use crate::entity::*;

 /*
 * something to do before sending entities to pipeline
 * the return type inside `Option` requires complete path(starts with `std` or crate in `Cargo.toml`)
 * attribute #[dyer::pipeline(attr)] mark the method and use it as that in `PipeLine` 
 * attr could be:
 *    initializer/disposer/process_entity/process_yerr
 */
#[dyer::pipeline(initializer)]
async fn func_name(_app: &mut App<Entities>) -> Option<std::fs::File> 
{
    None
}
