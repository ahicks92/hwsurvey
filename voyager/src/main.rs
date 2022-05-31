use clap::Parser;

#[derive(Parser)]
struct Args {
    url: String,
    app_name: String,
}

fn main() {
    env_logger::init();

    let args = Args::parse();
    hwsurvey_client::send_synchronously(args.url, args.app_name, 1);
}
