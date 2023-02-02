#[doc(inline)]
pub use transtype_impl::*;

#[doc(hidden)]
pub mod private {
    use crate::Wrapper;

    pub enum InnerType {}

    pub const fn requires_wrapper<T: Wrapper<InnerType>>() {}
}

pub trait Wrapper<T> {
    fn wrap(value: T) -> Self;
    fn unwrap(self) -> T;
}

impl<T> Wrapper<T> for Option<T> {
    fn wrap(value: T) -> Self {
        Some(value)
    }

    fn unwrap(self) -> T {
        self.unwrap()
    }
}

pub trait Wrapped: Sized {
    type Original;

    fn unwrap(self) -> Self::Original;
}
