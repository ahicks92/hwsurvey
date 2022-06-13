mod anonymization;
mod api;
mod writer;

use std::str::FromStr;
use std::time::Duration;

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

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();

    let ip: std::net::IpAddr =
        std::net::IpAddr::from_str(&args.address).expect("Could not parse IP address");

    let dburl = std::env::var("DATABASE_URL").expect("DATABASE_URL env var must be set");
    let writer = writer::spawn(&dburl).await?;

    let cf_ip = warp::header("CF-Connecting-IP").map(|x: String| Some(x));
    let remote_ip = warp::filters::addr::remote().map(|x: Option<std::net::SocketAddr>| {
        // This half of the filter only matches if the cloudflare header wasn't present, so log here.
        //
        // Also, if we got here and didn't get the ip at all, something is weird about our deployment, so log that too.
        only_every::only_every!(
            Duration::from_secs(30),
            log::warn!("Unable to get IP from cloudflare. Falling back to remote addr")
        );
        if x.is_none() {
            only_every::only_every!(
                Duration::from_secs(30),
                log::error!("Warp is unable to extract the remote address")
            );
        }
        x.map(|y| y.ip().to_string())
    });
    let ip_filter = cf_ip.or(remote_ip).unify();

    // Extract a country code as a 2-character array.
    let country_filter =
        warp::filters::header::optional::<String>("CF-IPCountry").map(|x: Option<String>| {
            if x.is_none() {
                only_every::only_every!(
                    Duration::from_secs(30),
                    log::warn!("Missing CF-IPCountry header")
                );
            }
            x
        });

    let submit = warp::path!("submit" / "v1")
        .and(warp::post())
        .and(warp::filters::body::content_length_limit(1024 * 10))
        .and(ip_filter)
        .and(country_filter)
        .and(warp::filters::body::bytes())
        .then(move |ip, country, body| {
            api::submit_v1::submit_v1(writer.clone(), ip, country, body)
        });

    warp::serve(submit).run((ip, args.port)).await;
    Ok(())
}
