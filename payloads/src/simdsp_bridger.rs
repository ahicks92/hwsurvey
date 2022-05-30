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
