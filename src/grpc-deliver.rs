#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

// Include automatically generated bindings for "HalonMTA.h"
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

mod config;
mod deliver;
mod utils;

use libc::LOG_CRIT;
use std::sync::OnceLock;
use tonic::transport::Channel;

use config::{config_init_parse, ConfigInit};
use deliver::{
    deliver_get_file, deliver_get_transaction_id, deliver_get_url, deliver_set_response,
};
use rfc822::deliverer_client::DelivererClient;
use rfc822::Rfc822Request;
use utils::{read_file, syslog};

static CONFIG_INIT: OnceLock<ConfigInit> = OnceLock::new();
static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

pub mod rfc822 {
    tonic::include_proto!("rfc822");
}

#[no_mangle]
pub extern "C" fn Halon_version() -> u32 {
    HALONMTA_PLUGIN_VERSION
}

#[no_mangle]
pub extern "C" fn Halon_init(hic: *mut HalonInitContext) -> bool {
    // Get config
    let Ok(cfg) = config_init_parse(hic) else {
        syslog(LOG_CRIT, "grpc-deliver: Failed to parse config");
        return false;
    };

    // Create runtime
    let mut runtime = tokio::runtime::Builder::new_multi_thread();
    runtime.enable_all();
    runtime.thread_name("p/grpc-deliver");
    if cfg.threads != 0 {
        runtime.worker_threads(cfg.threads);
    }

    // Create runtime
    let Ok(rt) = runtime.build() else {
        syslog(LOG_CRIT, "grpc-deliver: Failed to create runtime");
        return false;
    };
    let _ = RUNTIME.set(rt);

    // Store config globally
    let Ok(_) = CONFIG_INIT.set(cfg) else {
        syslog(LOG_CRIT, "grpc-deliver: Failed to store config globally");
        return false;
    };

    return true;
}

#[no_mangle]
pub extern "C" fn Halon_deliver(hdc: *mut HalonDeliverContext) {
    let _hdc = hdc as usize;
    let runtime = RUNTIME.get().expect("grpc-deliver: Failed to get runtime");
    runtime.spawn(async move {
        // Get file
        let Ok(fp) = deliver_get_file(_hdc as *mut HalonDeliverContext) else {
            deliver_set_response(
                _hdc as *mut HalonDeliverContext,
                421,
                "Temporary local error",
            )
            .expect("Failed to set response");
            return;
        };

        // Read file
        let Ok(rfc822) = read_file(fp) else {
            deliver_set_response(
                _hdc as *mut HalonDeliverContext,
                421,
                "Temporary local error",
            )
            .expect("Failed to set response");
            return;
        };

        // Get transaction ID
        let Ok(transaction_id) = deliver_get_transaction_id(_hdc as *mut HalonDeliverContext)
        else {
            deliver_set_response(
                _hdc as *mut HalonDeliverContext,
                421,
                "Temporary local error",
            )
            .expect("Failed to set response");
            return;
        };

        // Get URL
        let Ok(url) = deliver_get_url(_hdc as *mut HalonDeliverContext) else {
            deliver_set_response(
                _hdc as *mut HalonDeliverContext,
                421,
                "Temporary local error",
            )
            .expect("Failed to set response");
            return;
        };

        // Connect to gRPC server
        let mut client: DelivererClient<Channel> = match DelivererClient::connect(url).await {
            Err(err) => {
                deliver_set_response(
                    _hdc as *mut HalonDeliverContext,
                    421,
                    err.to_string().as_str(),
                )
                .expect("Failed to set response");
                return;
            }
            Ok(client) => client,
        };

        // Send gRPC request
        let request = tonic::Request::new(Rfc822Request {
            transactionid: transaction_id,
            rfc822: rfc822,
        });
        match client.deliver(request).await {
            Err(err) => {
                deliver_set_response(_hdc as *mut HalonDeliverContext, 421, err.message())
                    .expect("Failed to set response");
                return;
            }
            Ok(_) => (),
        }

        // Set success response
        deliver_set_response(_hdc as *mut HalonDeliverContext, 250, "OK")
            .expect("Failed to set response");
    });
}
