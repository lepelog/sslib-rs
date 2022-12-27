use std::{borrow::Cow, ops::Deref, iter::repeat};

use binrw::{BinRead, BinWrite};
use encoding_rs::SHIFT_JIS;

#[derive(thiserror::Error, Debug)]
pub enum EncodingError {
    #[error("Invalid Shift-Jis")]
    InvalidShiftJis,
    #[error("Input too long")]
    TooLong,
}

pub fn decode(buf: &[u8]) -> Option<Cow<str>> {
    let str_end = buf.iter().position(|b| *b == 0).unwrap_or(buf.len());
    SHIFT_JIS.decode_without_bom_handling_and_without_replacement(&buf[..str_end])
}

pub fn encode(s: &str) -> Result<Cow<[u8]>, EncodingError> {
    let (result, _, error) = SHIFT_JIS.encode(s);
    if error {
        return Err(EncodingError::InvalidShiftJis);
    }
    Ok(result)
}

/// return false if the given string can't be represented as shift-jis
pub fn encode_into(s: &str, buf: &mut [u8]) -> Result<(), EncodingError> {
    let result = encode(s)?;
    if result.len() > buf.len() {
        return Err(EncodingError::TooLong);
    } else {
        for (b, res) in buf.iter_mut().zip(result.iter().chain(repeat(&0))) {
            *b = *res;
        }
        return Ok(());
    }
}

pub struct NulTermShiftJis {
    pub data: String,
}

impl From<String> for NulTermShiftJis {
    fn from(s: String) -> Self {
        Self { data: s }
    }
}

impl BinRead for NulTermShiftJis {
    type Args = ();

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        opts: &binrw::ReadOptions,
        _: Self::Args,
    ) -> binrw::BinResult<Self> {
        let buf: Vec<u8> = binrw::helpers::until_exclusive(|&b| b == 0)(reader, opts, ())?;
        let (result, _, error) = SHIFT_JIS.decode(&buf);
        if error {
            let pos = reader
                .stream_position()?
                .saturating_sub(buf.len() as u64 + 1);
            return Err(binrw::Error::Custom {
                pos,
                err: Box::new(EncodingError::InvalidShiftJis),
            });
        }
        Ok(result.into_owned().into())
    }
}

pub fn write_nul_term_shift_jis<W: std::io::Write + std::io::Seek>(
    s: &str,
    writer: &mut W
) -> binrw::BinResult<()> {
    let result = encode(s).map_err(|e| {
        binrw::Error::Custom { pos: writer.stream_position().unwrap_or_default(), err: Box::new(format!("invalid for shift jis: {}", s)) }
    })?;
    writer.write_all(&result)?;
    writer.write_all(&[0])?;
    Ok(())
}

impl BinWrite for NulTermShiftJis {
    type Args = ();

    fn write_options<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        _: &binrw::WriteOptions,
        _: Self::Args,
    ) -> binrw::BinResult<()> {
        let result = encode(&self.data).map_err(|e| {
            // TODO: I don't think the position matters here
            binrw::Error::Custom {
                pos: writer.stream_position().unwrap_or_default(),
                err: Box::new(format!("invalid for shift jis: {}", self.data)),
            }
        })?;
        writer.write_all(&result)?;
        writer.write_all(&[0])?;
        Ok(())
    }
}
