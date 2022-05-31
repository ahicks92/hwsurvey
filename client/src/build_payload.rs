use anyhow::Result;
use sysinfo::{System, SystemExt};

use hwsurvey_payloads::{memory::Memory, Payload, PayloadV1};

pub fn build_payload(application_name: String) -> Result<Payload> {
    let simdsp = hwsurvey_simdsp_bridger::get_system_info()?;

    let sysinfo = System::new_with_specifics(sysinfo::RefreshKind::new().with_memory().with_cpu());
    let memory = Memory {
        // We want bytes, sysinfo gives us kb.
        total: sysinfo.total_memory() * 1024,
    };

    let mac_address_raw = mac_address::get_mac_address()?
        .ok_or_else(|| anyhow::anyhow!("Unable to get a MAC address"))?;

    let mac_address = hex::encode(mac_address_raw.bytes());
    let os = std::env::consts::OS.to_string();

    Ok(Payload::V1(PayloadV1 {
        simdsp,
        memory,
        os,
        mac_address,
        application_name,
    }))
}

#[test]
fn test_payload_building() {
    build_payload("test_app".to_string()).expect("Should be able to build the payload");
}
