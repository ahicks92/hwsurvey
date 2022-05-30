pub mod rows;
pub mod writer;

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

    let echoback = warp::path!("echoback")
        .and(warp::post())
        .and(warp::body::bytes())
        .and(warp::filters::body::content_length_limit(1024 * 10))
        .map(|body: bytes::Bytes| body.to_vec());

    let ip: std::net::IpAddr =
        std::net::IpAddr::from_str(&args.address).expect("Could not parse IP address");

    writer::spawn();
    warp::serve(echoback).run((ip, args.port)).await;
}
