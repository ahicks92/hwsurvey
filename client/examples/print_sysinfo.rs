use anyhow::Result;

fn main() -> Result<()> {
    let info = hwsurvey_client::simdsp_bridge::get_system_info()?;
    println!("{}", serde_json::to_string_pretty(&info)?);
    Ok(())
}
