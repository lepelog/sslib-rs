use std::{borrow::Cow, ops::Deref};

use encoding::{EncodingError, encode_into, decode};

mod encoding;
pub mod structs;

#[derive(Debug)]
pub enum Datatype<'a> {
    U32(u32, Option<MaskShift>),
    I32(i32, Option<MaskShift>),
    F32(f32),
    Str(Cow<'a, str>),
    Bytes(Cow<'a, [u8]>),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct MaskShift{
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

#[derive(thiserror::Error, Debug)]
pub enum DatatypeSetError {
    #[error("Incompatible datatypes")]
    InvalidDatatype,
    #[error("Invalid encoding, needs to be encodable to SHIFT-JIS")]
    Encoding,
    #[error("MaskShift is not valid here")]
    UnexpectedMaskShift,
    #[error("Incompatible length: expected {expected}, got {actual}")]
    ExactLen { expected: usize, actual: usize },
    #[error("String too long: expected {expected}, got {actual}")]
    TooLong { expected: usize, actual: usize },
}

#[derive(thiserror::Error, Debug)]
pub enum ContextSetError {
    #[error("Could not set {2} to {0}: {1}")]
    Inner(&'static str, DatatypeSetError, String),
    #[error("Could not find {name} in this struct")]
    NameNotFound { name: String }
}

pub trait SetByName {
    fn set(&mut self, name: &str, data: &Datatype<'_>) -> Result<(), ContextSetError>;
}

pub trait GetByName {
    fn get_u32<'a>(&'a self, name: &str) -> Option<u32>;
}

pub trait DatatypeSetable {
    fn set(&mut self, data: &Datatype<'_>) -> Result<(), DatatypeSetError>;
}

pub trait DatatypeGetable {
    fn get_u32(&self) -> Option<u32> {
        None
    }
    fn get_i32(&self) -> Option<i32> {
        None
    }
    fn get_f32(&self) -> Option<f32> {
        None
    }
    fn get_bytes(&self) -> Option<&[u8]> {
        None
    }
    fn get_string(&self) -> Option<Cow<str>> {
        None
    }
}

impl DatatypeSetable for u32 {
    fn set(&mut self, data: &Datatype<'_>) -> Result<(), DatatypeSetError> {
        *self = match *data {
            Datatype::I32(v, mask_shift) => mask_shift_set(mask_shift, *self, v as u32),
            Datatype::U32(v, mask_shift) => mask_shift_set(mask_shift, *self, v),
            _ => return Err(DatatypeSetError::InvalidDatatype),
        };
        Ok(())
    }
}

impl DatatypeGetable for u32 {
    fn get_u32(&self) -> Option<u32> {
        Some(*self)
    }

    fn get_i32(&self) -> Option<i32> {
        Some(*self as i32)
    }
}

impl DatatypeSetable for i32 {
    fn set(&mut self, data: &Datatype<'_>) -> Result<(), DatatypeSetError> {
        *self = match *data {
            Datatype::I32(v, mask_shift) => mask_shift_set(mask_shift, *self as u32, v as u32) as i32,
            Datatype::U32(v, mask_shift) => mask_shift_set(mask_shift, *self as u32, v) as i32,
            _ => return Err(DatatypeSetError::InvalidDatatype),
        };
        Ok(())
    }
}

impl DatatypeGetable for i32 {
    fn get_u32(&self) -> Option<u32> {
        Some(*self as u32)
    }

    fn get_i32(&self) -> Option<i32> {
        Some(*self)
    }
}

impl DatatypeSetable for u16 {
    fn set(&mut self, data: &Datatype<'_>) -> Result<(), DatatypeSetError> {
        *self = match *data {
            Datatype::I32(v, mask_shift) => mask_shift_set(mask_shift, *self as u32, v as u32) as u16,
            Datatype::U32(v, mask_shift) => mask_shift_set(mask_shift, *self as u32, v) as u16,
            _ => return Err(DatatypeSetError::InvalidDatatype),
        };
        Ok(())
    }
}

impl DatatypeGetable for u16 {
    fn get_u32(&self) -> Option<u32> {
        Some(self.clone().into())
    }

    fn get_i32(&self) -> Option<i32> {
        Some(self.clone().into())
    }
}

impl DatatypeSetable for i16 {
    fn set(&mut self, data: &Datatype<'_>) -> Result<(), DatatypeSetError> {
        *self = match *data {
            Datatype::I32(v, mask_shift) => mask_shift_set(mask_shift, *self as u32, v as u32) as i16,
            Datatype::U32(v, mask_shift) => mask_shift_set(mask_shift, *self as u32, v) as i16,
            _ => return Err(DatatypeSetError::InvalidDatatype),
        };
        Ok(())
    }
}

impl DatatypeGetable for i16 {
    fn get_u32(&self) -> Option<u32> {
        self.clone().try_into().ok()
    }

    fn get_i32(&self) -> Option<i32> {
        Some(self.clone().into())
    }
}

impl DatatypeSetable for u8 {
    fn set(&mut self, data: &Datatype<'_>) -> Result<(), DatatypeSetError> {
        *self = match *data {
            Datatype::I32(v, mask_shift) => mask_shift_set(mask_shift, *self as u32, v as u32) as u8,
            Datatype::U32(v, mask_shift) => mask_shift_set(mask_shift, *self as u32, v) as u8,
            _ => return Err(DatatypeSetError::InvalidDatatype),
        };
        Ok(())
    }
}

impl DatatypeGetable for u8 {
    fn get_u32(&self) -> Option<u32> {
        Some(self.clone().into())
    }

    fn get_i32(&self) -> Option<i32> {
        Some(self.clone().into())
    }
}

impl DatatypeSetable for i8 {
    fn set(&mut self, data: &Datatype<'_>) -> Result<(), DatatypeSetError> {
        *self = match *data {
            Datatype::I32(v, mask_shift) => mask_shift_set(mask_shift, *self as u32, v as u32) as i8,
            Datatype::U32(v, mask_shift) => mask_shift_set(mask_shift, *self as u32, v) as i8,
            _ => return Err(DatatypeSetError::InvalidDatatype),
        };
        Ok(())
    }
}

impl DatatypeGetable for i8 {
    fn get_u32(&self) -> Option<u32> {
        self.clone().try_into().ok()
    }

    fn get_i32(&self) -> Option<i32> {
        Some(self.clone().into())
    }
}

impl DatatypeSetable for f32 {
    fn set(&mut self, data: &Datatype<'_>) -> Result<(), DatatypeSetError> {
        *self = match *data {
            Datatype::F32(v) => v,
            // TODO: implicitly ignore mask_shift or error?
            Datatype::I32(v, _) => v as f32,
            Datatype::U32(v, _) => v as f32,
            _ => return Err(DatatypeSetError::InvalidDatatype),
        };
        Ok(())
    }
}

impl DatatypeGetable for f32 {
    fn get_f32(&self) -> Option<f32> {
        Some(*self)
    }
}

impl <const N: usize> DatatypeSetable for [u8; N] {
    fn set(&mut self, data: &Datatype<'_>) -> Result<(), DatatypeSetError> {
        match data {
            Datatype::Bytes(v) => {
                if v.len() == self.len() {
                    self.copy_from_slice(v.as_ref());
                } else {
                    return Err(DatatypeSetError::ExactLen {expected: self.len(), actual: v.len()});
                }
            },
            Datatype::Str(v) => {
                encode_into(v, self).map_err(|e| {
                    match e {
                        EncodingError::InvalidShiftJis => DatatypeSetError::Encoding,
                        EncodingError::TooLong => DatatypeSetError::TooLong { expected: self.len(), actual: v.len() },
                    }
                })?;
            },
            _ => return Err(DatatypeSetError::InvalidDatatype),
        };
        Ok(())
    }
}

impl <const N: usize> DatatypeGetable for [u8; N] {
    fn get_bytes(&self) -> Option<&[u8]> {
        Some(self)
    }

    fn get_string(&self) -> Option<Cow<str>> {
        decode(self)
    }
}

#[derive(sslib_proc::SetByName)]
struct Obj {
    params1: u32,
    params2: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    pub fn works() {
        let mut a: u32 = 0;
        a.set(&Datatype::I32(4, None)).unwrap();
        assert_eq!(a, 4);
        a.set(&Datatype::U32(4321, None)).unwrap();
        assert_eq!(a, 4321);
        assert!(a.set(&Datatype::I32(-1, None)).is_err());
        assert!(a.set(&Datatype::Bytes(Cow::Borrowed(&[1,2,4]))).is_err());
    }
}
