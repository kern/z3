//! Incremental upload framing and canonical-RAM block commits.
//!
//! Earlier complete blocks survive malformed later framing. A current block
//! is not committed until its whole destination header and payload are present.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UploadError {
    Truncated,
}

pub(super) struct UploadBlocks<'a> {
    data: &'a [u8],
    position: usize,
    finished: bool,
}
pub(super) struct UploadBlock<'a> {
    destination: u16,
    payload: &'a [u8],
}
impl<'a> UploadBlocks<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            position: 0,
            finished: false,
        }
    }
    pub fn next_block(&mut self) -> Result<Option<UploadBlock<'a>>, UploadError> {
        if self.finished {
            return Ok(None);
        }
        let p = self.position;
        if p + 2 > self.data.len() {
            return Err(UploadError::Truncated);
        }
        let length = usize::from(u16::from_le_bytes([self.data[p], self.data[p + 1]]));
        self.position += 2;
        if length == 0 {
            self.finished = true;
            return Ok(None);
        }
        let p = self.position;
        if p + 2 + length > self.data.len() {
            return Err(UploadError::Truncated);
        }
        let destination = u16::from_le_bytes([self.data[p], self.data[p + 1]]);
        let payload = &self.data[p + 2..p + 2 + length];
        self.position = p + 2 + length;
        Ok(Some(UploadBlock {
            destination,
            payload,
        }))
    }
}
impl UploadBlock<'_> {
    pub fn apply(self, ram: &mut [u8; 65536]) {
        let mut address = self.destination;
        for &byte in self.payload {
            ram[usize::from(address)] = byte;
            address = address.wrapping_add(1);
        }
    }
}
