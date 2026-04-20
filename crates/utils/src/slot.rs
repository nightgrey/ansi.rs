#[macro_export]
macro_rules! slot {
    ($(#[$meta:meta])* $vis:vis $name:ident, $t:ty) => {
        $(#[$meta])*
        #[derive(derive_more::Deref, derive_more::DerefMut)]
        #[allow(dead_code)]

        $vis struct $name($t);

        impl $name {
            #[inline]
            pub fn slot() -> &'static parking_lot::Mutex<Option<$t>> {
                static SLOT: std::sync::OnceLock<Mutex<Option<$t>>> = std::sync::OnceLock::new();
                SLOT.get_or_init(|| parking_lot::Mutex::new(None))
            }

            #[inline]
            pub fn lock() -> parking_lot::MutexGuard<'static, Option<$t>> {
                Self::slot().lock()
            }

            #[inline]
            pub fn get() -> Option<$t> {
                Self::lock().as_ref().cloned()
            }

            pub fn set(value: $t) {
                let mut slot = Self::lock();

                slot.replace(value);
            }

            pub fn clear() {
                let mut slot = Self::lock();

                slot.take();
            }
        }
    };
}
