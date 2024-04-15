// Copyright (c) 2021-2024 Yuki Kishimoto
// Distributed under the MIT software license

use bitcoin_rpc::Client;

fn main() {
    let rpc = Client::new("http://127.0.0.1:8332", "username", "password");

    println!("{:#?}", rpc.get_blockchain_info().unwrap());

    println!("{:#?}", rpc.get_mining_info().unwrap());

    println!("Difficulty: {}", rpc.get_difficulty().unwrap());

    println!("{:#?}", rpc.get_tx_out_set_info().unwrap());
}
