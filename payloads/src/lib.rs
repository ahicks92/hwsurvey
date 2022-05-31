pub mod memory;
pub mod simdsp_bridger;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PayloadV1 {
    pub simdsp: simdsp_bridger::SystemInfo,
    pub memory: memory::Memory,
    pub os: String,
    pub application_name: String,
    pub mac_address: String,
}
