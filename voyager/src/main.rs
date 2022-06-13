use clap::Parser;

#[derive(Parser)]
struct Args {
    url: String,
    token: String,
}

fn main() {
    env_logger::init();

    let args = Args::parse();
    hwsurvey_client::send_synchronously(args.url, args.token, 1);
}
