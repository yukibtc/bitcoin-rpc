// Copyright (c) 2021-2024 Yuki Kishimoto
// Distributed under the MIT software license

use std::time::Duration;

use bitcoin::{Block, BlockHash, Transaction, Txid};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Clone, Deserialize)]
struct GenericResult<T> {
    result: Option<T>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlockchainInfo {
    pub chain: String,
    pub blocks: u64,
    pub headers: u64,
    #[serde(rename = "bestblockhash")]
    pub best_block_hash: BlockHash,
    pub difficulty: f64,
    #[serde(rename = "mediantime")]
    pub median_time: u64,
    #[serde(rename = "initialblockdownload")]
    pub initial_block_download: bool,
    pub size_on_disk: u64,
    pub pruned: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NetworkInfo {
    pub version: u32,
    #[serde(rename = "networkactive")]
    pub network_active: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MiningInfo {
    pub blocks: u64,
    pub difficulty: f64,
    #[serde(rename = "networkhashps")]
    pub network_hash_ps: f64,
    #[serde(rename = "pooledtx")]
    pub pooled_tx: usize,
    pub chain: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PeerInfo {
    pub id: u32,
    pub addr: String,
    pub network: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TxIndex {
    pub synced: bool,
    pub best_block_height: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IndexInfo {
    pub txindex: TxIndex,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TxOutSetInfo {
    pub height: u64,
    #[serde(rename = "bestblock")]
    pub best_block: BlockHash,
    #[serde(rename = "txouts")]
    pub tx_outs: u64,
    pub total_amount: f64,
}

#[derive(Clone)]
pub struct Client {
    host: String,
    username: String,
    password: String,
}

#[derive(Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    SerdeJson(serde_json::Error),
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

impl Client {
    pub fn new(host: &str, username: &str, password: &str) -> Self {
        Self {
            host: host.into(),
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

    fn request<R, T>(
        &self,
        method: &str,
        params: &[serde_json::Value],
        timeout: T,
    ) -> Result<R, Error>
    where
        R: DeserializeOwned,
        T: Into<Option<Duration>>,
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

    pub fn get_block_count(&self) -> Result<u64, Error> {
        self.request("getblockcount", &[], None)
    }

    pub fn get_block_hash(&self, block_height: u64) -> Result<BlockHash, Error> {
        self.request("getblockhash", &[block_height.into()], None)
    }

    pub fn get_block(&self, block_hash: &BlockHash) -> Result<Block, Error> {
        self.request(
            "getblock",
            &[into_json(block_hash)?, 2.into()],
            Duration::from_secs(120),
        )
    }

    pub fn get_block_hex(&self, block_hash: &BlockHash) -> Result<String, Error> {
        self.request(
            "getblock",
            &[into_json(block_hash)?, 0.into()],
            Duration::from_secs(120),
        )
    }

    pub fn get_raw_mempool(&self) -> Result<Vec<Txid>, Error> {
        self.request("getrawmempool", &[], Duration::from_secs(120))
    }

    pub fn get_raw_transaction(&self, txid: &Txid) -> Result<Transaction, Error> {
        self.request(
            "getrawtransaction",
            &[into_json(txid)?, true.into()],
            Duration::from_secs(120),
        )
    }

    pub fn get_difficulty(&self) -> Result<f64, Error> {
        self.request("getdifficulty", &[], None)
    }

    pub fn get_tx_out_set_info(&self) -> Result<TxOutSetInfo, Error> {
        self.request("gettxoutsetinfo", &[], Duration::from_secs(1800))
    }
}

/// Shorthand for converting a variable into a serde_json::Value.
fn into_json<T>(val: T) -> Result<serde_json::Value, Error>
where
    T: serde::ser::Serialize,
{
    Ok(serde_json::to_value(val)?)
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Reqwest(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerdeJson(err)
    }
}
