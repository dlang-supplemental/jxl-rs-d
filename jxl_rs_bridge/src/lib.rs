//! C ABI around the `jxl` (jxl-rs) decoder for D / FFI consumers.

use jxl::api::{
    JxlDecoder, JxlDecoderOptions, JxlOutputBuffer, JxlPixelFormat, ProcessingResult,
    states::{Initialized, WithFrameInfo, WithImageInfo},
};
use jxl::headers::extra_channels::ExtraChannel;
use std::cell::RefCell;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;
use std::slice;

thread_local! {
    static LAST_ERROR: RefCell<String> = const { RefCell::new(String::new()) };
    static LAST_ERROR_C: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

fn set_error(msg: impl Into<String>) {
    LAST_ERROR.with(|e| *e.borrow_mut() = msg.into());
}

fn clear_error() {
    LAST_ERROR.with(|e| e.borrow_mut().clear());
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JxlRsStatus {
    Ok = 0,
    NeedMoreInput = 1,
    Error = 2,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JxlRsImageInfo {
    pub width: u32,
    pub height: u32,
    pub has_alpha: u8,
    pub is_animated: u8,
    pub valid: u8,
}

/// One-shot decode of a complete JXL buffer to tightly packed RGBA8.
///
/// On success, writes width/height and returns a heap buffer of `width*height*4`
/// bytes that the caller must free with [`jxl_rs_free`]. Returns null on error;
/// call [`jxl_rs_last_error`] for details.
///
/// # Safety
/// - `data` must be valid for `len` bytes (or null with `len == 0`).
/// - `out_width` / `out_height` must be valid writable `u32` pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jxl_rs_decode_rgba8(
    data: *const u8,
    len: usize,
    out_width: *mut u32,
    out_height: *mut u32,
) -> *mut u8 {
    clear_error();
    if out_width.is_null() || out_height.is_null() {
        set_error("out_width/out_height must be non-null");
        return ptr::null_mut();
    }
    if data.is_null() && len != 0 {
        set_error("data is null with non-zero length");
        return ptr::null_mut();
    }

    let result = catch_unwind(AssertUnwindSafe(|| {
        let input = if len == 0 {
            &[][..]
        } else {
            // SAFETY: caller guarantees `data` is valid for `len` bytes.
            unsafe { slice::from_raw_parts(data, len) }
        };
        decode_rgba8_owned(input)
    }));

    match result {
        Ok(Ok((w, h, pixels))) => {
            // SAFETY: out pointers checked non-null above.
            unsafe {
                *out_width = w;
                *out_height = h;
            }
            let mut boxed = pixels.into_boxed_slice();
            let ptr = boxed.as_mut_ptr();
            std::mem::forget(boxed);
            ptr
        }
        Ok(Err(e)) => {
            set_error(e);
            ptr::null_mut()
        }
        Err(_) => {
            set_error("panic while decoding");
            ptr::null_mut()
        }
    }
}

/// Frees a buffer returned by [`jxl_rs_decode_rgba8`].
///
/// # Safety
/// `ptr` must be null or a pointer previously returned by [`jxl_rs_decode_rgba8`]
/// with matching `len` (`width * height * 4`).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jxl_rs_free(ptr: *mut u8, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }
    // SAFETY: paired with into_boxed_slice + forget in decode.
    unsafe {
        let _ = Box::from_raw(slice::from_raw_parts_mut(ptr, len));
    }
}

/// Pointer to a thread-local NUL-terminated last error string (empty if none).
/// Valid until the next bridge call on this thread.
#[unsafe(no_mangle)]
pub extern "C" fn jxl_rs_last_error() -> *const u8 {
    LAST_ERROR.with(|e| {
        let s = e.borrow();
        LAST_ERROR_C.with(|buf| {
            let mut b = buf.borrow_mut();
            b.clear();
            b.extend_from_slice(s.as_bytes());
            b.push(0);
            b.as_ptr()
        })
    })
}

/// Probe basic info without allocating pixel storage.
///
/// # Safety
/// Same as [`jxl_rs_decode_rgba8`] for `data`/`len`; `out` must be non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jxl_rs_probe(
    data: *const u8,
    len: usize,
    out: *mut JxlRsImageInfo,
) -> JxlRsStatus {
    clear_error();
    if out.is_null() {
        set_error("out must be non-null");
        return JxlRsStatus::Error;
    }
    if data.is_null() && len != 0 {
        set_error("data is null with non-zero length");
        return JxlRsStatus::Error;
    }

    let result = catch_unwind(AssertUnwindSafe(|| {
        let input = if len == 0 {
            &[][..]
        } else {
            unsafe { slice::from_raw_parts(data, len) }
        };
        probe_info(input)
    }));

    match result {
        Ok(Ok(info)) => {
            unsafe { *out = info };
            JxlRsStatus::Ok
        }
        Ok(Err(e)) => {
            set_error(e);
            unsafe {
                *out = JxlRsImageInfo::default();
            }
            JxlRsStatus::Error
        }
        Err(_) => {
            set_error("panic while probing");
            unsafe {
                *out = JxlRsImageInfo::default();
            }
            JxlRsStatus::Error
        }
    }
}

fn probe_info(mut input: &[u8]) -> Result<JxlRsImageInfo, String> {
    let decoder = JxlDecoder::<Initialized>::new(JxlDecoderOptions::default());
    let with_info = match decoder
        .process(&mut input)
        .map_err(|e| e.to_string())?
    {
        ProcessingResult::Complete { result } => result,
        ProcessingResult::NeedsMoreInput { .. } => {
            return Err("truncated JXL (need more input for headers)".into());
        }
    };
    let basic = with_info.basic_info();
    Ok(JxlRsImageInfo {
        width: basic.size.0 as u32,
        height: basic.size.1 as u32,
        has_alpha: u8::from(
            basic
                .extra_channels
                .iter()
                .any(|c| matches!(c.ec_type, ExtraChannel::Alpha)),
        ),
        is_animated: u8::from(basic.animation.is_some()),
        valid: 1,
    })
}

fn decode_rgba8_owned(mut input: &[u8]) -> Result<(u32, u32, Vec<u8>), String> {
    let decoder = JxlDecoder::<Initialized>::new(JxlDecoderOptions::default());
    let mut with_info: JxlDecoder<WithImageInfo> = match decoder
        .process(&mut input)
        .map_err(|e| e.to_string())?
    {
        ProcessingResult::Complete { result } => result,
        ProcessingResult::NeedsMoreInput { .. } => {
            return Err("truncated JXL (need more input for headers)".into());
        }
    };

    let basic = with_info.basic_info().clone();
    let (w, h) = basic.size;
    if w == 0 || h == 0 {
        return Err("invalid image size".into());
    }
    let num_extra = basic.extra_channels.len();
    with_info.set_pixel_format(JxlPixelFormat::rgba8(num_extra));

    let with_frame: JxlDecoder<WithFrameInfo> = match with_info
        .process(&mut input)
        .map_err(|e| e.to_string())?
    {
        ProcessingResult::Complete { result } => result,
        ProcessingResult::NeedsMoreInput { .. } => {
            return Err("truncated JXL (need more input for frame header)".into());
        }
    };

    let nbytes = w
        .checked_mul(h)
        .and_then(|n| n.checked_mul(4))
        .ok_or_else(|| "image dimensions overflow".to_string())?;
    let mut pixels = vec![0u8; nbytes];
    let bytes_per_row = w * 4;

    let _with_info_again: JxlDecoder<WithImageInfo> = {
        let mut buffers = [JxlOutputBuffer::new(&mut pixels, h, bytes_per_row)];
        match with_frame
            .process(&mut input, &mut buffers)
            .map_err(|e| e.to_string())?
        {
            ProcessingResult::Complete { result } => result,
            ProcessingResult::NeedsMoreInput { .. } => {
                return Err("truncated JXL (need more input for pixels)".into());
            }
        }
    };

    Ok((w as u32, h as u32, pixels))
}
