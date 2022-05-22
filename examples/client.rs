// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

extern crate bitcoin_rpc;

use bitcoin_rpc::RpcClient;
use std::net::SocketAddr;

fn main() {
    let addr: SocketAddr = "127.0.0.1:8332".parse::<SocketAddr>().unwrap();
    let rpc = RpcClient::new(&addr, "username", "password");

    println!("{:#?}", rpc.get_blockchain_info().unwrap());

    println!("{:#?}", rpc.get_mining_info().unwrap());

    println!("Difficulty: {}", rpc.get_difficulty().unwrap());

    println!("{:#?}", rpc.get_txoutset_info().unwrap());
}
