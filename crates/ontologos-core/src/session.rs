use crate::reasoner::Profile;

/// Profile-specific incremental reasoning state held by [`crate::Reasoner`].
///
/// Sessions are single-threaded (reasonable's internal state is not `Send`).
pub trait ReasonerSession: std::any::Any + std::fmt::Debug {
    /// OWL profile this session serves.
    fn profile(&self) -> Profile;

    /// Drop cached inference state (e.g. after full rematerialization).
    fn clear(&mut self);

    /// Downcast helper for facade crates.
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}
