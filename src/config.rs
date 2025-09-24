use serde::Deserialize;
use std::{error::Error, ffi::c_char, ffi::c_void, ffi::CStr, ptr::null, ptr::null_mut};

/// The startup configuration for the plugin
#[derive(Deserialize, Debug)]
pub struct ConfigInit {
    #[serde(default)]
    pub threads: usize,
}

/// Parse the startup configuration for the plugin
pub fn config_init_parse(hic: *mut crate::HalonInitContext) -> Result<ConfigInit, Box<dyn Error>> {
    unsafe {
        let mut ok: bool;

        // Get info about the plugin
        let mut halon_config: *mut crate::HalonConfig = null_mut();
        let halon_config_ptr: *mut *mut crate::HalonConfig = &mut halon_config;
        ok = crate::HalonMTA_init_getinfo(
            hic,
            crate::HALONMTA_INIT_CONFIG as i32,
            null(),
            0,
            halon_config_ptr as *mut c_void,
            null_mut(),
        );
        if !ok {
            return Err(Box::from("Failed to get info about the plugin"));
        }

        // Convert init config to JSON
        let mut json_config: *mut c_char = null_mut();
        let json_config_ptr: *mut *mut c_char = &mut json_config;
        ok = crate::HalonMTA_config_to_json(halon_config, json_config_ptr, null_mut());
        if !ok {
            libc::free(json_config as *mut c_void);
            return Err(Box::from("Failed to convert init config to JSON"));
        }
        let json_config_cstr: &CStr = CStr::from_ptr(json_config);
        let json_config_str = String::from_utf8_lossy(json_config_cstr.to_bytes()).to_string();

        // Parse init config from JSON
        match serde_json::from_str(json_config_str.as_str()) {
            Ok(cfg) => {
                libc::free(json_config as *mut c_void);
                return Ok(cfg);
            }
            Err(err) => {
                libc::free(json_config as *mut c_void);
                return Err(Box::from(format!(
                    "Failed to parse init config from JSON: {}",
                    err.to_string()
                )));
            }
        };
    }
}
