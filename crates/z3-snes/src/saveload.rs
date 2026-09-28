//! Safe byte-stream adapter for the C emulator's `SaveLoadFunc` convention.

use core::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SaveLoadError {
    Truncated { offset: usize, needed: usize },
    TrailingBytes(usize),
    WrongBlockSize { expected: usize, actual: usize },
}

impl fmt::Display for SaveLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated { offset, needed } => {
                write!(
                    f,
                    "state stream is truncated at {offset}; need {needed} bytes"
                )
            }
            Self::TrailingBytes(bytes) => write!(f, "state stream has {bytes} trailing bytes"),
            Self::WrongBlockSize { expected, actual } => {
                write!(f, "state block has {actual} bytes; expected {expected}")
            }
        }
    }
}

impl std::error::Error for SaveLoadError {}

pub trait SaveLoad {
    fn transfer(&mut self, bytes: &mut [u8]) -> Result<(), SaveLoadError>;
    fn is_loading(&self) -> bool;
}

#[derive(Debug, Default)]
pub struct SaveState {
    bytes: Vec<u8>,
}

impl SaveState {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

impl SaveLoad for SaveState {
    fn transfer(&mut self, bytes: &mut [u8]) -> Result<(), SaveLoadError> {
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }

    fn is_loading(&self) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct LoadState<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> LoadState<'a> {
    #[must_use]
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    pub fn finish(self) -> Result<(), SaveLoadError> {
        let remaining = self.bytes.len() - self.offset;
        if remaining == 0 {
            Ok(())
        } else {
            Err(SaveLoadError::TrailingBytes(remaining))
        }
    }
}

impl SaveLoad for LoadState<'_> {
    fn transfer(&mut self, bytes: &mut [u8]) -> Result<(), SaveLoadError> {
        let end = self
            .offset
            .checked_add(bytes.len())
            .ok_or(SaveLoadError::Truncated {
                offset: self.offset,
                needed: bytes.len(),
            })?;
        let source = self
            .bytes
            .get(self.offset..end)
            .ok_or(SaveLoadError::Truncated {
                offset: self.offset,
                needed: bytes.len(),
            })?;
        bytes.copy_from_slice(source);
        self.offset = end;
        Ok(())
    }

    fn is_loading(&self) -> bool {
        true
    }
}

pub(crate) fn put_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

pub(crate) fn get_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

pub(crate) fn put_i16(bytes: &mut [u8], offset: usize, value: i16) {
    put_u16(bytes, offset, value as u16);
}

pub(crate) fn get_i16(bytes: &[u8], offset: usize) -> i16 {
    get_u16(bytes, offset) as i16
}

pub(crate) fn put_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

pub(crate) fn get_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("four bytes"))
}

pub(crate) fn bool_byte(value: bool) -> u8 {
    u8::from(value)
}
