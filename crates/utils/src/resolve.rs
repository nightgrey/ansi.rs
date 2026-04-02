/// Resolve a context-dependent value.
pub trait Resolve<T, Ctx> {
    /// Resolve a value.
    fn resolve(self, ctx: Ctx) -> T;
}

/// Automatically implements a contextual resolve, allowing `Ctx::resolve(self) -> T`.
pub trait ContextResolve<S, T> {
    /// Resolve a value.
    ///
    /// Panics if the value is unable to resolve.
    #[inline(always)]
    fn resolve(self, value: S) -> T;
}

impl<T, Ctx, S> ContextResolve<S, T> for Ctx
where
    S: Resolve<T, Ctx>,
{
    fn resolve(self, value: S) -> T {
        S::resolve(value, self)
    }
}
