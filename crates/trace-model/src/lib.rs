use std::collections::BTreeSet;
use std::fmt;
use std::fs::File;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct ChainKind(pub String);

impl ChainKind {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl fmt::Display for ChainKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct BlockRef {
    pub number: u64,
    pub hash: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct TxHash(pub String);

impl TxHash {
    pub fn is_valid(&self) -> bool {
        is_prefixed_hex(&self.0, 64)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Address(pub String);

impl Address {
    pub fn canonical(value: impl AsRef<str>) -> Self {
        Self(value.as_ref().trim().to_ascii_lowercase())
    }

    pub fn is_valid(&self) -> bool {
        is_prefixed_hex(&self.0, 40)
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct StorageKey(pub String);

impl StorageKey {
    pub fn canonical(value: impl AsRef<str>) -> Self {
        Self(value.as_ref().trim().to_ascii_lowercase())
    }

    pub fn is_valid(&self) -> bool {
        is_prefixed_hex(&self.0, 64)
    }
}

impl fmt::Display for StorageKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct TxIndex(pub u64);

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TraceAccessKey {
    Account { address: Address },
    Balance { address: Address },
    Nonce { address: Address },
    Code { address: Address },
    Storage { address: Address, slot: StorageKey },
}

impl TraceAccessKey {
    pub fn address(&self) -> &Address {
        match self {
            TraceAccessKey::Account { address }
            | TraceAccessKey::Balance { address }
            | TraceAccessKey::Nonce { address }
            | TraceAccessKey::Code { address }
            | TraceAccessKey::Storage { address, .. } => address,
        }
    }

    pub fn storage_slot(&self) -> Option<(&Address, &StorageKey)> {
        match self {
            TraceAccessKey::Storage { address, slot } => Some((address, slot)),
            _ => None,
        }
    }

    fn canonicalize(&mut self) {
        match self {
            TraceAccessKey::Account { address }
            | TraceAccessKey::Balance { address }
            | TraceAccessKey::Nonce { address }
            | TraceAccessKey::Code { address } => {
                *address = Address::canonical(&address.0);
            }
            TraceAccessKey::Storage { address, slot } => {
                *address = Address::canonical(&address.0);
                *slot = StorageKey::canonical(&slot.0);
            }
        }
    }

    fn validate(&self, tx_index: TxIndex, access_index: usize) -> Result<(), TraceModelError> {
        let location = format!("tx_index {} access {}", tx_index.0, access_index);
        match self {
            TraceAccessKey::Account { address }
            | TraceAccessKey::Balance { address }
            | TraceAccessKey::Nonce { address }
            | TraceAccessKey::Code { address } => validate_address(address, &location),
            TraceAccessKey::Storage { address, slot } => {
                validate_address(address, &format!("{location} storage address"))?;
                if !slot.is_valid() {
                    return Err(TraceModelError::InvalidStorageKey {
                        location: format!("{location} storage slot"),
                        value: slot.0.clone(),
                    });
                }
                Ok(())
            }
        }
    }
}

impl fmt::Display for TraceAccessKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TraceAccessKey::Account { address } => write!(f, "account:{address}"),
            TraceAccessKey::Balance { address } => write!(f, "balance:{address}"),
            TraceAccessKey::Nonce { address } => write!(f, "nonce:{address}"),
            TraceAccessKey::Code { address } => write!(f, "code:{address}"),
            TraceAccessKey::Storage { address, slot } => write!(f, "storage:{address}:{slot}"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceAccessKind {
    Read,
    Write,
    ReadWrite,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct TraceAccess {
    pub kind: TraceAccessKind,
    pub key: TraceAccessKey,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TraceAccessSet {
    pub reads: BTreeSet<TraceAccessKey>,
    pub writes: BTreeSet<TraceAccessKey>,
    pub reads_complete: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct TraceParseWarning {
    pub tx_index: Option<TxIndex>,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TxTrace {
    pub tx_index: TxIndex,
    pub tx_hash: TxHash,
    pub from: Address,
    pub to: Option<Address>,
    #[serde(default = "default_true")]
    pub read_info_complete: bool,
    #[serde(default)]
    pub accesses: Vec<TraceAccess>,
    #[serde(default)]
    pub warnings: Vec<TraceParseWarning>,
}

impl TxTrace {
    pub fn access_set(&self) -> TraceAccessSet {
        let mut set = TraceAccessSet {
            reads_complete: self.read_info_complete,
            ..TraceAccessSet::default()
        };
        for access in &self.accesses {
            match access.kind {
                TraceAccessKind::Read => {
                    set.reads.insert(access.key.clone());
                }
                TraceAccessKind::Write => {
                    set.writes.insert(access.key.clone());
                }
                TraceAccessKind::ReadWrite => {
                    set.reads.insert(access.key.clone());
                    set.writes.insert(access.key.clone());
                }
            }
        }
        set
    }

    fn normalize(&mut self) {
        self.from = Address::canonical(&self.from.0);
        if let Some(to) = &mut self.to {
            *to = Address::canonical(&to.0);
        }
        for access in &mut self.accesses {
            access.key.canonicalize();
        }
        self.accesses.sort();
        self.warnings.sort();
    }

    fn validate(&self) -> Result<(), TraceModelError> {
        if !self.tx_hash.is_valid() {
            return Err(TraceModelError::InvalidTxHash {
                tx_index: self.tx_index.0,
                value: self.tx_hash.0.clone(),
            });
        }
        validate_address(&self.from, &format!("tx_index {} from", self.tx_index.0))?;
        if let Some(to) = &self.to {
            validate_address(to, &format!("tx_index {} to", self.tx_index.0))?;
        }
        for (access_index, access) in self.accesses.iter().enumerate() {
            access.key.validate(self.tx_index, access_index)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BlockAccessTrace {
    pub chain: ChainKind,
    pub block: BlockRef,
    #[serde(default)]
    pub transactions: Vec<TxTrace>,
    #[serde(default)]
    pub warnings: Vec<TraceParseWarning>,
}

impl BlockAccessTrace {
    pub fn from_fixture_path(path: &Path) -> Result<Self, TraceModelError> {
        let file = File::open(path)?;
        let mut trace: Self = serde_json::from_reader(file)?;
        trace.normalize();
        trace.validate()?;
        Ok(trace)
    }

    pub fn from_geth_struct_logs_path(path: &Path) -> Result<Self, TraceModelError> {
        let file = File::open(path)?;
        let fixture: GethStructLogsFixture = serde_json::from_reader(file)?;
        let mut trace = fixture.into_block_trace();
        trace.normalize();
        trace.validate()?;
        Ok(trace)
    }

    pub fn normalize(&mut self) {
        for tx in &mut self.transactions {
            tx.normalize();
            if !tx.read_info_complete {
                tx.warnings.push(TraceParseWarning {
                    tx_index: Some(tx.tx_index),
                    message: "trace marks read information as incomplete".to_owned(),
                });
            }
            tx.warnings.sort();
            tx.warnings.dedup();
        }
        self.transactions.sort_by_key(|tx| tx.tx_index);
        self.warnings.sort();
        self.warnings.dedup();
    }

    pub fn validate(&self) -> Result<(), TraceModelError> {
        let mut seen = BTreeSet::new();
        for tx in &self.transactions {
            if !seen.insert(tx.tx_index) {
                return Err(TraceModelError::DuplicateTxIndex(tx.tx_index.0));
            }
        }
        for tx in &self.transactions {
            tx.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TraceModelError {
    #[error("failed to read fixture: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse fixture JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("fixture contains duplicate tx_index {0}")]
    DuplicateTxIndex(u64),
    #[error("invalid address at {location}: `{value}`; expected 0x followed by 40 hex characters")]
    InvalidAddress { location: String, value: String },
    #[error(
        "invalid storage key at {location}: `{value}`; expected 0x followed by 64 hex characters"
    )]
    InvalidStorageKey { location: String, value: String },
    #[error("invalid tx hash at tx_index {tx_index}: `{value}`; expected 0x followed by 64 hex characters")]
    InvalidTxHash { tx_index: u64, value: String },
}

fn default_true() -> bool {
    true
}

fn validate_address(address: &Address, location: &str) -> Result<(), TraceModelError> {
    if address.is_valid() {
        Ok(())
    } else {
        Err(TraceModelError::InvalidAddress {
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

#[derive(Debug, Deserialize)]
struct GethStructLogsFixture {
    chain: ChainKind,
    block: BlockRef,
    #[serde(default)]
    transactions: Vec<GethStructLogsTx>,
    #[serde(default)]
    warnings: Vec<TraceParseWarning>,
}

impl GethStructLogsFixture {
    fn into_block_trace(self) -> BlockAccessTrace {
        let mut warnings = self.warnings;
        warnings.push(TraceParseWarning {
            tx_index: None,
            message: "geth struct-log parser captures SLOAD/SSTORE storage accesses only; account, balance, nonce, and code reads are not represented".to_owned(),
        });
        BlockAccessTrace {
            chain: self.chain,
            block: self.block,
            transactions: self
                .transactions
                .into_iter()
                .map(GethStructLogsTx::into_tx_trace)
                .collect(),
            warnings,
        }
    }
}

#[derive(Debug, Deserialize)]
struct GethStructLogsTx {
    tx_index: TxIndex,
    tx_hash: TxHash,
    from: Address,
    to: Address,
    #[serde(rename = "structLogs", default)]
    struct_logs: Vec<GethStructLog>,
}

impl GethStructLogsTx {
    fn into_tx_trace(self) -> TxTrace {
        let mut accesses = Vec::new();
        let mut warnings = Vec::new();
        for (log_index, log) in self.struct_logs.iter().enumerate() {
            let op = log.op.to_ascii_uppercase();
            match op.as_str() {
                "SLOAD" => match stack_slot(log) {
                    Some(slot) => accesses.push(TraceAccess {
                        kind: TraceAccessKind::Read,
                        key: TraceAccessKey::Storage {
                            address: self.to.clone(),
                            slot,
                        },
                    }),
                    None => warnings.push(struct_log_warning(
                        self.tx_index,
                        log_index,
                        "SLOAD missing a valid storage slot on stack",
                    )),
                },
                "SSTORE" => match stack_slot(log) {
                    Some(slot) => accesses.push(TraceAccess {
                        kind: TraceAccessKind::Write,
                        key: TraceAccessKey::Storage {
                            address: self.to.clone(),
                            slot,
                        },
                    }),
                    None => warnings.push(struct_log_warning(
                        self.tx_index,
                        log_index,
                        "SSTORE missing a valid storage slot on stack",
                    )),
                },
                _ => {}
            }
        }
        warnings.push(TraceParseWarning {
            tx_index: Some(self.tx_index),
            message: "geth struct-log storage parser marks reads incomplete outside SLOAD"
                .to_owned(),
        });
        TxTrace {
            tx_index: self.tx_index,
            tx_hash: self.tx_hash,
            from: self.from,
            to: Some(self.to),
            read_info_complete: false,
            accesses,
            warnings,
        }
    }
}

#[derive(Debug, Deserialize)]
struct GethStructLog {
    op: String,
    #[serde(default)]
    stack: Vec<String>,
}

fn stack_slot(log: &GethStructLog) -> Option<StorageKey> {
    log.stack
        .last()
        .and_then(|word| storage_key_from_word(word))
}

fn storage_key_from_word(value: &str) -> Option<StorageKey> {
    let raw = value.trim();
    let hex = raw.strip_prefix("0x").unwrap_or(raw);
    if hex.len() > 64 || !hex.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        return None;
    }
    Some(StorageKey::canonical(format!("0x{hex:0>64}")))
}

fn struct_log_warning(tx_index: TxIndex, log_index: usize, message: &str) -> TraceParseWarning {
    TraceParseWarning {
        tx_index: Some(tx_index),
        message: format!("structLogs[{log_index}]: {message}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn access_set_splits_read_write() {
        let tx = TxTrace {
            tx_index: TxIndex(0),
            tx_hash: TxHash("0x1".to_owned()),
            from: Address::canonical("0xaaaa"),
            to: None,
            read_info_complete: true,
            accesses: vec![TraceAccess {
                kind: TraceAccessKind::ReadWrite,
                key: TraceAccessKey::Storage {
                    address: Address::canonical("0xBEEF"),
                    slot: StorageKey::canonical("0x00"),
                },
            }],
            warnings: Vec::new(),
        };
        let set = tx.access_set();
        assert_eq!(set.reads.len(), 1);
        assert_eq!(set.writes.len(), 1);
    }

    #[test]
    fn duplicate_tx_indices_are_rejected() {
        let tx = valid_tx(0);
        let mut trace = BlockAccessTrace {
            chain: ChainKind::new("fixture"),
            block: BlockRef {
                number: 1,
                hash: None,
            },
            transactions: vec![tx.clone(), tx],
            warnings: Vec::new(),
        };
        trace.normalize();

        assert!(matches!(
            trace.validate(),
            Err(TraceModelError::DuplicateTxIndex(0))
        ));
    }

    #[test]
    fn invalid_address_is_rejected_with_tx_context() {
        let mut trace = valid_trace(vec![valid_tx(7)]);
        trace.transactions[0].from = Address::canonical("0xnot-hex");

        let err = trace.validate().unwrap_err().to_string();
        assert!(err.contains("tx_index 7 from"));
        assert!(err.contains("expected 0x followed by 40 hex"));
    }

    #[test]
    fn invalid_access_address_is_rejected_with_tx_context() {
        let mut tx = valid_tx(7);
        tx.accesses[0].key = TraceAccessKey::Storage {
            address: Address::canonical("0x1234"),
            slot: StorageKey::canonical(
                "0x0000000000000000000000000000000000000000000000000000000000000001",
            ),
        };
        let trace = valid_trace(vec![tx]);

        let err = trace.validate().unwrap_err().to_string();
        assert!(err.contains("tx_index 7 access 0 storage address"));
        assert!(err.contains("expected 0x followed by 40 hex"));
    }

    #[test]
    fn invalid_storage_key_is_rejected_with_tx_context() {
        let mut tx = valid_tx(8);
        tx.accesses[0].key = TraceAccessKey::Storage {
            address: Address::canonical("0x1111111111111111111111111111111111111111"),
            slot: StorageKey::canonical("0x01"),
        };
        let trace = valid_trace(vec![tx]);

        let err = trace.validate().unwrap_err().to_string();
        assert!(err.contains("tx_index 8 access 0 storage slot"));
        assert!(err.contains("expected 0x followed by 64 hex"));
    }

    #[test]
    fn invalid_tx_hash_is_rejected_with_tx_context() {
        let mut trace = valid_trace(vec![valid_tx(9)]);
        trace.transactions[0].tx_hash = TxHash("0x1234".to_owned());

        let err = trace.validate().unwrap_err().to_string();
        assert!(err.contains("tx_index 9"));
        assert!(err.contains("expected 0x followed by 64 hex"));
    }

    #[test]
    fn committed_fixture_validates() {
        let mut trace: BlockAccessTrace =
            serde_json::from_str(include_str!("../../../fixtures/base-mini-trace.json")).unwrap();
        trace.normalize();
        trace.validate().unwrap();
    }

    #[test]
    fn geth_struct_logs_fixture_parses_storage_accesses() {
        let fixture: GethStructLogsFixture =
            serde_json::from_str(include_str!("../../../fixtures/geth-mini-struct-logs.json"))
                .unwrap();
        let mut trace = fixture.into_block_trace();
        trace.normalize();
        trace.validate().unwrap();

        assert_eq!(trace.transactions.len(), 3);
        assert_eq!(trace.transactions[0].accesses.len(), 2);
        assert!(!trace.transactions[0].read_info_complete);
        assert!(trace
            .warnings
            .iter()
            .any(|warning| warning.message.contains("SLOAD/SSTORE")));
    }

    fn valid_trace(transactions: Vec<TxTrace>) -> BlockAccessTrace {
        BlockAccessTrace {
            chain: ChainKind::new("fixture"),
            block: BlockRef {
                number: 1,
                hash: None,
            },
            transactions,
            warnings: Vec::new(),
        }
    }

    fn valid_tx(tx_index: u64) -> TxTrace {
        TxTrace {
            tx_index: TxIndex(tx_index),
            tx_hash: TxHash(format!("0x{tx_index:064x}")),
            from: Address::canonical("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            to: Some(Address::canonical(
                "0x1111111111111111111111111111111111111111",
            )),
            read_info_complete: true,
            accesses: vec![TraceAccess {
                kind: TraceAccessKind::ReadWrite,
                key: TraceAccessKey::Storage {
                    address: Address::canonical("0x1111111111111111111111111111111111111111"),
                    slot: StorageKey::canonical(
                        "0x0000000000000000000000000000000000000000000000000000000000000001",
                    ),
                },
            }],
            warnings: Vec::new(),
        }
    }
}
