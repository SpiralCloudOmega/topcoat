use ref_cast::{RefCastCustom, ref_cast_custom};

use crate::runtime::{Signal, Surrogated, impl_surrogate, impl_surrogate_ref};

#[derive(RefCastCustom, Clone, Copy)]
#[repr(transparent)]
pub struct WriteSignal<'a, T>(Signal<'a, T>);

impl<'a, T> WriteSignal<'a, T> {
    #[inline]
    pub(crate) const fn new(v: Signal<'a, T>) -> Self {
        Self(v)
    }

    #[ref_cast_custom]
    pub(crate) const fn ref_cast<'b>(v: &'b Signal<'a, T>) -> &'b Self;
}

impl<'a, T> WriteSignal<'a, T>
where
    T: Surrogated,
    for<'b> &'b T: Surrogated,
{
    pub fn read(&self) -> <&T as Surrogated>::Surrogate {
        self.0.read().into_surrogate()
    }

    pub fn set(&self, _v: T::Surrogate) {
        panic!("signals cannot be written to inside of a server-side expression");
    }
}

impl_surrogate!({'a, T} Signal<'a, T>, WriteSignal<'a, T>);
impl_surrogate_ref!({'a, T} Signal<'a, T>, WriteSignal<'a, T>);
