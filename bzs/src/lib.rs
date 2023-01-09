use std::{borrow::Cow, ops::Deref};

use encoding::{decode, encode_into, EncodingError};

mod encoding;
pub mod structs;

/// allows setting a numeric value only partially
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct MaskSet {
    mask: u32,
    set: u32,
}

pub trait MaskSetable {
    fn apply(&mut self, mask_set: MaskSet);
}

macro_rules! mask_setable_impl {
    ($ty:ident) => {
        impl MaskSetable for $ty {
            fn apply(&mut self, mask_set: MaskSet) {
                *self = (((*self as u32) & !mask_set.mask) | (mask_set.set & mask_set.mask)) as $ty;
            }
        }
    };
}

mask_setable_impl!(u8);
mask_setable_impl!(u16);
mask_setable_impl!(u32);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct MaskShift {
    mask: u32,
    shift: u32,
}

impl MaskShift {
    pub fn mask_shift_set(&self, value: u32, new_value: u32) -> u32 {
        (!(self.mask << self.shift) & value) | (new_value << self.shift)
    }

    pub fn mask_shift_get(&self, value: u32) -> u32 {
        (value >> self.shift) & self.mask
    }
}

pub fn mask_shift_set(mask_shift: Option<MaskShift>, value: u32, new_value: u32) -> u32 {
    if let Some(mask_shift) = mask_shift {
        mask_shift.mask_shift_set(value, new_value)
    } else {
        // without a mask, set the entire value
        new_value
    }
}

pub fn mask_shift_get(mask_shift: Option<MaskShift>, value: u32) -> u32 {
    if let Some(mask_shift) = mask_shift {
        mask_shift.mask_shift_get(value)
    } else {
        // without a mask, get the entire value
        value
    }
}
