use ref_cast::{RefCastCustom, ref_cast_custom};

use crate::runtime::{Surrogated, impl_surrogate, impl_surrogate_mut, impl_surrogate_ref};

#[derive(RefCastCustom, Clone, Copy)]
#[repr(transparent)]
#[allow(non_camel_case_types)]
pub struct f64(core::primitive::f64);

impl f64 {
    #[inline]
    pub(crate) const fn new(v: core::primitive::f64) -> Self {
        Self(v)
    }

    #[ref_cast_custom]
    pub(crate) const fn ref_cast(v: &core::primitive::f64) -> &Self;
    #[ref_cast_custom]
    pub(crate) const fn ref_cast_mut(v: &mut core::primitive::f64) -> &mut Self;
}

impl_surrogate!(core::primitive::f64, f64);
impl_surrogate_ref!(core::primitive::f64, f64);
impl_surrogate_mut!(core::primitive::f64, f64);

macro_rules! impl_binary_op {
    ($trait:ident, $method:ident, $op:tt) => {
        impl core::ops::$trait for f64 {
            type Output = f64;

            #[inline]
            fn $method(self, rhs: f64) -> f64 {
                f64(self.0 $op rhs.0)
            }
        }
    };
}

impl_binary_op!(Add, add, +);
impl_binary_op!(Sub, sub, -);
impl_binary_op!(Mul, mul, *);
impl_binary_op!(Div, div, /);
