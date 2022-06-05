use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use bytes::Bytes;

use hwsurvey_payloads::PayloadV1;

use crate::writer::WriterThread;

pub async fn report_v1_fallible(
    writer: &WriterThread,
    ip: Option<String>,
    country: Option<String>,
    body: Bytes,
) -> Result<()> {
    let payload: PayloadV1 = serde_json::from_slice(&body[..])?;
    let work = crate::writer::WorkItem {
        ip,
        country,
        payload,
    };

    writer.send(work)?;
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
