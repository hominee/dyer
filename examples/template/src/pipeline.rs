use dyer::dyer_macros::pipeline;

 /*
 * something to do before sending entities to pipeline
 * the return type inside `Option` requires complete path(starts with `std` or crate in `Cargo.toml`)
 * attribute #[pipeline(attr)] mark the method and use it as that in `PipeLine` 
 * attr could be:
 *    open_pipeline/close_pipeline/process_entity/process_yerr
 */
#[pipeline(open_pipeline)]
async fn func_name<'a>() -> &'a Option<std::fs::File> {
    &None
}
