mod api;
mod rows;
mod writer;

use std::str::FromStr;

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

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    env_logger::init();

    let args = Args::parse();

    let ip: std::net::IpAddr =
        std::net::IpAddr::from_str(&args.address).expect("Could not parse IP address");

    let writer = writer::spawn();

    let report = warp::path!("report" / "v1")
        .and(warp::post())
        .and(warp::filters::body::content_length_limit(1024 * 10))
        .and(warp::filters::body::bytes())
        .then(move |b| api::report_v1::report_v1(writer.clone(), b));

    warp::serve(report).run((ip, args.port)).await;
}
