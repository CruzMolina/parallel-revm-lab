use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};

pub type Balance = i128;
pub type StorageValue = i128;
pub type ReadSet = BTreeSet<AccessKey>;
pub type WriteSet = BTreeSet<AccessKey>;

#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
pub struct AccountId(pub u64);

#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
pub struct ContractId(pub u64);

#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
pub struct SlotId(pub u64);

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AccessKey {
    AccountBalance { account: AccountId },
    AccountNonce { account: AccountId },
    StorageSlot { contract: ContractId, slot: SlotId },
}

impl AccessKey {
    pub fn account_balance(account: AccountId) -> Self {
        Self::AccountBalance { account }
    }

    pub fn account_nonce(account: AccountId) -> Self {
        Self::AccountNonce { account }
    }

    pub fn storage(contract: ContractId, slot: SlotId) -> Self {
        Self::StorageSlot { contract, slot }
    }
}

impl fmt::Display for AccessKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AccessKey::AccountBalance { account } => write!(f, "acct:{}:balance", account.0),
            AccessKey::AccountNonce { account } => write!(f, "acct:{}:nonce", account.0),
            AccessKey::StorageSlot { contract, slot } => {
                write!(f, "contract:{}:slot:{}", contract.0, slot.0)
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Account {
    pub balance: Balance,
    pub nonce: u64,
}

impl Account {
    pub fn new(balance: Balance) -> Self {
        Self { balance, nonce: 0 }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct State {
    accounts: BTreeMap<AccountId, Account>,
    storage: BTreeMap<(ContractId, SlotId), StorageValue>,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_accounts_and_storage(
        accounts: u64,
        contracts: u64,
        slots_per_contract: u64,
    ) -> Self {
        let mut state = Self::new();
        for id in 0..accounts {
            state.accounts.insert(
                AccountId(id),
                Account::new(1_000_000_i128.saturating_add(i128::from(id))),
            );
        }
        for contract in 0..contracts {
            for slot in 0..slots_per_contract {
                state.storage.insert(
                    (ContractId(contract), SlotId(slot)),
                    1_000_000_i128
                        .saturating_add(i128::from(contract) * 1_000)
                        .saturating_add(i128::from(slot)),
                );
            }
        }
        state
    }

    pub fn accounts(&self) -> &BTreeMap<AccountId, Account> {
        &self.accounts
    }

    pub fn storage(&self) -> &BTreeMap<(ContractId, SlotId), StorageValue> {
        &self.storage
    }

    pub fn read(&self, key: &AccessKey) -> i128 {
        match key {
            AccessKey::AccountBalance { account } => self
                .accounts
                .get(account)
                .map(|acct| acct.balance)
                .unwrap_or_default(),
            AccessKey::AccountNonce { account } => self
                .accounts
                .get(account)
                .map(|acct| i128::from(acct.nonce))
                .unwrap_or_default(),
            AccessKey::StorageSlot { contract, slot } => self
                .storage
                .get(&(*contract, *slot))
                .copied()
                .unwrap_or_default(),
        }
    }

    pub fn write(&mut self, key: &AccessKey, value: i128) {
        match key {
            AccessKey::AccountBalance { account } => {
                self.accounts
                    .entry(*account)
                    .or_insert_with(|| Account::new(0))
                    .balance = value;
            }
            AccessKey::AccountNonce { account } => {
                self.accounts
                    .entry(*account)
                    .or_insert_with(|| Account::new(0))
                    .nonce = value.max(0) as u64;
            }
            AccessKey::StorageSlot { contract, slot } => {
                self.storage.insert((*contract, *slot), value);
            }
        }
    }

    pub fn apply_delta(&mut self, delta: &TxDelta) {
        for (key, value) in &delta.writes {
            self.write(key, *value);
        }
    }

    pub fn state_hash(&self) -> StateHash {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"parallel-revm-lab-state-v1");
        push_u64(&mut bytes, self.accounts.len() as u64);
        for (id, account) in &self.accounts {
            bytes.push(b'a');
            push_u64(&mut bytes, id.0);
            push_i128(&mut bytes, account.balance);
            push_u64(&mut bytes, account.nonce);
        }
        push_u64(&mut bytes, self.storage.len() as u64);
        for ((contract, slot), value) in &self.storage {
            bytes.push(b's');
            push_u64(&mut bytes, contract.0);
            push_u64(&mut bytes, slot.0);
            push_i128(&mut bytes, *value);
        }
        StateHash(stable_fnv1a64(&bytes))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct StateHash(pub u64);

impl fmt::Display for StateHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TxKind {
    Transfer {
        from: AccountId,
        to: AccountId,
        amount: Balance,
    },
    StorageAdd {
        contract: ContractId,
        slot: SlotId,
        delta: StorageValue,
    },
    StorageSet {
        contract: ContractId,
        slot: SlotId,
        value: StorageValue,
        read_prior: bool,
    },
    SwapLike {
        trader: AccountId,
        pool: ContractId,
        amount_in: Balance,
        min_out: Balance,
    },
    HotPool {
        account: AccountId,
        pool: ContractId,
        amount: Balance,
    },
    Noop,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Tx {
    pub id: u64,
    pub kind: TxKind,
    pub declared_reads: ReadSet,
    pub declared_writes: WriteSet,
}

impl Tx {
    pub fn new(id: u64, kind: TxKind) -> Self {
        let (declared_reads, declared_writes) = access_sets_for_kind(&kind);
        Self {
            id,
            kind,
            declared_reads,
            declared_writes,
        }
    }

    pub fn conflicts_with(&self, other: &Self) -> bool {
        access_sets_conflict(
            &self.declared_reads,
            &self.declared_writes,
            &other.declared_reads,
            &other.declared_writes,
        )
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TxDelta {
    pub writes: BTreeMap<AccessKey, i128>,
}

impl TxDelta {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: AccessKey, value: i128) {
        self.writes.insert(key, value);
    }

    pub fn is_empty(&self) -> bool {
        self.writes.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Applied,
    Noop,
    InsufficientBalance,
    SlippageExceeded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionOutcome {
    pub tx_id: u64,
    pub read_values: BTreeMap<AccessKey, i128>,
    pub read_set: ReadSet,
    pub write_set: WriteSet,
    pub delta: TxDelta,
    pub status: ExecutionStatus,
}

impl ExecutionOutcome {
    pub fn reads_match(&self, state: &State) -> bool {
        self.read_values
            .iter()
            .all(|(key, observed)| state.read(key) == *observed)
    }
}

pub fn execute_tx(state: &State, tx: &Tx) -> ExecutionOutcome {
    let mut reads = BTreeMap::new();
    let mut delta = TxDelta::new();
    let mut read_key = |key: AccessKey| {
        let value = state.read(&key);
        reads.insert(key, value);
        value
    };

    let status = match &tx.kind {
        TxKind::Transfer { from, to, amount } => {
            let amount = (*amount).max(0);
            let from_balance_key = AccessKey::account_balance(*from);
            let to_balance_key = AccessKey::account_balance(*to);
            let nonce_key = AccessKey::account_nonce(*from);
            let from_balance = read_key(from_balance_key.clone());
            let to_balance = read_key(to_balance_key.clone());
            let nonce = read_key(nonce_key.clone());

            delta.insert(nonce_key, nonce.saturating_add(1));
            if amount == 0 || from == to {
                ExecutionStatus::Noop
            } else if from_balance >= amount {
                delta.insert(from_balance_key, from_balance.saturating_sub(amount));
                delta.insert(to_balance_key, to_balance.saturating_add(amount));
                ExecutionStatus::Applied
            } else {
                ExecutionStatus::InsufficientBalance
            }
        }
        TxKind::StorageAdd {
            contract,
            slot,
            delta: addend,
        } => {
            let key = AccessKey::storage(*contract, *slot);
            let prior = read_key(key.clone());
            delta.insert(key, prior.saturating_add(*addend));
            ExecutionStatus::Applied
        }
        TxKind::StorageSet {
            contract,
            slot,
            value,
            read_prior,
        } => {
            let key = AccessKey::storage(*contract, *slot);
            if *read_prior {
                let _ = read_key(key.clone());
            }
            delta.insert(key, *value);
            ExecutionStatus::Applied
        }
        TxKind::SwapLike {
            trader,
            pool,
            amount_in,
            min_out,
        } => {
            let amount_in = (*amount_in).max(0);
            let reserve_in_key = AccessKey::storage(*pool, SlotId(0));
            let reserve_out_key = AccessKey::storage(*pool, SlotId(1));
            let trader_balance_key = AccessKey::account_balance(*trader);
            let reserve_in = read_key(reserve_in_key.clone()).max(0);
            let reserve_out = read_key(reserve_out_key.clone()).max(0);
            let trader_balance = read_key(trader_balance_key.clone());

            if amount_in == 0 || trader_balance < amount_in || reserve_out == 0 {
                ExecutionStatus::InsufficientBalance
            } else {
                let denominator = reserve_in.saturating_add(amount_in).saturating_add(1);
                let quoted_out = amount_in.saturating_mul(reserve_out) / denominator;
                let amount_out = quoted_out.min(reserve_out);
                if amount_out < (*min_out).max(0) {
                    ExecutionStatus::SlippageExceeded
                } else {
                    delta.insert(reserve_in_key, reserve_in.saturating_add(amount_in));
                    delta.insert(reserve_out_key, reserve_out.saturating_sub(amount_out));
                    delta.insert(
                        trader_balance_key,
                        trader_balance
                            .saturating_sub(amount_in)
                            .saturating_add(amount_out),
                    );
                    ExecutionStatus::Applied
                }
            }
        }
        TxKind::HotPool {
            account,
            pool,
            amount,
        } => {
            let amount = (*amount).max(0);
            let pool_key = AccessKey::storage(*pool, SlotId(0));
            let balance_key = AccessKey::account_balance(*account);
            let pool_value = read_key(pool_key.clone());
            let balance = read_key(balance_key.clone());
            let deposited = amount.min(balance.max(0));
            if deposited == 0 {
                ExecutionStatus::InsufficientBalance
            } else {
                delta.insert(pool_key, pool_value.saturating_add(deposited));
                delta.insert(balance_key, balance.saturating_sub(deposited));
                ExecutionStatus::Applied
            }
        }
        TxKind::Noop => ExecutionStatus::Noop,
    };

    ExecutionOutcome {
        tx_id: tx.id,
        read_values: reads,
        read_set: tx.declared_reads.clone(),
        write_set: tx.declared_writes.clone(),
        delta,
        status,
    }
}

pub fn access_sets_for_kind(kind: &TxKind) -> (ReadSet, WriteSet) {
    let mut reads = BTreeSet::new();
    let mut writes = BTreeSet::new();
    match kind {
        TxKind::Transfer { from, to, .. } => {
            reads.insert(AccessKey::account_balance(*from));
            reads.insert(AccessKey::account_nonce(*from));
            reads.insert(AccessKey::account_balance(*to));
            writes.insert(AccessKey::account_balance(*from));
            writes.insert(AccessKey::account_nonce(*from));
            writes.insert(AccessKey::account_balance(*to));
        }
        TxKind::StorageAdd { contract, slot, .. } => {
            let key = AccessKey::storage(*contract, *slot);
            reads.insert(key.clone());
            writes.insert(key);
        }
        TxKind::StorageSet {
            contract,
            slot,
            read_prior,
            ..
        } => {
            let key = AccessKey::storage(*contract, *slot);
            if *read_prior {
                reads.insert(key.clone());
            }
            writes.insert(key);
        }
        TxKind::SwapLike { trader, pool, .. } => {
            for key in [
                AccessKey::storage(*pool, SlotId(0)),
                AccessKey::storage(*pool, SlotId(1)),
                AccessKey::account_balance(*trader),
            ] {
                reads.insert(key.clone());
                writes.insert(key);
            }
        }
        TxKind::HotPool { account, pool, .. } => {
            for key in [
                AccessKey::storage(*pool, SlotId(0)),
                AccessKey::account_balance(*account),
            ] {
                reads.insert(key.clone());
                writes.insert(key);
            }
        }
        TxKind::Noop => {}
    }
    (reads, writes)
}

pub fn access_sets_conflict(
    left_reads: &ReadSet,
    left_writes: &WriteSet,
    right_reads: &ReadSet,
    right_writes: &WriteSet,
) -> bool {
    left_writes
        .iter()
        .any(|key| right_writes.contains(key) || right_reads.contains(key))
        || right_writes.iter().any(|key| left_reads.contains(key))
}

fn push_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

fn push_i128(bytes: &mut Vec<u8>, value: i128) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

fn stable_fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_read_write_apply_delta() {
        let mut state = State::new();
        let balance = AccessKey::account_balance(AccountId(7));
        let storage = AccessKey::storage(ContractId(1), SlotId(2));
        state.write(&balance, 100);
        state.write(&storage, 12);

        let mut delta = TxDelta::new();
        delta.insert(balance.clone(), 90);
        delta.insert(storage.clone(), 18);
        state.apply_delta(&delta);

        assert_eq!(state.read(&balance), 90);
        assert_eq!(state.read(&storage), 18);
    }

    #[test]
    fn same_state_has_same_stable_hash() {
        let left = State::with_accounts_and_storage(8, 3, 4);
        let right = State::with_accounts_and_storage(8, 3, 4);
        assert_eq!(left.state_hash(), right.state_hash());
    }

    #[test]
    fn state_hash_changes_after_delta() {
        let mut state = State::with_accounts_and_storage(2, 1, 1);
        let before = state.state_hash();
        state.write(&AccessKey::account_balance(AccountId(0)), 123);
        assert_ne!(before, state.state_hash());
    }

    #[test]
    fn conflict_detection_covers_read_write_and_write_write() {
        let transfer = Tx::new(
            0,
            TxKind::Transfer {
                from: AccountId(0),
                to: AccountId(1),
                amount: 1,
            },
        );
        let storage = Tx::new(
            1,
            TxKind::StorageAdd {
                contract: ContractId(0),
                slot: SlotId(0),
                delta: 1,
            },
        );
        let hot = Tx::new(
            2,
            TxKind::HotPool {
                account: AccountId(1),
                pool: ContractId(0),
                amount: 3,
            },
        );

        assert!(!transfer.conflicts_with(&storage));
        assert!(transfer.conflicts_with(&hot));
        assert!(storage.conflicts_with(&hot));
    }

    #[test]
    fn insufficient_balance_is_deterministic_and_increments_nonce() {
        let state = State::with_accounts_and_storage(2, 0, 0);
        let tx = Tx::new(
            0,
            TxKind::Transfer {
                from: AccountId(0),
                to: AccountId(1),
                amount: 2_000_000,
            },
        );

        let left = execute_tx(&state, &tx);
        let right = execute_tx(&state, &tx);
        assert_eq!(left, right);
        assert_eq!(left.status, ExecutionStatus::InsufficientBalance);
        assert!(left
            .delta
            .writes
            .contains_key(&AccessKey::account_nonce(AccountId(0))));
    }
}
