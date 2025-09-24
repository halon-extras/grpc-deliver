use std::error::Error;
use std::{ffi::c_char, ffi::CStr, ffi::CString, ptr::null_mut};

use crate::{
    HalonDeliverContext, HalonHSLValue, HalonMTA_deliver_done, HalonMTA_deliver_getinfo,
    HalonMTA_deliver_setinfo, HalonMTA_hsl_value_array_find, HalonMTA_hsl_value_get,
    HalonMTA_message_getinfo, HalonQueueMessage, HALONMTA_HSL_TYPE_STRING, HALONMTA_INFO_ARGUMENTS,
    HALONMTA_INFO_FILE, HALONMTA_INFO_MESSAGE, HALONMTA_MESSAGE_TRANSACTIONID,
    HALONMTA_RESULT_CODE, HALONMTA_RESULT_REASON,
};

/// Get the arguments for the delivery attempt
pub fn deliver_get_arguments(
    hdc: *mut HalonDeliverContext,
) -> Result<*mut HalonHSLValue, Box<dyn Error>> {
    unsafe {
        let mut args: *mut HalonHSLValue = std::ptr::null_mut();
        let ok = HalonMTA_deliver_getinfo(
            hdc,
            HALONMTA_INFO_ARGUMENTS as i32,
            std::ptr::null(),
            0,
            &mut args as *mut _ as *mut libc::c_void,
            std::ptr::null_mut(),
        );
        if !ok {
            return Err(Box::from("Failed to get arguments"));
        }
        Ok(args)
    }
}

/// Get information about the message
pub fn deliver_get_message(
    hdc: *mut HalonDeliverContext,
) -> Result<*mut HalonQueueMessage, Box<dyn Error>> {
    unsafe {
        let mut hqm: *mut HalonQueueMessage = std::ptr::null_mut();
        let ok = HalonMTA_deliver_getinfo(
            hdc,
            HALONMTA_INFO_MESSAGE as i32,
            std::ptr::null(),
            0,
            &mut hqm as *mut _ as *mut libc::c_void,
            std::ptr::null_mut(),
        );
        if !ok {
            return Err(Box::from("Failed to get info about the message"));
        }
        Ok(hqm)
    }
}

/// Get the pointer to the mail file
pub fn deliver_get_file(hdc: *mut HalonDeliverContext) -> Result<*mut libc::FILE, Box<dyn Error>> {
    unsafe {
        let mut fp: *mut libc::FILE = std::ptr::null_mut();
        let ok: bool = HalonMTA_deliver_getinfo(
            hdc,
            HALONMTA_INFO_FILE as i32,
            std::ptr::null(),
            0,
            &mut fp as *mut _ as *mut libc::c_void,
            std::ptr::null_mut(),
        );
        if !ok {
            return Err(Box::from("Failed to get pointer to mail file"));
        }
        Ok(fp)
    }
}

/// Get the transaction ID
pub fn deliver_get_transaction_id(hdc: *mut HalonDeliverContext) -> Result<String, Box<dyn Error>> {
    unsafe {
        let hqm = deliver_get_message(hdc)?;
        let mut char: *mut c_char = null_mut();
        let ok = HalonMTA_message_getinfo(
            hqm,
            HALONMTA_MESSAGE_TRANSACTIONID as i32,
            std::ptr::null(),
            0,
            &mut char as *mut _ as *mut libc::c_void,
            std::ptr::null_mut(),
        );
        if !ok {
            return Err(Box::from("Failed to get transaction ID"));
        }
        let transaction_id_cstr: &CStr = CStr::from_ptr(char);
        let transaction_id = String::from_utf8_lossy(transaction_id_cstr.to_bytes()).to_string();
        Ok(transaction_id)
    }
}

/// Get the URL
pub fn deliver_get_url(hdc: *mut HalonDeliverContext) -> Result<String, Box<dyn Error>> {
    unsafe {
        let arguments = deliver_get_arguments(hdc)?;
        if !arguments.is_null() {
            let key = CString::new("url")?;
            let value = HalonMTA_hsl_value_array_find(arguments, key.as_ptr());
            if !value.is_null() {
                let mut char: *const c_char = std::ptr::null();
                let ok = HalonMTA_hsl_value_get(
                    value,
                    HALONMTA_HSL_TYPE_STRING as i32,
                    &mut char as *mut _ as *mut libc::c_void,
                    std::ptr::null_mut(),
                );
                if !ok {
                    return Err(Box::from("Failed to get URL"));
                }
                let url_cstr: &CStr = CStr::from_ptr(char);
                let url = String::from_utf8_lossy(url_cstr.to_bytes()).to_string();
                return Ok(url);
            }
        }
        return Err(Box::from("Missing or invalid arguments"));
    }
}

/// Set delivery response
pub fn deliver_set_response(
    hdc: *mut HalonDeliverContext,
    status: i32,
    reason: &str,
) -> Result<(), Box<dyn Error>> {
    unsafe {
        let mut ok: bool;
        ok = HalonMTA_deliver_setinfo(
            hdc,
            HALONMTA_RESULT_CODE as i32,
            &status as *const _ as *const libc::c_void,
            std::mem::size_of_val(&status),
        );
        if !ok {
            return Err(Box::from("Failed to set result code"));
        }
        let Ok(_reason) = std::ffi::CString::new(reason) else {
            return Err(Box::from("Failed to create C string"));
        };
        ok = HalonMTA_deliver_setinfo(
            hdc,
            HALONMTA_RESULT_REASON as i32,
            _reason.as_ptr() as *const libc::c_void,
            0,
        );
        if !ok {
            return Err(Box::from("Failed to set result reason"));
        }
        HalonMTA_deliver_done(hdc);
        Ok(())
    }
}
