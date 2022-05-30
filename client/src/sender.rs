use std::time::Duration;

use anyhow::Result;
use rand::prelude::*;
use reqwest::blocking::Client;

const RETRY_DUR: Duration = Duration::from_secs(30);
const JITTER: Duration = Duration::from_secs(20);
const MAX_ATTEMPTS: u64 = 5;

/// This should be large because we're going to be running in one GCP region for all over the world.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

fn check_f64_for_duration(input: f64) -> Result<()> {
    if input < 0.0 || !input.is_finite() {
        anyhow::bail!("Unable to compute retry: {}", input);
    }

    Ok(())
}

fn compute_jitter() -> Result<Duration> {
    let frac = thread_rng().gen_range(0.5f64..=1.5f64);
    let candidate = JITTER.as_secs_f64() * frac;
    check_f64_for_duration(candidate)?;
    Ok(Duration::from_secs_f64(candidate))
}

fn compute_sleep(attempts: u64) -> Result<Duration> {
    let base = RETRY_DUR + compute_jitter()?;
    let candidate = base.as_secs_f64().powi(attempts as i32);
    check_f64_for_duration(candidate)?;
    Ok(Duration::from_secs_f64(candidate))
}

fn attempt_sending(
    client: &Client,
    url: &reqwest::Url,
    payload: &hwsurvey_payloads::Payload,
) -> Result<()> {
    let serialized = serde_json::to_string(payload)?;

    let resp = client
        .post(url.clone())
        .body(serialized)
        .timeout(REQUEST_TIMEOUT)
        .send()?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().unwrap_or_else(|e| {
            log::warn!("Error reading body from server: {:?}", e);
            String::from("Unable to print body")
        });
        let loggable = &body[..body.len().min(1024)];

        log::warn!(
            "Got non-2xx status from server: {} {}",
            status.as_u16(),
            loggable
        );
    }

    Ok(())
}

fn sending_thread_fallible(url: String, appname: String, max_attempts: u64) -> Result<()> {
    let url = reqwest::Url::parse(&url)?;
    let payload = crate::build_payload::build_payload(appname)?;
    let client = Client::new();

    for i in 1..=max_attempts {
        match attempt_sending(&client, &url, &payload) {
            Ok(_) => break,
            Err(e) => {
                log::warn!("Error sending metrics. Retrying. Got: {:?}", e);
                if i == max_attempts {
                    anyhow::bail!("Unable to send: {:?}", e);
                }
                std::thread::sleep(compute_sleep(i)?);
                continue;
            }
        }
    }

    log::info!("Sent metrics");
    Ok(())
}

pub fn send_synchronously(url: String, appname: String, max_attempts: u64) {
    if let Err(e) = sending_thread_fallible(url, appname, max_attempts) {
        log::warn!("Unable to send metrics: {:?}", e);
    }
}

pub fn send_metrics(url: String, appname: String) {
    std::thread::spawn(|| send_synchronously(url, appname, MAX_ATTEMPTS));
}
