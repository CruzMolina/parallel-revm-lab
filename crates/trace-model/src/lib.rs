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

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Address(pub String);

impl Address {
    pub fn canonical(value: impl AsRef<str>) -> Self {
        Self(value.as_ref().trim().to_ascii_lowercase())
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
}

fn default_true() -> bool {
    true
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
        let tx = TxTrace {
            tx_index: TxIndex(0),
            tx_hash: TxHash("0x1".to_owned()),
            from: Address::canonical("0xaaaa"),
            to: None,
            read_info_complete: true,
            accesses: Vec::new(),
            warnings: Vec::new(),
        };
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
}
