/// Trait to encapsulate behaviour to resolve a potentially context-dependent value
/// into a context-independent value.
pub trait Resolve<Into, Context> {
    /// Resolve a value.
    fn resolve(self, context: Context) -> Into;
}
