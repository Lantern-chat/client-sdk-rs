use client_sdk::models::gateway::message::ServerMsg;

fn main() {
    let schema = schemars::schema_for!(ServerMsg);
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
