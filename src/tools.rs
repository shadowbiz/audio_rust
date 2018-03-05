#![allow(dead_code)]

use std::fs::File;
use std::io::prelude::*;

use std::os::windows::ffi::OsStrExt;
use std::ffi::OsStr;

pub struct BitScanResult {
    found: bool,
    index: u32,
}

#[inline]
pub fn find_leastsignificant_setbit(value: u32) -> BitScanResult {
    for t in 0..32 {
        if value & (1 << t) != 0 {
            return BitScanResult {
                found: true,
                index: t,
            };
        }
    }
    BitScanResult {
        found: false,
        index: 0,
    }
}

#[inline(always)]
pub fn change_bytes(value: u32, bits: u32, new_value: u32) -> u32 {
    let shift = new_value << (8 * bits);
    let mask = 0xFF << shift;
    return (!mask & value) | shift;
}

#[inline(always)]
pub fn get_index(x: i32, y: i32, width: i32) -> usize {
    (y * width + x) as usize
}

#[inline(always)]
pub fn kilobytes(bytes: u64) -> u64 {
    (bytes * 1024)
}

#[inline(always)]
pub fn megabytes(bytes: u64) -> u64 {
    kilobytes(bytes) * 1024
}

#[inline(always)]
pub fn gigabytes(bytes: u64) -> u64 {
    megabytes(bytes) * 1024
}

#[inline(always)]
pub fn terabytes(bytes: u64) -> u64 {
    gigabytes(bytes) * 1024
}

#[inline(always)]
pub fn to_kilobytes(bytes: u64) -> u64 {
    bytes / 1024
}

#[inline(always)]
pub fn to_megabytes(bytes: u64) -> u64 {
    to_kilobytes(bytes) / 1024
}

#[inline(always)]
pub fn to_gigabytes(bytes: u64) -> u64 {
    to_megabytes(bytes) / 1024
}

#[inline(always)]
pub fn to_terabytes(bytes: u64) -> u64 {
    to_gigabytes(bytes) / 1024
}

#[inline(always)]
pub fn to_wstring(str: &str) -> Vec<u16> {
    OsStr::new(str)
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect()
}

#[cfg(target_arch = "x86")]
#[inline(always)]
#[cold]
pub unsafe fn fast_set32(dst: *mut u32, src: u32, len: usize) {
    asm!("cld
        rep stosd"
        :
        : "{edi}"(dst as usize), "{eax}"(src), "{ecx}"(len)
        : "cc", "memory", "edi", "ecx"
        : "intel", "volatile");
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
#[cold]
pub fn fast_set32(dst: *mut u32, src: u32, len: usize) {
    unsafe {
        asm!("cld
        rep stosd"
        :
        : "{rdi}"(dst as usize), "{eax}"(src), "{rcx}"(len)
        : "cc", "memory", "rdi", "rcx"
        : "intel", "volatile");
    }
}

pub fn read_file() -> Vec<u8> {
    let mut data = Vec::new();
    let mut f = File::open("../../images/logo.bmp").expect("Unable to open file");
    f.read_to_end(&mut data).expect("Unable to read data");
    data
}
