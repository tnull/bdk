//! structs from the esplora API
//!
//! see: <https://github.com/Blockstream/esplora/blob/master/API.md>
use crate::{BlockTime, Error};
use bitcoin::{OutPoint, Script, Transaction, TxIn, TxOut, Txid, Witness, BlockHash};

#[derive(serde::Deserialize, Clone, Debug, PartialEq)]
pub struct PrevOut {
    pub value: u64,
    pub scriptpubkey: Script,
}

#[derive(serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Vin {
    pub txid: Txid,
    pub vout: u32,
    // None if coinbase
    pub prevout: Option<PrevOut>,
    pub scriptsig: Script,
    #[serde(deserialize_with = "deserialize_witness", default)]
    pub witness: Vec<Vec<u8>>,
    pub sequence: u32,
    pub is_coinbase: bool,
}

#[derive(serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Vout {
    pub value: u64,
    pub scriptpubkey: Script,
}

#[maybe_async]
/// Trait for getting the status of a transaction by txid
pub trait GetTxStatus {
    /// Fetch the status of a transaction given its txid
    fn get_tx_status(&self, txid: &Txid) -> Result<Option<TxStatus>, Error>;
}

#[derive(serde::Deserialize, Clone, Debug, PartialEq)]
pub struct TxStatus {
    pub confirmed: bool,
    pub block_height: Option<u32>,
	pub block_hash: Option<BlockHash>,
    pub block_time: Option<u64>,
}

#[maybe_async]
/// Trait for getting a merkle proof of inclusion for a transaction
pub trait GetMerkleProof {
    /// Fetch the merkle proof of a transaction given its txid
    fn get_merkle_proof(&self, txid: &Txid, block_height: u32) -> Result<Option<MerkleProof>, Error>;
}

#[derive(serde::Deserialize, Clone, Debug, PartialEq)]
pub struct MerkleProof {
    block_height: u32,
    merkle: Vec<Txid>,
    pos: usize,
}

#[maybe_async]
/// Trait for getting the spending status of an output
pub trait GetOutputStatus {
    /// Fetch the output spending status given a txid and vout
    fn get_output_status(&self, txid: &Txid, vout: &Vout) -> Result<Option<OutputStatus>, Error>;
}

#[derive(serde::Deserialize, Clone, Debug, PartialEq)]
pub struct OutputStatus {
	spent: bool,
	txid: Option<Txid>,
	vin: Option<Vin>,
	status: Option<TxStatus>,
}


#[derive(serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Tx {
    pub txid: Txid,
    pub version: i32,
    pub locktime: u32,
    pub vin: Vec<Vin>,
    pub vout: Vec<Vout>,
    pub status: TxStatus,
    pub fee: u64,
}

impl Tx {
    pub fn to_tx(&self) -> Transaction {
        Transaction {
            version: self.version,
            lock_time: self.locktime,
            input: self
                .vin
                .iter()
                .cloned()
                .map(|vin| TxIn {
                    previous_output: OutPoint {
                        txid: vin.txid,
                        vout: vin.vout,
                    },
                    script_sig: vin.scriptsig,
                    sequence: vin.sequence,
                    witness: Witness::from_vec(vin.witness),
                })
                .collect(),
            output: self
                .vout
                .iter()
                .cloned()
                .map(|vout| TxOut {
                    value: vout.value,
                    script_pubkey: vout.scriptpubkey,
                })
                .collect(),
        }
    }

    pub fn confirmation_time(&self) -> Option<BlockTime> {
        match self.status {
            TxStatus {
                confirmed: true,
                block_height: Some(height),
                block_time: Some(timestamp),
				..
            } => Some(BlockTime { timestamp, height }),
            _ => None,
        }
    }

    pub fn previous_outputs(&self) -> Vec<Option<TxOut>> {
        self.vin
            .iter()
            .cloned()
            .map(|vin| {
                vin.prevout.map(|po| TxOut {
                    script_pubkey: po.scriptpubkey,
                    value: po.value,
                })
            })
            .collect()
    }
}

fn deserialize_witness<'de, D>(d: D) -> Result<Vec<Vec<u8>>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    use crate::serde::Deserialize;
    use bitcoin::hashes::hex::FromHex;
    let list = Vec::<String>::deserialize(d)?;
    list.into_iter()
        .map(|hex_str| Vec::<u8>::from_hex(&hex_str))
        .collect::<Result<Vec<Vec<u8>>, _>>()
        .map_err(serde::de::Error::custom)
}
