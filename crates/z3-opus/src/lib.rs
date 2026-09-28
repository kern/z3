//! Minimal safe wrapper around the libopus decoder API used by Zelda 3 OPUZ.

use std::ffi::{c_int, c_uchar, c_void};
use std::ptr::NonNull;

const OPUS_OK: c_int = 0;
const OPUS_RESET_STATE: c_int = 4028;

#[link(name = "opus")]
unsafe extern "C" {
    fn opus_decoder_create(sample_rate: c_int, channels: c_int, error: *mut c_int) -> *mut c_void;
    fn opus_decoder_destroy(decoder: *mut c_void);
    fn opus_decode(
        decoder: *mut c_void,
        data: *const c_uchar,
        len: c_int,
        pcm: *mut i16,
        frame_size: c_int,
        decode_fec: c_int,
    ) -> c_int;
    fn opus_decoder_ctl(decoder: *mut c_void, request: c_int, ...) -> c_int;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Error(pub i32);

#[derive(Debug)]
pub struct Decoder(NonNull<c_void>);

// SAFETY: an OpusDecoder has no thread affinity. `Decoder` exposes only
// `&mut self` operations, so moving it between threads cannot create
// concurrent libopus access; native audio owns it behind the runtime mutex.
unsafe impl Send for Decoder {}

impl Decoder {
    pub fn new_stereo_48khz() -> Result<Self, Error> {
        let mut error = OPUS_OK;
        // SAFETY: libopus accepts these documented constants and writes only `error`.
        let decoder = unsafe { opus_decoder_create(48_000, 2, &mut error) };
        NonNull::new(decoder).map(Self).ok_or(Error(error))
    }

    pub fn reset(&mut self) -> Result<(), Error> {
        // SAFETY: self owns a live OpusDecoder and RESET_STATE has no extra argument.
        let result = unsafe { opus_decoder_ctl(self.0.as_ptr(), OPUS_RESET_STATE) };
        (result == OPUS_OK).then_some(()).ok_or(Error(result))
    }

    pub fn decode(&mut self, packet: &[u8], output: &mut [i16]) -> Result<usize, Error> {
        let frames = output.len() / 2;
        let packet_len = c_int::try_from(packet.len()).map_err(|_| Error(-1))?;
        let frame_size = c_int::try_from(frames).map_err(|_| Error(-1))?;
        // SAFETY: all pointers refer to live slices for the lengths passed; the decoder is owned.
        let result = unsafe {
            opus_decode(
                self.0.as_ptr(),
                packet.as_ptr(),
                packet_len,
                output.as_mut_ptr(),
                frame_size,
                0,
            )
        };
        usize::try_from(result).map_err(|_| Error(result))
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        // SAFETY: the pointer was created by opus_decoder_create and is destroyed once here.
        unsafe { opus_decoder_destroy(self.0.as_ptr()) };
    }
}
