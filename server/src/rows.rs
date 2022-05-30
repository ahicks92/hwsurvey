//! Get from a payload to a set of normalized rows, suitable for the database.
use hwsurvey_payloads::{Payload, PayloadV1};
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

/// This list is from simdsp. See system_info.cpp.
const VALID_ARCHES: &[&str] = &["aarch64", "x86"];

/// Also from simdsp.
const VALID_MANUFACTURERS: &[&str] = &["intel", "apple"];

/// This list is from https://doc.rust-lang.org/std/env/consts/constant.OS.html
///
/// We only do the ones we care about for now.
const VALID_OSES: &[&str] = &["linux", "macos", "windows", "freebsd", "openbsd"];

#[derive(Debug)]
pub struct CpuCapabilitiesRow {
    pub architecture: &'static str,
    pub manufacturer: &'static str,
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
    pub architecture: &'static str,
    pub manufacturer: &'static str,

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
    pub os: &'static str,
    pub architecture: &'static str,
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

fn round_cache(cache: u64) -> u64 {
    bin(cache, CACHE_BINS)
}

fn round_mem(mem: u64) -> u64 {
    bin(mem, MEM_BINS)
}

fn normalize_string(input: &str, list: &[&'static str]) -> &'static str {
    for i in list.iter() {
        if *i == input {
            return *i;
        }
    }

    "unknown"
}

fn normalize_manufacturer(input: &str) -> &'static str {
    normalize_string(input, VALID_MANUFACTURERS)
}

fn normalize_architecture(arch: &str) -> &'static str {
    normalize_string(arch, VALID_ARCHES)
}

fn normalize_os(os: &str) -> &'static str {
    normalize_string(os, VALID_OSES)
}

fn identity<T>(input: T) -> T {
    input
}

macro_rules! fields {
    ($payload: expr, $from: expr, $out: ident, $norm: ident, $($id: ident),*) => {
        $out {
            manufacturer: normalize_manufacturer($payload.simdsp.cpu_manufacturer.as_str()),
            architecture: normalize_architecture($payload.simdsp.cpu_architecture.as_str()),
            $($id: $norm($from.$id)),*
        }
    }
}

fn extract_cpu_caps(payload: &PayloadV1) -> CpuCapabilitiesRow {
    fields!(
        payload,
        payload.simdsp.cpu_capabilities,
        CpuCapabilitiesRow,
        identity,
        x86_sse2,
        x86_sse3,
        x86_ssse3,
        x86_sse4_1,
        x86_avx,
        x86_avx2,
        x86_avx512f,
        x86_fma3
    )
}

fn extract_caches_row(payload: &PayloadV1) -> CpuCachesRow {
    fields!(
        payload,
        payload.simdsp.cache_info,
        CpuCachesRow,
        round_cache,
        l1i,
        l1d,
        l1u,
        l2i,
        l2d,
        l2u,
        l3i,
        l3d,
        l3u
    )
}

fn extract_os_row(payload: &PayloadV1) -> OsRow {
    OsRow {
        os: normalize_os(&payload.os),
        architecture: normalize_architecture(&payload.simdsp.cpu_architecture),
    }
}

fn extract_mem_row(payload: &PayloadV1) -> MemoryRow {
    MemoryRow {
        manufacturer: normalize_manufacturer(&payload.simdsp.cpu_manufacturer),
        os: normalize_os(&payload.os),
        total_mem: round_mem(payload.memory.total),
    }
}

pub fn payload_to_rows(payload: &Payload) -> Rows {
    match payload {
        Payload::V1(p) => Rows {
            cpu_capabilities: extract_cpu_caps(p),
            cpu_caches: extract_caches_row(p),
            os: extract_os_row(p),
            memory: extract_mem_row(p),
        },
    }
}
