use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use async_channel::{bounded, Receiver, Sender};
use chrono::{DateTime, Utc};
use tokio::select;
use tokio_postgres::{Client, Statement};
use uuid::Uuid;

use hwsurvey_payloads::PayloadV1;
const CPU_CAPABILITIES_TABLE: &str = "cpu_capabilities";
const CPU_CACHES_TABLE: &str = "cpu_caches";
const MEMORY_TABLE: &str = "memory";
const CF_COUNTRY_TABLE: &str = "cf_country";

/// String to use for unknown IPs.
const UNKNOWN_IP: &str = "123.123.123.123";

/// Unknown country code.
///
/// matches Cloudflare's value from:
/// https://developers.cloudflare.com/fundamentals/get-started/reference/http-request-headers/
const UNKNOWN_COUNTRY: &str = "XX";

/// How many items should we try to batch for?
const BATCH_SIZE: usize = 100;

/// How many items should we allow to be pending before we start erroring?
const MAX_OUTSTANDING_ITEMS: usize = 1000;

/// After this time, flush the batch even if the batch doesn't have much data in it.
const FLUSH_INTERVAL: Duration = Duration::from_secs(5);

/// A cache of statements.
type StatementCache = Mutex<HashMap<&'static str, Arc<Statement>>>;

#[derive(Debug)]
pub struct WorkItem {
    pub country: Option<String>,
    pub ip: Option<String>,
    pub payload: PayloadV1,
}

/// Groups a bunch of parameters all our writing functions need.
struct Context {
    day: DateTime<Utc>,
    application: uuid::Uuid,
    os: uuid::Uuid,
    architecture: uuid::Uuid,
    cpu_manufacturer: Uuid,
}

/// Contains caches mapping various strings to the uuids they go with from the db.  Loaded at startup.
#[derive(Debug)]
struct UuidCache {
    application: HashMap<String, uuid::Uuid>,
    os: HashMap<String, uuid::Uuid>,
    cpu_manufacturer: HashMap<String, Uuid>,
    architecture: HashMap<String, Uuid>,
}

impl UuidCache {
    async fn load(client: &Client) -> Result<UuidCache> {
        let mut os = Default::default();
        let mut cpu_manufacturer = Default::default();
        let mut architecture = Default::default();
        let mut application = Default::default();

        let tables: &mut [(&str, &mut HashMap<String, Uuid>)] = &mut [
            ("os", &mut os),
            ("cpu_manufacturer", &mut cpu_manufacturer),
            ("cpu_architecture", &mut architecture),
            ("application", &mut application),
        ];

        for (name, dest) in tables.iter_mut() {
            let query = format!("SELECT id, name FROM {}", name);
            let rows = client.query(&query, &[]).await?;
            for r in rows {
                dest.insert(r.get("name"), r.get("id"));
            }
        }

        Ok(UuidCache {
            os,
            cpu_manufacturer,
            architecture,
            application,
        })
    }

    /// Get the OS, returning the uuid for the unknown value if this OS is unknown to us.
    fn get_os(&self, os: &str) -> Uuid {
        self.os
            .get(os)
            .unwrap_or_else(|| self.os.get("unknown").expect("unknown is always present"))
            .clone()
    }

    /// Get the CPU manufacturer, returning the uuid for the unknown value if it is unknown to us.
    fn get_cpu_manufacturer(&self, manufacturer: &str) -> Uuid {
        self.cpu_manufacturer
            .get(manufacturer)
            .unwrap_or_else(|| self.os.get("unknown").expect("unknown is always present"))
            .clone()
    }

    fn get_architecture(&self, arch: &str) -> Uuid {
        self.architecture
            .get(arch)
            .unwrap_or_else(|| self.os.get("unknown").expect("unknown is always present"))
            .clone()
    }

    /// Get the application.
    ///
    /// Returns None if the application isn't present, since we're not willing to collect for unknown applications.
    fn get_application(&self, app: &str) -> Option<Uuid> {
        self.application
            .get(app)
            .or_else(|| self.os.get("unknown"))
            .cloned()
    }
}

pub struct WriterThread {
    receiver: Receiver<WorkItem>,
    sender: Sender<WorkItem>,
}

impl WriterThread {
    pub fn send(&self, item: WorkItem) -> Result<()> {
        self.sender.try_send(item)?;
        Ok(())
    }
}

/// Build a query to insert into the hlls for one of our metrics tables.
///
/// We could do these as compile-time constants but that's incredibly error-prone.  Instead, we will assume that the
/// first query of each kind is like all the others, ensure this by wrapping the queries behind functions, and cache the
/// prepared strings.
///
/// If we need to optimize later, we can try to reliably pull these out into constants, but the sheer number of
/// off-by-one errors that are possible there and our inability to tell that we made one other than having subtley wrong
/// reporting makes that less than appealing.
fn build_query_string(table: &str, factors: &[&str]) -> String {
    let all_cols = factors
        .iter()
        .copied()
        .chain((&["user_id", "user_ip"]).iter().copied());

    let all_cols = itertools::join(all_cols, ",");

    let factor_params = itertools::join((1..=factors.len()).map(|x| format!("${}", x)), ",");

    let user_id_param = format!("${}", factors.len() + 1);
    let user_ip_param = format!("${}", factors.len() + 2);

    format!(
        r#"
INSERT INTO {table}({all_cols}) VALUES(
({factor_params}, hll_empty() || hll_hash_text({user_id_param}), hll_empty() || hll_hash_text({user_ip_param}))
ON CONFLICT UPDATE SET
(user_id, user_ip) = (
    user_id || hll_hash_text({user_id_param}),
    user_ip || hll_hash_text({user_ip_param})
)"#,
    )
}

/// Run an upsert query against a client.
///
/// This function assumes that the same table always gets the same query with the same factors in the same order, and
/// exists to prevent off-by-ones and the like if we tried to write this at compile-time.  The HashMap is used to cache
/// statements and is keyed by table name.
async fn run_query(
    client: &Client,
    cache: &StatementCache,
    table_name: &'static str,
    user_id: &str,
    user_ip: &str,
    factors: &[(&str, &(dyn tokio_postgres::types::ToSql + Sync))],
) -> Result<()> {
    let stmt = {
        if let Some(s) = cache.lock().unwrap().get(table_name) {
            s.clone()
        } else {
            let fact_names: smallvec::SmallVec<[&str; 64]> = factors.iter().map(|x| x.0).collect();
            let stmt = Arc::new(
                client
                    .prepare(&build_query_string(table_name, &fact_names[..]))
                    .await?,
            );
            cache.lock().unwrap().insert(table_name, stmt.clone());
            stmt
        }
    };

    let mut all_params: smallvec::SmallVec<[_; 64]> = factors.iter().map(|x| x.1).collect();
    all_params.push(&user_id);
    all_params.push(&user_ip);

    client.execute(&*stmt, &all_params[..]).await?;
    Ok(())
}

async fn write_cpu_capabilities(
    client: &Client,
    cache: &StatementCache,
    context: &Context,
    work: &WorkItem,
) -> Result<()> {
    let c = &work.payload.simdsp.cpu_capabilities;
    run_query(
        client,
        cache,
        CPU_CAPABILITIES_TABLE,
        &work.payload.machine_id,
        work.ip.as_deref().unwrap_or(UNKNOWN_IP),
        &[
            ("day", &context.day),
            ("application", &context.application),
            ("os", &context.os),
            ("cpu_manufacturer", &context.cpu_manufacturer),
            ("architecture", &context.architecture),
            ("x86_sse2", &c.x86_sse2),
            ("x86_sse3", &c.x86_sse3),
            ("x86_ssse3", &c.x86_ssse3),
            ("x86_sse4_1", &c.x86_sse4_1),
            ("x86_fma3", &c.x86_fma3),
            ("x86_avx", &c.x86_avx),
            ("x86_avx2", &c.x86_avx2),
            ("x86_avx512f", &c.x86_avx512f),
        ],
    )
    .await?;
    Ok(())
}

async fn write_cpu_caches(
    client: &Client,
    cache: &StatementCache,
    context: &Context,
    work: &WorkItem,
) -> Result<()> {
    use crate::anonymization::round_cache;

    fn anon(x: u64) -> i64 {
        round_cache(x) as i64
    }

    let c = &work.payload.simdsp.cache_info;
    run_query(
        client,
        cache,
        CPU_CACHES_TABLE,
        &work.payload.machine_id,
        work.ip.as_deref().unwrap_or(UNKNOWN_IP),
        &[
            ("day", &context.day),
            ("application", &context.application),
            ("l1i", &anon(c.l1i)),
            ("l1d", &anon(c.l1d)),
            ("l1u", &anon(c.l1u)),
            ("l2i", &anon(c.l2i)),
            ("l2d", &anon(c.l2d)),
            ("l2u", &anon(c.l2u)),
            ("l3i", &anon(c.l3i)),
            ("l3d", &anon(c.l3d)),
            ("l3u", &anon(c.l3u)),
        ],
    )
    .await?;
    Ok(())
}

async fn write_memory(
    client: &Client,
    cache: &StatementCache,
    context: &Context,
    work: &WorkItem,
) -> Result<()> {
    use crate::anonymization::round_mem;

    fn anon(x: u64) -> i64 {
        round_mem(x) as i64
    }

    run_query(
        client,
        cache,
        CPU_CACHES_TABLE,
        &work.payload.machine_id,
        work.ip.as_deref().unwrap_or(UNKNOWN_IP),
        &[
            ("day", &context.day),
            ("application", &context.application),
            ("total_memory", &anon(work.payload.memory.total)),
        ],
    )
    .await?;
    Ok(())
}

async fn write_cf_country(
    client: &Client,
    cache: &StatementCache,
    context: &Context,
    work: &WorkItem,
) -> Result<()> {
    let country = work.country.as_deref().unwrap_or(UNKNOWN_COUNTRY);

    // If we somehow get a non-2-character country code here, something is wrong.
    if country.len() != 2 {
        anyhow::bail!("Got country code {} which is invalid", country);
    }

    run_query(
        client,
        cache,
        CF_COUNTRY_TABLE,
        &work.payload.machine_id,
        work.ip.as_deref().unwrap_or(UNKNOWN_IP),
        &[
            ("day", &context.day),
            ("application", &context.application),
            ("country", &country),
        ],
    )
    .await?;
    Ok(())
}

/// Write a work item.
///
/// Logs on failure, and writes what it can.
async fn write_work_item(
    client: &Client,
    cache: &StatementCache,
    context: &Context,
    work: &WorkItem,
) {
    let (caches, caps, mem, country) = tokio::join!(
        write_cpu_caches(client, cache, context, work),
        write_cpu_capabilities(client, cache, context, work),
        write_memory(client, cache, context, work),
        write_cf_country(client, cache, context, work)
    );

    if let Err(e) = caps {
        log::error!("Unable to write CPU capabilities: {:?}", e);
    }
    if let Err(e) = caches {
        log::error!("Unable to write cache info: {:?}", e);
    }
    if let Err(e) = mem {
        log::error!("Unable to write memory info: {:?}", e);
    }
    if let Err(e) = country {
        log::error!("Unable to write country info: {:?}", e);
    }
}

/// Flush a batch.  Handles failures by logging.
async fn flush_batch(batch: &mut Vec<WorkItem>) {
    if batch.is_empty() {
        return;
    }

    log::info!("Would write: {:?}", batch);
    batch.clear();
}

async fn writer_task_fallible(writer: Arc<WriterThread>) -> Result<()> {
    let mut batch: Vec<WorkItem> = vec![];
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

async fn writer_task(
    writer: Arc<WriterThread>,
    client: Client,
    connection_task: tokio::task::JoinHandle<std::result::Result<(), tokio_postgres::Error>>,
) {
    log::info!("Writer running");

    writer_task_fallible(writer)
        .await
        .expect("The writer crashed");
}

pub async fn spawn(db_url: &str) -> Result<Arc<WriterThread>> {
    let (client, connection) = tokio_postgres::connect(db_url, tokio_postgres::NoTls).await?;
    let connection_task = tokio::spawn(connection);

    let uuid_cache = UuidCache::load(&client).await?;
    log::info!("Uuid cache is: {:?}", uuid_cache);

    let (sender, receiver) = bounded(MAX_OUTSTANDING_ITEMS);
    let thread = Arc::new(WriterThread { sender, receiver });

    let thread_cloned = thread.clone();

    tokio::spawn(writer_task(thread_cloned, client, connection_task));

    Ok(thread)
}
