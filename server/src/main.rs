mod api;
mod rows;
mod writer;

use std::str::FromStr;

use anyhow::Result;
use clap::Parser;
use warp::Filter;

#[derive(Parser)]
struct Args {
    #[clap(default_value_t=String::from("127.0.0.1"))]
    #[clap(long = "--address")]
    address: String,

    #[clap(default_value_t = 10000)]
    #[clap(long = "--port")]
    port: u16,
}

refinery::embed_migrations!();

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();

    let ip: std::net::IpAddr =
        std::net::IpAddr::from_str(&args.address).expect("Could not parse IP address");

    let dburl = std::env::var("DATABASE_URL").expect("DATABASE_URL env var must be set");
    let (mut db_cli, db_conn) = tokio_postgres::connect(&dburl, tokio_postgres::tls::NoTls).await?;
    tokio::task::spawn(async move {
        db_conn.await.expect("Fatal database error");
    });

    log::info!("Connected to database. Applying migrations...");
    migrations::runner().run_async(&mut db_cli).await?;

    let writer = writer::spawn();

    let report = warp::path!("report" / "v1")
        .and(warp::post())
        .and(warp::filters::body::content_length_limit(1024 * 10))
        .and(warp::filters::body::bytes())
        .then(move |b| api::report_v1::report_v1(writer.clone(), b));

    warp::serve(report).run((ip, args.port)).await;
    Ok(())
}
