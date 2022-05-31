use anyhow::Result;
use sysinfo::{System, SystemExt};

use hwsurvey_payloads::{memory::Memory, PayloadV1};

const SALT: &str = "98badb58-e077-11ec-8edf-00d8612ce6ed";

/// Apply sha512 to a string combined with a fixed, hard-coded salt.  Then apply sha512 again.
///
/// This is what bitcoin does, so secure enough for our purposes given that the inputs are already pretty anonymous.
fn double_hash(input: &str) -> String {
    use sha2::{Digest, Sha512};

    let level2 = {
        let mut hasher = Sha512::new();
        hasher.update(SALT.as_bytes());
        hasher.update(&[0]);
        hasher.update(input);
        let res = hasher.finalize();
        hex::encode(&res[..])
    };

    let mut hasher = Sha512::new();
    hasher.update(level2.as_bytes());
    let res = hasher.finalize();
    hex::encode(&res[..])
}
pub fn build_payload(application_name: String) -> Result<PayloadV1> {
    let simdsp = hwsurvey_simdsp_bridger::get_system_info()?;

    let sysinfo = System::new_with_specifics(sysinfo::RefreshKind::new().with_memory().with_cpu());
    let memory = Memory {
        // We want bytes, sysinfo gives us kb.
        total: sysinfo.total_memory() * 1024,
    };

    let machine_id = {
        let mac_address_raw = mac_address::get_mac_address()?
            .ok_or_else(|| anyhow::anyhow!("Unable to get a MAC address"))?;

        let mac_address = hex::encode(mac_address_raw.bytes());

        let maybe_hostname = sysinfo.host_name();
        let hostname = match maybe_hostname.as_ref() {
            Some(x) => x.as_str(),
            None => {
                log::warn!("Unable to get hostname. Using ahrd-coded default");
                "unknown"
            }
        };

        double_hash(&format!("{}\n{}", mac_address, hostname))
    };
    let os = std::env::consts::OS.to_string();

    Ok(PayloadV1 {
        simdsp,
        memory,
        os,
        machine_id,
        application_name,
    })
}

#[test]
fn test_payload_building() {
    build_payload("test_app".to_string()).expect("Should be able to build the payload");
}
