// Copy from https://github.com/daedalus2022/ctp-sys/blob/5d5590c48bc319ca3575ec089aa73ed2c774004e/src/ffi_utils.rs without modification
use encoding::all::GB18030;
use encoding::{DecoderTrap, Encoding};
use simple_error::SimpleError;
use std::borrow::Cow;

pub fn ascii_cstr_to_str_i8(v: &[i8]) -> Result<&str, SimpleError> {
    let mut s = unsafe { std::slice::from_raw_parts(v.as_ptr() as *mut u8, v.len()) };
    ascii_cstr_to_str(&mut s)
}

pub fn ascii_cstr_to_str(s: &[u8]) -> Result<&str, SimpleError> {
    match s.last() {
        Some(&0u8) => {
            let len = memchr::memchr(0, s).unwrap();
            let ascii_s = &s[0..len];
            if ascii_s.is_ascii() {
                unsafe { Ok(std::str::from_utf8_unchecked(ascii_s)) }
            } else {
                Err(SimpleError::new("cstr is not ascii"))
            }
        }
        Some(&c) => Err(SimpleError::new(format!(
            "cstr should terminate with null instead of {:#x}",
            c
        ))),
        None => Err(SimpleError::new("cstr cannot have 0 length")),
    }
}

/// 省略一些ascii检查, 在能确保source是正常ascii的情况下使用
pub fn get_ascii_str(source: &[i8]) -> Result<&str, SimpleError> {
    let source = unsafe { std::slice::from_raw_parts(source.as_ptr() as *mut u8, source.len()) };
    for i in 0..source.len() {
        if source[i] == 0 {
            return unsafe { Ok(std::str::from_utf8_unchecked(&source[0..i])) };
        }
    }
    Err(SimpleError::new("ascii str should terminate with null"))
}

pub fn trading_day_from_ctp_trading_day(i: &[i8]) -> i32 {
    let d = ascii_cstr_to_str_i8(i);
    if d.is_err() {
        return 0;
    }
    let d = d.unwrap().trim();
    if d.len() == 0 {
        return 0;
    }
    let o = d.parse();
    match o {
        Ok(v) => v,
        Err(e) => panic!("{} {}", e, d),
    }
}

pub fn set_cstr_from_str_truncate_i8(buffer: &mut [i8], text: &str) {
    let v = unsafe { std::slice::from_raw_parts_mut(buffer.as_ptr() as *mut u8, buffer.len()) };
    set_cstr_from_str_truncate(v, text)
}

pub fn set_cstr_from_str_truncate(buffer: &mut [u8], text: &str) {
    for (place, data) in buffer
        .split_last_mut()
        .expect("buffer len 0 in set_cstr_from_str_truncate")
        .1
        .iter_mut()
        .zip(text.as_bytes().iter())
    {
        *place = *data;
    }
    unsafe {
        *buffer.get_unchecked_mut(text.len()) = 0u8;
    }
}

/// 创建目录
pub fn check_make_dir(dir: &str) {
    match std::fs::create_dir_all(dir) {
        Ok(_) => (),
        Err(e) => {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
            } else {
                panic!("创建dir={}目录失败 {}", dir, e);
            }
        }
    }
}

#[macro_export]
macro_rules! print_rsp_info {
    ($p:expr) => {
        if let Some(p) = $p {
            info!(
                "ErrorID={} Msg={}",
                p.ErrorID,
                ctp_sys::gb18030_cstr_to_str_i8(&p.ErrorMsg).to_string()
            );
        }
    };
}

pub fn gb18030_cstr_to_str_i8(v: &[i8]) -> Cow<str> {
    let v = unsafe { std::slice::from_raw_parts(v.as_ptr() as *mut u8, v.len()) };
    gb18030_cstr_to_str(v)
}

pub fn gb18030_cstr_to_str(v: &[u8]) -> Cow<str> {
    let slice = v.split(|&c| c == 0u8).next().unwrap();
    if slice.is_ascii() {
        unsafe {
            return Cow::Borrowed::<str>(std::str::from_utf8_unchecked(slice));
        }
    }
    match GB18030.decode(slice, DecoderTrap::Replace) {
        Ok(s) => Cow::Owned(s),
        Err(e) => e,
    }
}
