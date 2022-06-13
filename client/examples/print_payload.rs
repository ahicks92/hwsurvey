//! For debugging, prints the payload that we would send.

fn main() {
    println!("{:?}", hwsurvey_client::build_payload::build_payload());
}
