use std::error::Error;
use std::ffi::CString;
use std::io;

/// A wrapper for the "syslog" C function
pub fn syslog(pri: libc::c_int, str: &str) {
    match CString::new("%s") {
        Ok(format_cstr) => match CString::new(str) {
            Ok(cstr) => {
                unsafe { libc::syslog(pri, format_cstr.as_ptr(), cstr.as_ptr()) };
            }
            Err(_) => (),
        },
        Err(_) => (),
    }
}

/// Read contents of a "FILE" C pointer
pub fn read_file(fp: *mut libc::FILE) -> Result<Vec<u8>, Box<dyn Error>> {
    unsafe {
        if libc::fseek(fp, 0, libc::SEEK_SET) != 0 {
            return Err(Box::from(io::Error::last_os_error()));
        }
        let mut out = Vec::new();
        let mut buf = [0u8; 8192];
        loop {
            let n = libc::fread(buf.as_mut_ptr() as *mut libc::c_void, 1, buf.len(), fp) as usize;
            if n == 0 {
                if libc::ferror(fp) != 0 {
                    return Err(Box::from(io::Error::last_os_error()));
                }
                break;
            }
            out.extend_from_slice(&buf[..n]);
        }
        Ok(out)
    }
}
