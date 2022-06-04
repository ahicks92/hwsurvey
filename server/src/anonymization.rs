const KB: u64 = 1024;
const MB: u64 = KB * 1024;
const GB: u64 = 1024 * MB;

const CACHE_BINS: &[u64] = &[
    0,
    KB,
    2 * KB,
    4 * KB,
    8 * KB,
    16 * KB,
    32 * KB,
    MB,
    4 * MB,
    8 * MB,
    16 * MB,
    32 * MB,
    64 * MB,
    128 * MB,
    256 * MB,
    u64::MAX,
];

const MEM_BINS: &[u64] = &[GB, 2 * GB, 4 * GB, 8 * GB, 16 * GB, u64::MAX];

#[derive(Debug)]
pub struct CpuCapabilitiesRow {
    pub architecture: String,
    pub manufacturer: String,
    pub x86_sse2: bool,
    pub x86_sse3: bool,
    pub x86_ssse3: bool,
    pub x86_sse4_1: bool,
    pub x86_fma3: bool,
    pub x86_avx: bool,
    pub x86_avx2: bool,
    pub x86_avx512f: bool,
}

#[derive(Debug)]
pub struct CpuCachesRow {
    pub architecture: String,
    pub manufacturer: String,

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

#[derive(Debug)]
pub struct OsRow {
    pub os: String,
    pub architecture: String,
}

#[derive(Debug)]
pub struct MemoryRow {
    pub os: &'static str,
    pub manufacturer: &'static str,
    pub total_mem: u64,
}

#[derive(Debug)]
pub struct Rows {
    pub cpu_capabilities: CpuCapabilitiesRow,
    pub cpu_caches: CpuCachesRow,
    pub os: OsRow,
    pub memory: MemoryRow,
}

fn bin(input: u64, bins: &[u64]) -> u64 {
    let mut out = 0;

    for i in bins.iter() {
        if *i > input {
            break;
        }

        out = *i;
    }

    out
}

/// Round the cache to one of our hard-coded bins for anonymization purposes.
pub fn round_cache(cache: u64) -> u64 {
    bin(cache, CACHE_BINS)
}

/// Round the memory to one of our hard-coded bins for anonymization purposes.
pub fn round_mem(mem: u64) -> u64 {
    bin(mem, MEM_BINS)
}
