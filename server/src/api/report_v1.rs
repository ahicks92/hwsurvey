use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use bytes::Bytes;

use hwsurvey_payloads::PayloadV1;

use crate::writer::WriterThread;

/// Sentinel value for when we can't extract an IP, which should almost never happen.
pub const UNKNOWN_IP_STRING: &str = "unknown_ip";

/// The unknown country from Cloudflare's docs:
/// https://developers.cloudflare.com/fundamentals/get-started/reference/http-request-headers/
///
/// We fill this in automatically for local development or if the header is missing.
pub const UNKNOWN_COUNTRY: [char; 2] = ['X', 'X'];

pub async fn report_v1_fallible(
    writer: &WriterThread,
    _ip: Option<String>,
    _country: Option<String>,
    body: Bytes,
) -> Result<()> {
    let payload: PayloadV1 = serde_json::from_slice(&body[..])?;

    writer.send(payload)?;
    Ok(())
}

pub async fn report_v1(
    writer: Arc<WriterThread>,
    ip: Option<String>,
    country: Option<String>,
    body: Bytes,
) -> impl warp::reply::Reply {
    let status = match report_v1_fallible(&*writer, ip, country, body).await {
        Ok(_) => warp::http::StatusCode::OK,
        Err(e) => {
            only_every::only_every!(Duration::from_secs(3), {
                log::error!("Could not handle reporting request because {:?}", e);
            });

            // We really don't want to leak anything to users because users are very untrusted and if someone is going
            // to find a way to crash us they will, so just claim nothing happened.
            warp::http::StatusCode::BAD_REQUEST
        }
    };

    warp::reply::with_status(warp::reply::reply(), status)
}
