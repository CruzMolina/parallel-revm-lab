use parallel_revm_lab_analyzer::{analyze_block_trace, ParallelismReport};
use parallel_revm_lab_trace_model::{
    Address as TraceAddress, BlockAccessTrace, BlockRef, ChainKind, StorageKey, TraceAccess,
    TraceAccessKey, TraceAccessKind, TraceParseWarning, TxHash, TxIndex, TxTrace,
};
use revm::{
    context::TxEnv,
    database::{CacheDB, EmptyDB},
    primitives::{Address, Bytes, TxKind, U256},
    state::{AccountInfo, Bytecode},
    Context, ExecuteEvm, MainBuilder, MainContext,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BytecodeFixture {
    CounterSlot { slot: u8 },
    WriteSlot { slot: u8 },
    HotSlot { slot: u8 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RevmSmokeTx {
    pub tx_index: u64,
    pub caller: Address,
    pub contract: Address,
    pub fixture: BytecodeFixture,
}

pub fn smoke_report(txs: &[RevmSmokeTx]) -> Result<ParallelismReport, RevmSmokeError> {
    Ok(analyze_block_trace(&smoke_trace(txs)?, false, false))
}

pub fn smoke_trace(txs: &[RevmSmokeTx]) -> Result<BlockAccessTrace, RevmSmokeError> {
    let mut traces = Vec::with_capacity(txs.len());
    for tx in txs {
        execute_fixture(tx)?;
        traces.push(tx_trace(tx));
    }

    let mut trace = BlockAccessTrace {
        chain: ChainKind::new("revm-smoke"),
        block: BlockRef {
            number: 0,
            hash: None,
        },
        transactions: traces,
        warnings: vec![TraceParseWarning {
            tx_index: None,
            message: "revm smoke derives storage observations from known bytecode fixtures; it is not a general EVM tracer".to_owned(),
        }],
    };
    trace.normalize();
    trace.validate()?;
    Ok(trace)
}

fn execute_fixture(tx: &RevmSmokeTx) -> Result<(), RevmSmokeError> {
    let mut db = CacheDB::<EmptyDB>::default();
    db.insert_account_info(
        tx.caller,
        AccountInfo {
            balance: U256::from(1_000_000_000_000_000_u64),
            ..AccountInfo::default()
        },
    );
    db.insert_account_info(
        tx.contract,
        AccountInfo::default().with_code(Bytecode::new_raw(bytecode(tx.fixture))),
    );
    db.insert_account_storage(tx.contract, U256::from(slot(tx.fixture)), U256::ZERO)
        .map_err(|_| RevmSmokeError::StorageSeed)?;

    let ctx = Context::mainnet().with_db(db);
    let mut evm = ctx.build_mainnet();
    let tx_env = TxEnv::builder()
        .caller(tx.caller)
        .kind(TxKind::Call(tx.contract))
        .gas_limit(100_000)
        .build()
        .map_err(|err| RevmSmokeError::BuildTx(err.to_string()))?;
    let result = evm
        .transact(tx_env)
        .map_err(|err| RevmSmokeError::Execution(err.to_string()))?;
    if result.result.is_success() {
        Ok(())
    } else {
        Err(RevmSmokeError::Unsuccessful(format!("{:?}", result.result)))
    }
}

fn tx_trace(tx: &RevmSmokeTx) -> TxTrace {
    let access_kind = match tx.fixture {
        BytecodeFixture::CounterSlot { .. } | BytecodeFixture::HotSlot { .. } => {
            TraceAccessKind::ReadWrite
        }
        BytecodeFixture::WriteSlot { .. } => TraceAccessKind::Write,
    };
    TxTrace {
        tx_index: TxIndex(tx.tx_index),
        tx_hash: TxHash(format!("0x{:064x}", tx.tx_index)),
        from: trace_address(tx.caller),
        to: Some(trace_address(tx.contract)),
        read_info_complete: matches!(
            tx.fixture,
            BytecodeFixture::CounterSlot { .. } | BytecodeFixture::HotSlot { .. }
        ),
        accesses: vec![TraceAccess {
            kind: access_kind,
            key: TraceAccessKey::Storage {
                address: trace_address(tx.contract),
                slot: StorageKey::canonical(format!("0x{:064x}", slot(tx.fixture))),
            },
        }],
        warnings: Vec::new(),
    }
}

fn bytecode(fixture: BytecodeFixture) -> Bytes {
    match fixture {
        BytecodeFixture::CounterSlot { slot } | BytecodeFixture::HotSlot { slot } => {
            Bytes::from(vec![
                0x60, slot, 0x54, 0x60, 0x01, 0x01, 0x60, slot, 0x55, 0x00,
            ])
        }
        BytecodeFixture::WriteSlot { slot } => {
            Bytes::from(vec![0x60, 0x01, 0x60, slot, 0x55, 0x00])
        }
    }
}

fn slot(fixture: BytecodeFixture) -> u8 {
    match fixture {
        BytecodeFixture::CounterSlot { slot }
        | BytecodeFixture::WriteSlot { slot }
        | BytecodeFixture::HotSlot { slot } => slot,
    }
}

fn trace_address(address: Address) -> TraceAddress {
    TraceAddress::canonical(address.to_string())
}

#[derive(Debug, thiserror::Error)]
pub enum RevmSmokeError {
    #[error("failed to build revm tx env: {0}")]
    BuildTx(String),
    #[error("failed to seed revm storage")]
    StorageSeed,
    #[error("revm execution failed: {0}")]
    Execution(String),
    #[error("revm execution was not successful: {0}")]
    Unsuccessful(String),
    #[error(transparent)]
    TraceModel(#[from] parallel_revm_lab_trace_model::TraceModelError),
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONTRACT: Address = Address::new([0x22; 20]);

    #[test]
    fn revm_counter_txs_conflict_on_same_slot() {
        let txs = vec![
            tx(0, 0x10, BytecodeFixture::CounterSlot { slot: 0 }),
            tx(1, 0x11, BytecodeFixture::CounterSlot { slot: 0 }),
        ];
        let report = smoke_report(&txs).unwrap();

        assert_eq!(report.conflict_pair_count, 1);
        assert_eq!(report.wave_count, 2);
        assert_eq!(report.critical_path_length, 2);
    }

    #[test]
    fn revm_independent_slot_txs_can_share_wave() {
        let txs = vec![
            tx(0, 0x10, BytecodeFixture::WriteSlot { slot: 1 }),
            tx(1, 0x11, BytecodeFixture::WriteSlot { slot: 2 }),
            tx(2, 0x12, BytecodeFixture::WriteSlot { slot: 3 }),
        ];
        let report = smoke_report(&txs).unwrap();

        assert_eq!(report.conflict_pair_count, 0);
        assert_eq!(report.wave_count, 1);
        assert_eq!(report.max_wave_width, 3);
    }

    #[test]
    fn revm_smoke_report_is_deterministic() {
        let txs = vec![
            tx(0, 0x10, BytecodeFixture::HotSlot { slot: 7 }),
            tx(1, 0x11, BytecodeFixture::WriteSlot { slot: 8 }),
            tx(2, 0x12, BytecodeFixture::HotSlot { slot: 7 }),
        ];

        let left = smoke_report(&txs).unwrap();
        let right = smoke_report(&txs).unwrap();
        assert_eq!(left.deterministic_hash, right.deterministic_hash);
        assert_eq!(left.conflict_pair_count, 1);
        assert_eq!(left.wave_count, 2);
    }

    fn tx(tx_index: u64, caller_byte: u8, fixture: BytecodeFixture) -> RevmSmokeTx {
        RevmSmokeTx {
            tx_index,
            caller: Address::new([caller_byte; 20]),
            contract: CONTRACT,
            fixture,
        }
    }
}
