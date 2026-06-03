use std::fmt;
use std::str::FromStr;

use parallel_revm_lab_model::{AccountId, ContractId, SlotId, State, Tx, TxKind};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkloadKind {
    Erc20,
    Storage,
    SwapLike,
    HotPool,
    Mixed,
}

impl fmt::Display for WorkloadKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkloadKind::Erc20 => write!(f, "erc20"),
            WorkloadKind::Storage => write!(f, "storage"),
            WorkloadKind::SwapLike => write!(f, "swap-like"),
            WorkloadKind::HotPool => write!(f, "hot-pool"),
            WorkloadKind::Mixed => write!(f, "mixed"),
        }
    }
}

impl FromStr for WorkloadKind {
    type Err = WorkloadError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "erc20" => Ok(Self::Erc20),
            "storage" => Ok(Self::Storage),
            "swap-like" | "swap_like" => Ok(Self::SwapLike),
            "hot-pool" | "hot_pool" => Ok(Self::HotPool),
            "mixed" => Ok(Self::Mixed),
            other => Err(WorkloadError::UnknownWorkload(other.to_owned())),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConflictPreset {
    C0,
    C20,
    C50,
    C70,
    C95,
}

impl ConflictPreset {
    pub fn ratio(self) -> f64 {
        match self {
            ConflictPreset::C0 => 0.0,
            ConflictPreset::C20 => 0.2,
            ConflictPreset::C50 => 0.5,
            ConflictPreset::C70 => 0.7,
            ConflictPreset::C95 => 0.95,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkloadConfig {
    pub kind: WorkloadKind,
    pub tx_count: usize,
    pub requested_conflict: f64,
    pub vm_steps: u64,
    pub accounts: u64,
    pub contracts: u64,
    pub hot_slots: u64,
    pub seed: u64,
}

impl WorkloadConfig {
    pub fn new(kind: WorkloadKind, tx_count: usize, requested_conflict: f64, seed: u64) -> Self {
        Self {
            kind,
            tx_count,
            requested_conflict,
            vm_steps: 0,
            accounts: 256,
            contracts: 64,
            hot_slots: 4,
            seed,
        }
    }

    pub fn normalized(&self) -> Self {
        Self {
            kind: self.kind,
            tx_count: self.tx_count,
            requested_conflict: self.requested_conflict.clamp(0.0, 1.0),
            vm_steps: self.vm_steps,
            accounts: self.accounts.max(2),
            contracts: self.contracts.max(1),
            hot_slots: self.hot_slots.max(1),
            seed: self.seed,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Workload {
    pub config: WorkloadConfig,
    pub initial_state: State,
    pub txs: Vec<Tx>,
    pub conflict_pairs: u64,
    pub observed_conflict: f64,
}

#[derive(Debug, thiserror::Error)]
pub enum WorkloadError {
    #[error("unknown workload `{0}`; expected erc20, storage, swap-like, hot-pool, or mixed")]
    UnknownWorkload(String),
}

pub fn generate_workload(config: WorkloadConfig) -> Workload {
    let config = config.normalized();
    let mut rng = ChaCha8Rng::seed_from_u64(config.seed);
    let slots_per_contract = config.hot_slots.max(8).saturating_mul(4);
    let initial_state =
        State::with_accounts_and_storage(config.accounts, config.contracts, slots_per_contract);
    let mut txs = Vec::with_capacity(config.tx_count);

    for id in 0..config.tx_count {
        let tx_kind = match config.kind {
            WorkloadKind::Erc20 => transfer_tx(&config, &mut rng),
            WorkloadKind::Storage => storage_tx(&config, slots_per_contract, &mut rng),
            WorkloadKind::SwapLike => swap_like_tx(&config, &mut rng),
            WorkloadKind::HotPool => hot_pool_tx(&config, &mut rng),
            WorkloadKind::Mixed => match rng.gen_range(0..100) {
                0..=34 => transfer_tx(&config, &mut rng),
                35..=59 => storage_tx(&config, slots_per_contract, &mut rng),
                60..=79 => swap_like_tx(&config, &mut rng),
                80..=94 => hot_pool_tx(&config, &mut rng),
                _ => TxKind::Noop,
            },
        };
        txs.push(Tx::new(id as u64, tx_kind).with_vm_steps(config.vm_steps));
    }

    let conflict_pairs = count_conflict_pairs(&txs);
    let possible_pairs = pair_count(txs.len());
    let observed_conflict = if possible_pairs == 0 {
        0.0
    } else {
        conflict_pairs as f64 / possible_pairs as f64
    };

    Workload {
        config,
        initial_state,
        txs,
        conflict_pairs,
        observed_conflict,
    }
}

pub fn count_conflict_pairs(txs: &[Tx]) -> u64 {
    let mut conflicts = 0_u64;
    for left in 0..txs.len() {
        for right in (left + 1)..txs.len() {
            if txs[left].conflicts_with(&txs[right]) {
                conflicts += 1;
            }
        }
    }
    conflicts
}

fn pair_count(len: usize) -> u64 {
    if len < 2 {
        0
    } else {
        (len as u64 * (len as u64 - 1)) / 2
    }
}

fn transfer_tx(config: &WorkloadConfig, rng: &mut ChaCha8Rng) -> TxKind {
    let hot = choose_hot(config, rng);
    let hot_accounts = config
        .hot_slots
        .saturating_add(1)
        .min(config.accounts)
        .max(2);
    let range = if hot { hot_accounts } else { config.accounts };
    let from = AccountId(rng.gen_range(0..range));
    let mut to = AccountId(rng.gen_range(0..range));
    if to == from {
        to = AccountId((to.0 + 1) % range);
    }
    TxKind::Transfer {
        from,
        to,
        amount: rng.gen_range(1_i128..=1_000),
    }
}

fn storage_tx(config: &WorkloadConfig, slots_per_contract: u64, rng: &mut ChaCha8Rng) -> TxKind {
    let hot = choose_hot(config, rng);
    let contract = if hot {
        ContractId(0)
    } else {
        ContractId(rng.gen_range(0..config.contracts))
    };
    let slot_limit = if hot {
        config.hot_slots
    } else {
        slots_per_contract
    };
    let slot = SlotId(rng.gen_range(0..slot_limit.max(1)));
    if rng.gen_bool(0.65) {
        TxKind::StorageAdd {
            contract,
            slot,
            delta: rng.gen_range(-50_i128..=50).max(1),
        }
    } else {
        TxKind::StorageSet {
            contract,
            slot,
            value: rng.gen_range(1_i128..=2_000_000),
            read_prior: rng.gen_bool(0.5),
        }
    }
}

fn swap_like_tx(config: &WorkloadConfig, rng: &mut ChaCha8Rng) -> TxKind {
    let hot = choose_hot(config, rng);
    let pool = if hot {
        ContractId(0)
    } else {
        ContractId(rng.gen_range(0..config.contracts))
    };
    let trader_range = if hot {
        config
            .hot_slots
            .saturating_add(1)
            .min(config.accounts)
            .max(2)
    } else {
        config.accounts
    };
    TxKind::SwapLike {
        trader: AccountId(rng.gen_range(0..trader_range)),
        pool,
        amount_in: rng.gen_range(1_i128..=750),
        min_out: 0,
    }
}

fn hot_pool_tx(config: &WorkloadConfig, rng: &mut ChaCha8Rng) -> TxKind {
    let account_range = if choose_hot(config, rng) {
        config
            .hot_slots
            .saturating_add(1)
            .min(config.accounts)
            .max(2)
    } else {
        config.accounts
    };
    TxKind::HotPool {
        account: AccountId(rng.gen_range(0..account_range)),
        pool: ContractId(0),
        amount: rng.gen_range(1_i128..=250),
    }
}

fn choose_hot(config: &WorkloadConfig, rng: &mut ChaCha8Rng) -> bool {
    rng.gen_bool(config.requested_conflict.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_generates_same_workload_and_state_hash() {
        let config = WorkloadConfig::new(WorkloadKind::Mixed, 100, 0.5, 42);
        let left = generate_workload(config.clone());
        let right = generate_workload(config);

        assert_eq!(
            left.initial_state.state_hash(),
            right.initial_state.state_hash()
        );
        assert_eq!(left.txs, right.txs);
        assert_eq!(left.conflict_pairs, right.conflict_pairs);
    }

    #[test]
    fn zero_and_one_tx_conflict_rate_is_zero() {
        for tx_count in [0, 1] {
            let workload =
                generate_workload(WorkloadConfig::new(WorkloadKind::Erc20, tx_count, 0.95, 7));
            assert_eq!(workload.conflict_pairs, 0);
            assert_eq!(workload.observed_conflict, 0.0);
        }
    }

    #[test]
    fn conflict_preset_values_are_documented_ratios() {
        assert_eq!(ConflictPreset::C0.ratio(), 0.0);
        assert_eq!(ConflictPreset::C20.ratio(), 0.2);
        assert_eq!(ConflictPreset::C50.ratio(), 0.5);
        assert_eq!(ConflictPreset::C70.ratio(), 0.7);
        assert_eq!(ConflictPreset::C95.ratio(), 0.95);
    }
}
