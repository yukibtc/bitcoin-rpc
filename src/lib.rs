// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

#[macro_use]
extern crate serde;

use std::net::SocketAddr;
use std::time::Duration;

use serde::de::DeserializeOwned;
use serde_json::json;

#[derive(Deserialize, Debug)]
struct GenericResult<T> {
    result: Option<T>,
}

#[derive(Deserialize, Debug)]
pub struct BlockchainInfo {
    pub chain: String,
    pub blocks: u32,
    pub headers: u32,
    #[serde(rename = "bestblockhash")]
    pub best_block_hash: String,
    pub difficulty: f64,
    #[serde(rename = "mediantime")]
    pub median_time: u64,
    #[serde(rename = "initialblockdownload")]
    pub initial_block_download: bool,
    pub size_on_disk: u64,
    pub pruned: bool,
}

#[derive(Deserialize, Debug)]
pub struct NetworkInfo {
    pub version: u32,
    #[serde(rename = "networkactive")]
    pub network_active: bool,
}

#[derive(Deserialize, Debug)]
pub struct MiningInfo {
    pub blocks: u32,
    pub difficulty: f64,
    pub networkhashps: f64,
    pub pooledtx: u32,
    pub chain: String,
}

#[derive(Deserialize, Debug)]
pub struct PeerInfo {
    pub id: u32,
    pub addr: String,
    pub network: String,
}

#[derive(Deserialize, Debug)]
pub struct TxIndex {
    pub synced: bool,
    pub best_block_height: u32,
}

#[derive(Deserialize, Debug)]
pub struct IndexInfo {
    pub txindex: TxIndex,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TxIn {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub txid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vout: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prevout: Option<TxOut>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct TxOutScripts {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct TxOut {
    pub value: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    #[serde(rename = "scriptPubKey")]
    pub script_pub_key: TxOutScripts,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Transaction {
    pub txid: String,
    pub vin: Vec<TxIn>,
    pub vout: Vec<TxOut>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Block {
    pub hash: String,
    pub confirmations: u32,
    pub size: u32,
    pub height: u32,
    pub version: u32,
    pub tx: Vec<Transaction>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TxOutSetInfo {
    pub height: u32,
    pub bestblock: String,
    pub txouts: u32,
    pub total_amount: f64,
}

#[derive(Clone)]
pub struct RpcClient {
    host: String,
    username: String,
    password: String,
}

#[derive(Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    FailedToDeserialize(String),
    BadResult,
    Unauthorized,
    BadRequest,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    TooManyRequests,
    UnhandledClientError,
    InternalServerError,
    NotImplemented,
    BadGateway,
    ServiceUnavailable,
    GatewayTimeout,
    UnhandledServerError,
}

impl RpcClient {
    pub fn new(addr: &SocketAddr, username: &str, password: &str) -> Self {
        Self {
            host: format!("http://{}", *addr),
            username: username.into(),
            password: password.into(),
        }
    }

    fn call_jsonrpc<T>(
        &self,
        method: &str,
        params: &[serde_json::Value],
        timeout: T,
    ) -> Result<String, Error>
    where
        T: Into<Option<Duration>>,
    {
        let body: String = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        })
        .to_string();

        let client = reqwest::blocking::Client::builder()
            .timeout(timeout)
            .build()?;

        let res = client
            .post(&self.host)
            .basic_auth(self.username.as_str(), Some(self.password.as_str()))
            .body(body)
            .send()?;

        match reqwest::StatusCode::as_u16(&res.status()) {
            0_u16..=399_u16 => Ok(res.text()?),
            400 => Err(Error::BadRequest),
            401 => Err(Error::Unauthorized),
            402 => Err(Error::UnhandledClientError),
            403 => Err(Error::Forbidden),
            404 => Err(Error::NotFound),
            405 => Err(Error::MethodNotAllowed),
            406_u16..=428_u16 => Err(Error::UnhandledClientError),
            429 => Err(Error::TooManyRequests),
            430_u16..=499_u16 => Err(Error::UnhandledClientError),
            500 => Err(Error::InternalServerError),
            501 => Err(Error::NotImplemented),
            502 => Err(Error::BadGateway),
            503 => Err(Error::ServiceUnavailable),
            504 => Err(Error::GatewayTimeout),
            _ => Err(Error::UnhandledServerError),
        }
    }

    fn deserialize<T>(data: String) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        match serde_json::from_str::<GenericResult<T>>(data.as_str()) {
            Ok(u) => match u.result {
                Some(data) => Ok(data),
                None => Err(Error::BadResult),
            },
            Err(error) => Err(Error::FailedToDeserialize(error.to_string())),
        }
    }

    fn request<R, T>(&self, method: &str, params: &[serde_json::Value], timeout: T) -> Result<R, Error>
    where
        R: DeserializeOwned,
        T: Into<Option<Duration>>
    {
        let response = self.call_jsonrpc(method, params, timeout)?;
        Self::deserialize::<R>(response)
    }

    pub fn get_blockchain_info(&self) -> Result<BlockchainInfo, Error> {
        self.request("getblockchaininfo", &[], None)
    }

    pub fn get_network_info(&self) -> Result<NetworkInfo, Error> {
        self.request("getnetworkinfo", &[], None)
    }

    pub fn get_mining_info(&self) -> Result<MiningInfo, Error> {
        self.request("getmininginfo", &[], None)
    }

    pub fn get_peer_info(&self) -> Result<Vec<PeerInfo>, Error> {
        self.request("getpeerinfo", &[], None)
    }

    pub fn get_index_info(&self) -> Result<IndexInfo, Error> {
        self.request("getindexinfo", &[], None)
    }

    pub fn get_block_count(&self) -> Result<u32, Error> {
        self.request("getblockcount", &[], None)
    }

    pub fn get_block_hash(&self, block_height: u32) -> Result<String, Error> {
        self.request("getblockhash", &[block_height.into()], None)
    }

    pub fn get_block(&self, block_hash: &str) -> Result<Block, Error> {
        self.request("getblock", &[block_hash.into(), 2.into()], Duration::from_secs(120))
    }

    pub fn get_block_hex(&self, block_hash: &str) -> Result<String, Error> {
        self.request("getblock", &[block_hash.into(), 0.into()], Duration::from_secs(120))
    }

    pub fn get_raw_mempool(&self) -> Result<Vec<String>, Error> {
        self.request("getrawmempool", &[], Duration::from_secs(120))
    }

    pub fn get_raw_transaction(&self, txid: &str) -> Result<Transaction, Error> {
        self.request("getrawtransaction", &[txid.into(), true.into()], Duration::from_secs(120))
    }

    pub fn get_raw_transaction_with_prevouts(&self, txid: &str) -> Result<Transaction, Error> {
        let mut raw_transaction = self.get_raw_transaction(txid)?;

        raw_transaction.vin.iter_mut().for_each(|input| {
            if let Some(input_txid) = &input.txid {
                if let Some(vout) = input.vout {
                    if let Ok(prev_raw_transaction) = self.get_raw_transaction(input_txid.as_str())
                    {
                        prev_raw_transaction.vout.into_iter().for_each(|output| {
                            if let Some(output_n) = output.n {
                                if output_n == vout {
                                    input.prevout = Some(output);
                                }
                            }
                        });
                    }
                }
            }
        });

        Ok(raw_transaction)
    }

    pub fn get_difficulty(&self) -> Result<f64, Error> {
        self.request("getdifficulty", &[], None)
    }

    pub fn get_txoutset_info(&self) -> Result<TxOutSetInfo, Error> {
        self.request("gettxoutsetinfo", &[], Duration::from_secs(1800))
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Reqwest(err)
    }
}
