//! Parse the output from simdsp_bridge with serde.
use anyhow::Result;

use hwsurvey_payloads::simdsp_bridger::SystemInfo;

pub fn get_system_info() -> Result<SystemInfo> {
    extern "C" {
        fn simdspBridgeGetSystemInfoAsJson() -> *mut u8;
        fn simdspBridgeFreeJsonString(data: *mut u8);
    }

    unsafe {
        let data = simdspBridgeGetSystemInfoAsJson();
        let rust_string = std::ffi::CStr::from_ptr(data as *const i8);

        let ret = rust_string
            .to_str()
            .map_err(|e| anyhow::anyhow!(e))
            .and_then(|x| Ok(serde_json::from_str(x)?));

        simdspBridgeFreeJsonString(data);
        ret
    }
}
