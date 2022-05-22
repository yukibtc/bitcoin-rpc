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
pub enum RpcError {
    ReqwestError(reqwest::Error),
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
    ) -> Result<String, RpcError>
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
            400 => Err(RpcError::BadRequest),
            401 => Err(RpcError::Unauthorized),
            402 => Err(RpcError::UnhandledClientError),
            403 => Err(RpcError::Forbidden),
            404 => Err(RpcError::NotFound),
            405 => Err(RpcError::MethodNotAllowed),
            406_u16..=428_u16 => Err(RpcError::UnhandledClientError),
            429 => Err(RpcError::TooManyRequests),
            430_u16..=499_u16 => Err(RpcError::UnhandledClientError),
            500 => Err(RpcError::InternalServerError),
            501 => Err(RpcError::NotImplemented),
            502 => Err(RpcError::BadGateway),
            503 => Err(RpcError::ServiceUnavailable),
            504 => Err(RpcError::GatewayTimeout),
            _ => Err(RpcError::UnhandledServerError),
        }
    }

    fn deserialize<T>(data: String) -> Result<T, RpcError>
    where
        T: DeserializeOwned,
    {
        match serde_json::from_str::<GenericResult<T>>(data.as_str()) {
            Ok(u) => match u.result {
                Some(data) => Ok(data),
                None => Err(RpcError::BadResult),
            },
            Err(error) => Err(RpcError::FailedToDeserialize(error.to_string())),
        }
    }

    fn request<R, T>(&self, method: &str, params: &[serde_json::Value], timeout: T) -> Result<R, RpcError>
    where
        R: DeserializeOwned,
        T: Into<Option<Duration>>
    {
        let response = self.call_jsonrpc(method, params, timeout)?;
        Self::deserialize::<R>(response)
    }

    pub fn get_blockchain_info(&self) -> Result<BlockchainInfo, RpcError> {
        self.request("getblockchaininfo", &[], None)
    }

    pub fn get_network_info(&self) -> Result<NetworkInfo, RpcError> {
        self.request("getnetworkinfo", &[], None)
    }

    pub fn get_mining_info(&self) -> Result<MiningInfo, RpcError> {
        self.request("getmininginfo", &[], None)
    }

    pub fn get_peer_info(&self) -> Result<Vec<PeerInfo>, RpcError> {
        self.request("getpeerinfo", &[], None)
    }

    pub fn get_index_info(&self) -> Result<IndexInfo, RpcError> {
        self.request("getindexinfo", &[], None)
    }

    pub fn get_block_count(&self) -> Result<u32, RpcError> {
        self.request("getblockcount", &[], None)
    }

    pub fn get_block_hash(&self, block_height: u32) -> Result<String, RpcError> {
        self.request("getblockhash", &[block_height.into()], None)
    }

    pub fn get_block(&self, block_hash: &str) -> Result<Block, RpcError> {
        self.request("getblock", &[block_hash.into(), 2.into()], Duration::from_secs(120))
    }

    pub fn get_raw_mempool(&self) -> Result<Vec<String>, RpcError> {
        self.request("getrawmempool", &[], Duration::from_secs(120))
    }

    pub fn get_raw_transaction(&self, txid: &str) -> Result<Transaction, RpcError> {
        self.request("getrawtransaction", &[txid.into(), true.into()], Duration::from_secs(120))
    }

    pub fn get_raw_transaction_with_prevouts(&self, txid: &str) -> Result<Transaction, RpcError> {
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

    pub fn get_difficulty(&self) -> Result<f64, RpcError> {
        self.request("getdifficulty", &[], None)
    }

    pub fn get_txoutset_info(&self) -> Result<TxOutSetInfo, RpcError> {
        self.request("gettxoutsetinfo", &[], Duration::from_secs(1800))
    }
}

impl From<reqwest::Error> for RpcError {
    fn from(err: reqwest::Error) -> Self {
        RpcError::ReqwestError(err)
    }
}
