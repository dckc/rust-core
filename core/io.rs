// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use fail::EINTR;
use os::errno;
use option::{Some, None, Option};
use ops::Drop;
use container::Container;
use c_types::c_int;
use slice::{to_mut_ptr, to_ptr};

enum FILE {}

mod detail {
    use super::FILE;

    extern {
        pub static stdin: *mut FILE;
        pub static stdout: *mut FILE;
        pub static stderr: *mut FILE;
    }
}

extern {
    fn fread(ptr: *mut u8, size: uint, nmemb: uint, stream: *mut FILE) -> uint;
    fn fwrite(ptr: *u8, size: uint, nmemb: uint, stream: *mut FILE) -> uint;
    fn fflush(fp: *mut FILE) -> c_int;

    fn fread_unlocked(ptr: *mut u8, size: uint, nmemb: uint, stream: *mut FILE) -> uint;
    fn fwrite_unlocked(ptr: *u8, size: uint, nmemb: uint, stream: *mut FILE) -> uint;
    fn fflush_unlocked(fp: *mut FILE) -> c_int;

    fn fileno(stream: *mut FILE) -> c_int;

    fn fdatasync(filedes: c_int) -> c_int;
    fn fsync(filedes: c_int) -> c_int;

    fn fopen(path: *u8, mode: *u8) -> *mut FILE;
    fn fclose(fp: *mut FILE) -> c_int;
}

pub struct Error {
    code: c_int
}

pub struct StdStream {
    file: *mut FILE
}

pub fn stdin() -> StdStream {
    StdStream { file: detail::stdin }
}

pub fn stdout() -> StdStream {
    StdStream { file: detail::stdout }
}

pub fn stderr() -> StdStream {
    StdStream { file: detail::stderr }
}

impl StdStream {
    pub fn read(&mut self, xs: &mut [u8]) -> uint {
        unsafe {
            fread(to_mut_ptr(xs), 1, xs.len(), self.file)
        }
    }

    pub fn write(&mut self, xs: &[u8]) -> uint {
        unsafe {
            fwrite(to_ptr(xs), 1, xs.len(), self.file)
        }
    }

    pub fn flush(&mut self) -> Option<Error> {
        if unsafe { fflush(self.file) } != 0 {
            Some(Error { code: errno() })
        } else {
            None
        }
    }
}

pub struct File {
    file: *mut FILE
}

impl File {
    pub unsafe fn open(path: *u8, mode: *u8) -> Option<File> {
        let fp = fopen(path, mode);
        if fp == 0 as *mut FILE {
            None
        } else {
            Some(File { file: fp })
        }
    }

    pub fn read(&mut self, xs: &mut [u8]) -> uint {
        unsafe {
            fread_unlocked(to_mut_ptr(xs), 1, xs.len(), self.file)
        }
    }

    pub fn write(&mut self, xs: &[u8]) -> uint {
        unsafe {
            fwrite_unlocked(to_ptr(xs), 1, xs.len(), self.file)
        }
    }

    pub fn flush(&mut self) -> Option<Error> {
        if unsafe { fflush_unlocked(self.file) } != 0 {
            Some(Error { code: errno() })
        } else {
            None
        }
    }

    pub fn datasync(&mut self) -> Option<Error> {
        loop {
            let ret = unsafe { fdatasync(fileno(self.file)) };
            if ret == -1 {
                let code = errno();
                if code != EINTR {
                    return Some(Error { code: code })
                }
            } else {
                return None
            }
        }
    }

    pub fn sync(&mut self) -> Option<Error> {
        loop {
            let ret = unsafe { fsync(fileno(self.file)) };
            if ret == -1 {
                let code = errno();
                if code != EINTR {
                    return Some(Error { code: code })
                }
            } else {
                return None
            }
        }
    }
}

impl Drop for File {
    fn drop(&mut self) {
        unsafe {
            fclose(self.file);
        }
    }
}
