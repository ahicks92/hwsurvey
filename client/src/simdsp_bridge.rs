//! Parse the output from simdsp_bridge with serde.
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub cpu_manufacturer: String,
    pub cpu_architecture: String,
    pub cache_info: CacheInfo,
    pub cpu_capabilities: CpuCapabilities,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CpuCapabilities {
    pub x86_sse2: bool,
    pub x86_sse3: bool,
    pub x86_ssse3: bool,
    pub x86_sse4_1: bool,
    pub x86_popcnt_insn: bool,
    pub x86_avx: bool,
    pub x86_avx2: bool,
    pub x86_fma3: bool,
    pub x86_fma4: bool,
    pub x86_xop: bool,
    pub x86_avx512f: bool,
    pub x86_avx512bw: bool,
    pub x86_avx512dq: bool,
    pub x86_avx512vl: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheInfo {
    pub l1i: u64,
    pub l1d: u64,
    pub l1u: u64,
    pub l2i: u64,
    pub l2d: u64,
    pub l2u: u64,
    pub l3i: u64,
    pub l3d: u64,
    pub l3u: u64,
}

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
