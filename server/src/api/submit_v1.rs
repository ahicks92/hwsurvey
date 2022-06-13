use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use bytes::Bytes;
use serde::{Deserialize, Deserializer};
use uuid::Uuid;

use hwsurvey_payloads::PayloadV1;

use crate::writer::WriterThread;

pub async fn submit_v1_fallible(
    writer: &WriterThread,
    token: uuid::Uuid,
    ip: Option<String>,
    country: Option<String>,
    body: Bytes,
) -> Result<()> {
    let payload: PayloadV1 = serde_json::from_slice(&body[..])?;
    let work = crate::writer::WorkItem {
        token,
        ip,
        country,
        payload,
        received_at: chrono::Utc::now(),
    };

    writer.send(work)?;
    Ok(())
}

fn deserialize_uuid<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    let s: String = String::deserialize(deserializer)?;
    Uuid::parse_str(&s).map_err(|_| D::Error::custom("Not a valid uuid"))
}

#[derive(serde::Deserialize)]
pub struct Qparams {
    #[serde(deserialize_with = "deserialize_uuid")]
    token: Uuid,
}

pub async fn submit_v1(
    writer: Arc<WriterThread>,
    qparams: Qparams,
    ip: Option<String>,
    country: Option<String>,
    body: Bytes,
) -> impl warp::reply::Reply {
    let status = match submit_v1_fallible(&*writer, qparams.token, ip, country, body).await {
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
