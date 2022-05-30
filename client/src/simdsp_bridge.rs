//! Parse the output from simdsp_bridge with serde.
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub cpu_manufacturer: String,
    pub cache_info: CacheInfo,
    pub cpu_capabilities: CpuCapabilities,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CpuCapabilities {
    pub sse2: bool,
    pub sse3: bool,
    pub ssse3: bool,
    pub sse4_1: bool,
    pub popcnt_insn: bool,
    pub avx: bool,
    pub avx2: bool,
    pub fma3: bool,
    pub fma4: bool,
    pub xop: bool,
    pub avx512f: bool,
    pub avx512bw: bool,
    pub avx512dq: bool,
    pub avx512vl: bool,
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
