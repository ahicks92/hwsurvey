use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_channel::{bounded, Receiver, Sender};
use tokio::select;

use hwsurvey_payloads::PayloadV1;

/// How many items should we try to batch for?
const BATCH_SIZE: usize = 100;

/// How many items should we allow to be pending before we start erroring?
const MAX_OUTSTANDING_ITEMS: usize = 1000;

/// After this time, flush the batch even if the batch doesn't have much data in it.
const FLUSH_INTERVAL: Duration = Duration::from_secs(5);

pub struct WriterThread {
    receiver: Receiver<PayloadV1>,
    sender: Sender<PayloadV1>,
}

impl WriterThread {
    pub fn send(&self, item: PayloadV1) -> Result<()> {
        self.sender.try_send(item)?;
        Ok(())
    }
}

/// Flush a batch.  Handles failures by logging.
async fn flush_batch(batch: &mut Vec<PayloadV1>) {
    if batch.is_empty() {
        return;
    }

    log::info!("Would write: {:?}", batch);
    batch.clear();
}

async fn writer_task_fallible(writer: Arc<WriterThread>) -> Result<()> {
    let mut batch: Vec<PayloadV1> = vec![];
    let mut flush_tick = tokio::time::interval(FLUSH_INTERVAL);

    loop {
        select! {
            Ok(r) = writer.receiver.recv() => {
                batch.push(r);
                if batch.len() > BATCH_SIZE {
                    flush_batch(&mut batch).await;
                }
            },
            _ = flush_tick.tick() => {
                flush_batch(&mut batch).await;
            }
        }
    }
}

async fn writer_task(writer: Arc<WriterThread>) {
    log::info!("Writer running");

    writer_task_fallible(writer)
        .await
        .expect("The writer crashed");
}

pub fn spawn() -> Arc<WriterThread> {
    let (sender, receiver) = bounded(MAX_OUTSTANDING_ITEMS);
    let thread = Arc::new(WriterThread { sender, receiver });

    let thread_cloned = thread.clone();

    tokio::spawn(writer_task(thread_cloned));

    thread
}
