use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use bytes::Bytes;

use hwsurvey_payloads::Payload;

use crate::writer::WriterThread;

pub async fn report_v1_fallible(writer: &WriterThread, body: Bytes) -> Result<()> {
    let payload: Payload = serde_json::from_slice(&body[..])?;

    let rows = crate::rows::payload_to_rows(&payload);

    writer.send(rows)?;
    Ok(())
}

pub async fn report_v1(writer: Arc<WriterThread>, body: Bytes) -> impl warp::reply::Reply {
    let status = match report_v1_fallible(&*writer, body).await {
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
