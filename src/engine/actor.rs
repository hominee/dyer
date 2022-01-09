//! Stands for a project, Set up initial condition and necessary component to start the workflow
//!
use crate::component::Task;
use crate::engine::App;
use crate::plugin::Affixor;
use async_trait::async_trait;
use std::error::Error;

/// Although it starts a project, the work to do here is not very complicated
/// it is as simple as setting up initial condition and other basic things.
///
/// Right here, if entry_affix returns None, then [Affix] is disabled
///
/// [Affix]: crate::component::affix::Affix
#[async_trait]
pub trait Actor<E, A>
where
    A: Affixor + Send + 'static,
{
    /// create a instance
    async fn new() -> Self
    where
        Self: Sized;

    /// implementation [Affixor] happens here
    /// when None is returned actor will disable affix customization
    async fn entry_affix(&mut self) -> Option<A>;

    /// method to generate [Task] when open [Actor]
    async fn entry_task(&mut self) -> Result<Vec<Task>, Box<dyn Error>>;

    /// preparation before enter [Actor]
    async fn open_actor(&mut self, app: &mut App<E>);

    /// preparation before close [Actor]
    async fn close_actor(&mut self, app: &mut App<E>);
}
