use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    Address, BlockAccessTrace, BlockRef, ChainKind, StorageKey, TraceAccess, TraceAccessKey,
    TraceAccessKind, TraceParseWarning, TxHash, TxIndex, TxTrace,
};

pub const TRACE_PACK_SCHEMA_VERSION: &str = "trace-pack-v1";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TracePack {
    pub manifest: TracePackManifest,
    pub blocks: Vec<TracePackBlock>,
}

impl TracePack {
    pub fn load_dir(path: &Path) -> Result<Self, TracePackError> {
        let manifest_path = path.join("manifest.json");
        let manifest: TracePackManifest = read_json(&manifest_path)?;
        let mut blocks = Vec::new();
        for number in manifest.start_block..=manifest.end_block {
            let block_path = block_path(path, number);
            if !block_path.exists() {
                return Err(TracePackError::MissingBlockFile {
                    block_number: number,
                    path: block_path,
                });
            }
            blocks.push(read_json(&block_path)?);
        }
        let mut pack = Self { manifest, blocks };
        pack.normalize();
        pack.validate()?;
        Ok(pack)
    }

    pub fn write_dir(&self, path: &Path) -> Result<(), TracePackError> {
        let mut pack = self.clone();
        pack.normalize();
        pack.validate()?;
        std::fs::create_dir_all(path.join("blocks"))?;
        write_json(&path.join("manifest.json"), &pack.manifest)?;
        for block in &pack.blocks {
            write_json(&block_path(path, block.block_number), block)?;
        }
        Ok(())
    }

    pub fn normalize(&mut self) {
        self.manifest.normalize();
        for block in &mut self.blocks {
            block.normalize();
        }
        self.blocks.sort_by_key(|block| block.block_number);
    }

    pub fn validate(&self) -> Result<(), TracePackError> {
        self.manifest.validate()?;
        let mut blocks_by_number = BTreeMap::new();
        for block in &self.blocks {
            if blocks_by_number.insert(block.block_number, block).is_some() {
                return Err(TracePackError::DuplicateBlock(block.block_number));
            }
            if block.chain != self.manifest.chain {
                return Err(TracePackError::InvalidBlock {
                    block_number: block.block_number,
                    reason: format!(
                        "block chain `{}` does not match manifest chain `{}`",
                        block.chain, self.manifest.chain
                    ),
                });
            }
            if block.block_number < self.manifest.start_block
                || block.block_number > self.manifest.end_block
            {
                return Err(TracePackError::InvalidBlock {
                    block_number: block.block_number,
                    reason: "block number outside manifest range".to_owned(),
                });
            }
            block.validate()?;
        }
        let expected = self.manifest.end_block - self.manifest.start_block + 1;
        if self.blocks.len() as u64 != expected {
            return Err(TracePackError::InvalidManifest {
                reason: format!(
                    "manifest range expects {expected} block files but loaded {}",
                    self.blocks.len()
                ),
            });
        }
        for number in self.manifest.start_block..=self.manifest.end_block {
            if !blocks_by_number.contains_key(&number) {
                return Err(TracePackError::InvalidManifest {
                    reason: format!("manifest range is missing block {number}"),
                });
            }
        }
        if self.manifest.start_block < self.manifest.end_block {
            for number in (self.manifest.start_block + 1)..=self.manifest.end_block {
                let previous = blocks_by_number
                    .get(&(number - 1))
                    .expect("previous block exists after range check");
                let current = blocks_by_number
                    .get(&number)
                    .expect("current block exists after range check");
                if let (Some(previous_hash), Some(parent_hash)) =
                    (&previous.block_hash, &current.parent_hash)
                {
                    if parent_hash != previous_hash {
                        return Err(TracePackError::InvalidBlock {
                            block_number: current.block_number,
                            reason: format!(
                                "parent_hash `{parent_hash}` does not match previous block {} hash `{previous_hash}`",
                                previous.block_number
                            ),
                        });
                    }
                }
            }
        }
        Ok(())
    }

    pub fn to_block_traces(&self) -> Vec<BlockAccessTrace> {
        self.blocks
            .iter()
            .map(TracePackBlock::to_block_trace)
            .collect()
    }

    pub fn has_complete_gas(&self) -> bool {
        self.blocks.iter().all(TracePackBlock::has_complete_gas)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TracePackManifest {
    pub schema_version: String,
    pub chain: ChainKind,
    pub source: String,
    pub provenance: String,
    pub start_block: u64,
    pub end_block: u64,
    pub created_by_tool_version: String,
    pub tracer_kind: String,
    #[serde(default)]
    pub notes: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
}

impl TracePackManifest {
    pub fn normalize(&mut self) {
        self.chain = ChainKind::new(self.chain.0.trim().to_ascii_lowercase());
        normalize_strings(&mut self.notes);
        normalize_strings(&mut self.warnings);
    }

    pub fn validate(&self) -> Result<(), TracePackError> {
        if self.schema_version != TRACE_PACK_SCHEMA_VERSION {
            return Err(TracePackError::InvalidManifest {
                reason: format!(
                    "unsupported schema_version `{}`; expected `{}`",
                    self.schema_version, TRACE_PACK_SCHEMA_VERSION
                ),
            });
        }
        if self.chain.0.trim().is_empty() {
            return Err(TracePackError::InvalidManifest {
                reason: "chain must not be empty".to_owned(),
            });
        }
        if self.source.trim().is_empty() {
            return Err(TracePackError::InvalidManifest {
                reason: "source must not be empty".to_owned(),
            });
        }
        if self.provenance.trim().is_empty() {
            return Err(TracePackError::InvalidManifest {
                reason: "provenance must not be empty".to_owned(),
            });
        }
        if self.tracer_kind.trim().is_empty() {
            return Err(TracePackError::InvalidManifest {
                reason: "tracer_kind must not be empty".to_owned(),
            });
        }
        if self.start_block > self.end_block {
            return Err(TracePackError::InvalidManifest {
                reason: "start_block must be <= end_block".to_owned(),
            });
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TracePackBlock {
    pub chain: ChainKind,
    pub block_number: u64,
    pub block_hash: Option<String>,
    pub parent_hash: Option<String>,
    pub tx_count: usize,
    pub total_gas_used: Option<u64>,
    #[serde(default)]
    pub transactions: Vec<TracePackTx>,
    #[serde(default)]
    pub warnings: Vec<String>,
}

impl TracePackBlock {
    pub fn normalize(&mut self) {
        self.chain = ChainKind::new(self.chain.0.trim().to_ascii_lowercase());
        if let Some(hash) = &mut self.block_hash {
            *hash = hash.trim().to_ascii_lowercase();
        }
        if let Some(hash) = &mut self.parent_hash {
            *hash = hash.trim().to_ascii_lowercase();
        }
        normalize_strings(&mut self.warnings);
        for tx in &mut self.transactions {
            tx.normalize();
        }
        self.transactions.sort_by_key(|tx| tx.tx_index);
    }

    pub fn validate(&self) -> Result<(), TracePackError> {
        validate_optional_hash(self.block_number, "block_hash", self.block_hash.as_deref())?;
        validate_optional_hash(
            self.block_number,
            "parent_hash",
            self.parent_hash.as_deref(),
        )?;
        if self.tx_count != self.transactions.len() {
            return Err(TracePackError::InvalidBlock {
                block_number: self.block_number,
                reason: format!(
                    "tx_count {} does not match {} transactions",
                    self.tx_count,
                    self.transactions.len()
                ),
            });
        }
        let mut seen = BTreeSet::new();
        for tx in &self.transactions {
            if !seen.insert(tx.tx_index) {
                return Err(TracePackError::DuplicateTxIndex {
                    block_number: self.block_number,
                    tx_index: tx.tx_index.0,
                });
            }
            tx.validate(self.block_number)?;
        }
        if let Some(total_gas_used) = self.total_gas_used {
            let mut complete = true;
            let mut tx_gas_sum = 0_u64;
            for tx in &self.transactions {
                let Some(gas_used) = tx.gas_used else {
                    complete = false;
                    break;
                };
                tx_gas_sum = tx_gas_sum.checked_add(gas_used).ok_or_else(|| {
                    TracePackError::InvalidBlock {
                        block_number: self.block_number,
                        reason: "sum of transaction gas_used values overflows u64".to_owned(),
                    }
                })?;
            }
            if complete && total_gas_used != tx_gas_sum {
                return Err(TracePackError::InvalidBlock {
                    block_number: self.block_number,
                    reason: format!(
                        "total_gas_used {total_gas_used} does not match sum of transaction gas_used values {tx_gas_sum}"
                    ),
                });
            }
        }
        Ok(())
    }

    pub fn has_complete_gas(&self) -> bool {
        self.total_gas_used.is_some() && self.transactions.iter().all(|tx| tx.gas_used.is_some())
    }

    pub fn to_block_trace(&self) -> BlockAccessTrace {
        let mut warnings = self
            .warnings
            .iter()
            .map(|message| TraceParseWarning {
                tx_index: None,
                message: message.clone(),
            })
            .collect::<Vec<_>>();
        warnings.push(TraceParseWarning {
            tx_index: None,
            message: "trace pack access observations may be incomplete; gas-weighted scheduling is theoretical".to_owned(),
        });

        let mut trace = BlockAccessTrace {
            chain: self.chain.clone(),
            block: BlockRef {
                number: self.block_number,
                hash: self.block_hash.clone(),
            },
            transactions: self
                .transactions
                .iter()
                .map(|tx| tx.to_tx_trace(self.block_number))
                .collect(),
            warnings,
        };
        trace.normalize();
        trace
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TracePackTx {
    pub tx_index: TxIndex,
    pub tx_hash: TxHash,
    pub from: Option<Address>,
    pub to: Option<Address>,
    pub gas_used: Option<u64>,
    pub status: Option<String>,
    #[serde(default)]
    pub accesses: Vec<TracePackAccess>,
    #[serde(default)]
    pub warnings: Vec<String>,
}

impl TracePackTx {
    fn normalize(&mut self) {
        if let Some(from) = &mut self.from {
            *from = Address::canonical(&from.0);
        }
        if let Some(to) = &mut self.to {
            *to = Address::canonical(&to.0);
        }
        if let Some(status) = &mut self.status {
            *status = status.trim().to_ascii_lowercase();
        }
        normalize_strings(&mut self.warnings);
        for access in &mut self.accesses {
            access.normalize();
        }
        self.accesses.sort();
        self.accesses.dedup();
    }

    fn validate(&self, block_number: u64) -> Result<(), TracePackError> {
        if !self.tx_hash.is_valid() {
            return Err(TracePackError::InvalidTxHash {
                block_number,
                tx_index: self.tx_index.0,
                value: self.tx_hash.0.clone(),
            });
        }
        if let Some(from) = &self.from {
            validate_address(block_number, self.tx_index, "from", from)?;
        }
        if let Some(to) = &self.to {
            validate_address(block_number, self.tx_index, "to", to)?;
        }
        for (access_index, access) in self.accesses.iter().enumerate() {
            access.validate(block_number, self.tx_index, access_index)?;
        }
        Ok(())
    }

    fn to_tx_trace(&self, block_number: u64) -> TxTrace {
        let mut warnings = self
            .warnings
            .iter()
            .map(|message| TraceParseWarning {
                tx_index: Some(self.tx_index),
                message: message.clone(),
            })
            .collect::<Vec<_>>();
        let from = match &self.from {
            Some(from) => from.clone(),
            None => {
                warnings.push(TraceParseWarning {
                    tx_index: Some(self.tx_index),
                    message: format!(
                        "trace pack block {block_number} missing sender; zero address substituted for normalized analysis"
                    ),
                });
                Address::canonical("0x0000000000000000000000000000000000000000")
            }
        };
        TxTrace {
            tx_index: self.tx_index,
            tx_hash: self.tx_hash.clone(),
            from,
            to: self.to.clone(),
            read_info_complete: false,
            accesses: self
                .accesses
                .iter()
                .map(TracePackAccess::to_trace_access)
                .collect(),
            warnings,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct TracePackAccess {
    pub address: Address,
    pub slot: Option<StorageKey>,
    pub kind: TracePackAccessKind,
    pub op: Option<String>,
    pub pc: Option<u64>,
    pub depth: Option<u64>,
    pub gas_remaining: Option<u64>,
}

impl TracePackAccess {
    fn normalize(&mut self) {
        self.address = Address::canonical(&self.address.0);
        if let Some(slot) = &mut self.slot {
            *slot = StorageKey::canonical(&slot.0);
        }
        if let Some(op) = &mut self.op {
            *op = op.trim().to_ascii_uppercase();
        }
    }

    fn validate(
        &self,
        block_number: u64,
        tx_index: TxIndex,
        access_index: usize,
    ) -> Result<(), TracePackError> {
        if !self.address.is_valid() {
            return Err(TracePackError::InvalidAddress {
                block_number,
                tx_index: tx_index.0,
                location: format!("access {access_index} address"),
                value: self.address.0.clone(),
            });
        }
        if let Some(slot) = &self.slot {
            if !slot.is_valid() {
                return Err(TracePackError::InvalidStorageKey {
                    block_number,
                    tx_index: tx_index.0,
                    access_index,
                    value: slot.0.clone(),
                });
            }
        }
        if matches!(self.op.as_deref(), Some("SLOAD" | "SSTORE")) && self.slot.is_none() {
            return Err(TracePackError::InvalidAccess {
                block_number,
                tx_index: tx_index.0,
                access_index,
                reason: "SLOAD/SSTORE access must include a storage slot".to_owned(),
            });
        }
        Ok(())
    }

    fn to_trace_access(&self) -> TraceAccess {
        let key = match &self.slot {
            Some(slot) => TraceAccessKey::Storage {
                address: self.address.clone(),
                slot: slot.clone(),
            },
            None => TraceAccessKey::Account {
                address: self.address.clone(),
            },
        };
        let kind = match self.kind {
            TracePackAccessKind::Read | TracePackAccessKind::Call => TraceAccessKind::Read,
            TracePackAccessKind::Write
            | TracePackAccessKind::Create
            | TracePackAccessKind::Selfdestruct => TraceAccessKind::Write,
            TracePackAccessKind::ReadWrite | TracePackAccessKind::AccountTouch => {
                TraceAccessKind::ReadWrite
            }
        };
        TraceAccess { kind, key }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TracePackAccessKind {
    Read,
    Write,
    ReadWrite,
    AccountTouch,
    Call,
    Create,
    Selfdestruct,
}

#[derive(Debug, thiserror::Error)]
pub enum TracePackError {
    #[error("failed to read/write trace pack file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse trace pack JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("missing trace-pack block file for block {block_number}: {path}")]
    MissingBlockFile { block_number: u64, path: PathBuf },
    #[error("invalid trace-pack manifest: {reason}")]
    InvalidManifest { reason: String },
    #[error("trace pack contains duplicate block {0}")]
    DuplicateBlock(u64),
    #[error("invalid trace-pack block {block_number}: {reason}")]
    InvalidBlock { block_number: u64, reason: String },
    #[error("trace pack block {block_number} contains duplicate tx_index {tx_index}")]
    DuplicateTxIndex { block_number: u64, tx_index: u64 },
    #[error(
        "invalid tx hash at block {block_number} tx_index {tx_index}: `{value}`; expected 0x followed by 64 hex characters"
    )]
    InvalidTxHash {
        block_number: u64,
        tx_index: u64,
        value: String,
    },
    #[error(
        "invalid address at block {block_number} tx_index {tx_index} {location}: `{value}`; expected 0x followed by 40 hex characters"
    )]
    InvalidAddress {
        block_number: u64,
        tx_index: u64,
        location: String,
        value: String,
    },
    #[error(
        "invalid storage key at block {block_number} tx_index {tx_index} access {access_index}: `{value}`; expected 0x followed by 64 hex characters"
    )]
    InvalidStorageKey {
        block_number: u64,
        tx_index: u64,
        access_index: usize,
        value: String,
    },
    #[error("invalid access at block {block_number} tx_index {tx_index} access {access_index}: {reason}")]
    InvalidAccess {
        block_number: u64,
        tx_index: u64,
        access_index: usize,
        reason: String,
    },
}

fn read_json<T>(path: &Path) -> Result<T, TracePackError>
where
    T: for<'de> Deserialize<'de>,
{
    let file = File::open(path)?;
    Ok(serde_json::from_reader(file)?)
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<(), TracePackError> {
    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, value)?;
    Ok(())
}

fn block_path(root: &Path, number: u64) -> PathBuf {
    root.join("blocks").join(format!("{number}.json"))
}

fn normalize_strings(values: &mut Vec<String>) {
    values
        .iter_mut()
        .for_each(|value| *value = value.trim().to_owned());
    values.sort();
    values.dedup();
}

fn validate_optional_hash(
    block_number: u64,
    location: &str,
    value: Option<&str>,
) -> Result<(), TracePackError> {
    if let Some(value) = value {
        if !is_prefixed_hex(value, 64) {
            return Err(TracePackError::InvalidBlock {
                block_number,
                reason: format!(
                    "invalid {location} `{value}`; expected 0x followed by 64 hex characters"
                ),
            });
        }
    }
    Ok(())
}

fn validate_address(
    block_number: u64,
    tx_index: TxIndex,
    location: &str,
    address: &Address,
) -> Result<(), TracePackError> {
    if address.is_valid() {
        Ok(())
    } else {
        Err(TracePackError::InvalidAddress {
            block_number,
            tx_index: tx_index.0,
            location: location.to_owned(),
            value: address.0.clone(),
        })
    }
}

fn is_prefixed_hex(value: &str, hex_len: usize) -> bool {
    let Some(hex) = value.strip_prefix("0x") else {
        return false;
    };
    hex.len() == hex_len && hex.as_bytes().iter().all(u8::is_ascii_hexdigit)
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn valid_trace_pack_loads() {
        let dir = tempdir().unwrap();
        valid_pack().write_dir(dir.path()).unwrap();

        let loaded = TracePack::load_dir(dir.path()).unwrap();
        assert_eq!(loaded.manifest.schema_version, TRACE_PACK_SCHEMA_VERSION);
        assert_eq!(loaded.blocks.len(), 1);
        assert!(loaded.has_complete_gas());
    }

    #[test]
    fn invalid_manifest_is_rejected() {
        let mut pack = valid_pack();
        pack.manifest.schema_version = "old".to_owned();

        let err = pack.validate().unwrap_err().to_string();
        assert!(err.contains("unsupported schema_version"));
    }

    #[test]
    fn invalid_tx_hash_has_block_and_tx_context() {
        let mut pack = valid_pack();
        pack.blocks[0].transactions[0].tx_hash = TxHash("0x1234".to_owned());

        let err = pack.validate().unwrap_err().to_string();
        assert!(err.contains("block 1 tx_index 0"));
        assert!(err.contains("expected 0x followed by 64 hex"));
    }

    #[test]
    fn invalid_address_has_block_and_tx_context() {
        let mut pack = valid_pack();
        pack.blocks[0].transactions[0].accesses[0].address = Address::canonical("0x1234");

        let err = pack.validate().unwrap_err().to_string();
        assert!(err.contains("block 1 tx_index 0 access 0 address"));
        assert!(err.contains("expected 0x followed by 40 hex"));
    }

    #[test]
    fn missing_optional_gas_is_allowed() {
        let mut pack = valid_pack();
        pack.blocks[0].total_gas_used = None;
        pack.blocks[0].transactions[0].gas_used = None;
        pack.normalize();
        pack.validate().unwrap();

        assert!(!pack.has_complete_gas());
    }

    #[test]
    fn duplicate_accesses_are_normalized_deterministically() {
        let mut pack = valid_pack();
        let duplicate = pack.blocks[0].transactions[0].accesses[0].clone();
        pack.blocks[0].transactions[0].accesses.push(duplicate);
        pack.normalize();
        pack.validate().unwrap();

        assert_eq!(pack.blocks[0].transactions[0].accesses.len(), 1);
    }

    #[test]
    fn complete_gas_total_must_match_tx_sum() {
        let mut pack = valid_pack();
        pack.blocks[0].total_gas_used = Some(21_001);

        let err = pack.validate().unwrap_err().to_string();
        assert!(err.contains("total_gas_used 21001"));
        assert!(err.contains("sum of transaction gas_used values 21000"));
    }

    #[test]
    fn parent_hash_must_match_previous_block_hash_when_present() {
        let mut pack = two_block_pack();
        pack.blocks[1].parent_hash = Some(format!("0x{:064x}", 99));

        let err = pack.validate().unwrap_err().to_string();
        assert!(err.contains("parent_hash"));
        assert!(err.contains("previous block 1 hash"));
    }

    fn valid_pack() -> TracePack {
        TracePack {
            manifest: TracePackManifest {
                schema_version: TRACE_PACK_SCHEMA_VERSION.to_owned(),
                chain: ChainKind::new("base"),
                source: "committed-demo-fixture".to_owned(),
                provenance: "synthetic/demo fixture".to_owned(),
                start_block: 1,
                end_block: 1,
                created_by_tool_version: "test".to_owned(),
                tracer_kind: "unit-test".to_owned(),
                notes: Vec::new(),
                warnings: Vec::new(),
            },
            blocks: vec![TracePackBlock {
                chain: ChainKind::new("base"),
                block_number: 1,
                block_hash: Some(format!("0x{:064x}", 1)),
                parent_hash: Some(format!("0x{:064x}", 0)),
                tx_count: 1,
                total_gas_used: Some(21_000),
                transactions: vec![TracePackTx {
                    tx_index: TxIndex(0),
                    tx_hash: TxHash(format!("0x{:064x}", 10)),
                    from: Some(Address::canonical(
                        "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                    )),
                    to: Some(Address::canonical(
                        "0x1111111111111111111111111111111111111111",
                    )),
                    gas_used: Some(21_000),
                    status: Some("0x1".to_owned()),
                    accesses: vec![TracePackAccess {
                        address: Address::canonical("0x1111111111111111111111111111111111111111"),
                        slot: Some(StorageKey::canonical(format!("0x{:064x}", 1))),
                        kind: TracePackAccessKind::ReadWrite,
                        op: Some("SSTORE".to_owned()),
                        pc: Some(7),
                        depth: Some(1),
                        gas_remaining: Some(50_000),
                    }],
                    warnings: Vec::new(),
                }],
                warnings: Vec::new(),
            }],
        }
    }

    fn two_block_pack() -> TracePack {
        let mut pack = valid_pack();
        pack.manifest.end_block = 2;
        let mut second = pack.blocks[0].clone();
        second.block_number = 2;
        second.block_hash = Some(format!("0x{:064x}", 2));
        second.parent_hash = Some(format!("0x{:064x}", 1));
        second.transactions[0].tx_hash = TxHash(format!("0x{:064x}", 11));
        pack.blocks.push(second);
        pack
    }
}
