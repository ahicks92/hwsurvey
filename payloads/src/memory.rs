use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Memory {
    pub total: u64,
}
