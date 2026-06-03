use std::collections::{BTreeMap, BTreeSet};

use parallel_revm_lab_trace_model::{
    BlockAccessTrace, TraceAccessKey, TraceAccessKind, TraceParseWarning, TxIndex,
};
use serde::{Deserialize, Serialize};

mod dossier;

pub use dossier::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParallelismReport {
    pub report_version: String,
    pub chain: String,
    pub block: BlockReportRef,
    pub tx_count: usize,
    pub access_model: String,
    pub conflict_pair_count: u64,
    pub conflict_percentage: f64,
    pub wave_count: usize,
    pub max_wave_width: usize,
    pub critical_path_length: usize,
    pub theoretical_parallelism_ceiling: f64,
    pub top_hot_contracts: Vec<HotItem>,
    pub top_hot_storage_slots: Vec<HotItem>,
    pub access_histogram: BTreeMap<String, u64>,
    pub tx_summaries: Vec<TxSummary>,
    pub warnings: Vec<TraceParseWarning>,
    pub generated_from_fixture: bool,
    pub generated_from_rpc: bool,
    pub deterministic_hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockReportRef {
    pub number: u64,
    pub hash: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HotItem {
    pub key: String,
    pub count: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TxSummary {
    pub tx_index: u64,
    pub tx_hash: String,
    pub read_count: usize,
    pub write_count: usize,
    pub conflict_degree: u64,
    pub wave: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConflictPair {
    pub earlier: TxIndex,
    pub later: TxIndex,
    pub keys: Vec<TraceAccessKey>,
}

pub fn analyze_block_trace(
    trace: &BlockAccessTrace,
    generated_from_fixture: bool,
    generated_from_rpc: bool,
) -> ParallelismReport {
    let mut normalized_trace = trace.clone();
    normalized_trace.normalize();
    let trace = &normalized_trace;
    let tx_count = trace.transactions.len();
    let has_incomplete_reads = trace.transactions.iter().any(|tx| !tx.read_info_complete);
    let conflict_pairs = conflict_pairs(trace);
    let dependencies = dependency_graph(&conflict_pairs);
    let waves = build_waves(trace, &dependencies);
    let tx_to_wave = tx_wave_map(&waves);
    let conflict_degrees = conflict_degrees(&conflict_pairs);
    let critical_path_length = critical_path_length(trace, &dependencies);
    let possible_pairs = pair_count(tx_count);
    let conflict_percentage = if possible_pairs == 0 {
        0.0
    } else {
        (conflict_pairs.len() as f64 / possible_pairs as f64) * 100.0
    };

    let mut warnings = trace.warnings.clone();
    for tx in &trace.transactions {
        warnings.extend(tx.warnings.clone());
        if !tx.read_info_complete {
            warnings.push(TraceParseWarning {
                tx_index: Some(tx.tx_index),
                message: "analysis is a lower bound: read set is marked incomplete".to_owned(),
            });
        }
    }
    warnings.sort();
    warnings.dedup();

    let mut report = ParallelismReport {
        report_version: "1".to_owned(),
        chain: trace.chain.to_string(),
        block: BlockReportRef {
            number: trace.block.number,
            hash: trace.block.hash.clone(),
        },
        tx_count,
        access_model: if has_incomplete_reads {
            "declared-read-write-lower-bound".to_owned()
        } else {
            "declared-read-write-complete".to_owned()
        },
        conflict_pair_count: conflict_pairs.len() as u64,
        conflict_percentage,
        wave_count: waves.len(),
        max_wave_width: waves.iter().map(Vec::len).max().unwrap_or(0),
        critical_path_length,
        theoretical_parallelism_ceiling: if critical_path_length == 0 {
            0.0
        } else {
            tx_count as f64 / critical_path_length as f64
        },
        top_hot_contracts: top_hot_contracts(trace, 5),
        top_hot_storage_slots: top_hot_storage_slots(trace, 5),
        access_histogram: access_histogram(trace),
        tx_summaries: trace
            .transactions
            .iter()
            .map(|tx| {
                let set = tx.access_set();
                TxSummary {
                    tx_index: tx.tx_index.0,
                    tx_hash: tx.tx_hash.0.clone(),
                    read_count: set.reads.len(),
                    write_count: set.writes.len(),
                    conflict_degree: *conflict_degrees.get(&tx.tx_index).unwrap_or(&0),
                    wave: *tx_to_wave.get(&tx.tx_index).unwrap_or(&0),
                }
            })
            .collect(),
        warnings,
        generated_from_fixture,
        generated_from_rpc,
        deterministic_hash: String::new(),
    };
    report.deterministic_hash = deterministic_hash(&report);
    report
}

pub fn render_markdown(report: &ParallelismReport) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# Parallelism Report: {} block {}\n\n",
        report.chain, report.block.number
    ));
    out.push_str("## Summary\n\n");
    out.push_str(&format!("- Transactions: {}\n", report.tx_count));
    out.push_str(&format!(
        "- Conflict pairs: {} ({:.3}%)\n",
        report.conflict_pair_count, report.conflict_percentage
    ));
    out.push_str(&format!("- Waves: {}\n", report.wave_count));
    out.push_str(&format!("- Max wave width: {}\n", report.max_wave_width));
    out.push_str(&format!(
        "- Critical path length: {}\n",
        report.critical_path_length
    ));
    out.push_str(&format!(
        "- Theoretical parallelism ceiling: {:.3}x\n\n",
        report.theoretical_parallelism_ceiling
    ));

    out.push_str("## Hot Contracts\n\n| contract | accesses |\n| --- | ---: |\n");
    for item in &report.top_hot_contracts {
        out.push_str(&format!("| `{}` | {} |\n", item.key, item.count));
    }
    out.push_str("\n## Hot Storage Slots\n\n| slot | accesses |\n| --- | ---: |\n");
    for item in &report.top_hot_storage_slots {
        out.push_str(&format!("| `{}` | {} |\n", item.key, item.count));
    }
    out.push_str("\n## Waves\n\n| tx | wave | degree | reads | writes |\n| ---: | ---: | ---: | ---: | ---: |\n");
    for tx in &report.tx_summaries {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            tx.tx_index, tx.wave, tx.conflict_degree, tx.read_count, tx.write_count
        ));
    }
    if !report.warnings.is_empty() {
        out.push_str("\n## Warnings\n\n");
        for warning in &report.warnings {
            out.push_str(&format!("- {}\n", warning_label(warning)));
        }
    }
    out.push_str("\n## Caveats\n\nThis report studies access-set contention and deterministic scheduling structure. It is not a production throughput or gas benchmark. If a trace marks reads incomplete, conflict counts are lower bounds.\n");
    out
}

pub fn render_html(report: &ParallelismReport) -> String {
    render_html_with_command(report, None)
}

pub fn render_html_with_command(report: &ParallelismReport, command: Option<&str>) -> String {
    let contract_bars = report
        .top_hot_contracts
        .iter()
        .map(|item| {
            let width = item.count.min(100);
            format!(
                "<div><code>{}</code><div style=\"background:#d7e7ff;width:{}%;height:10px\"></div><span>{}</span></div>",
                escape_html(&item.key),
                width,
                item.count
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let storage_bars = report
        .top_hot_storage_slots
        .iter()
        .map(|item| {
            let width = item.count.min(100);
            format!(
                "<div><code>{}</code><div style=\"background:#dff3df;width:{}%;height:10px\"></div><span>{}</span></div>",
                escape_html(&item.key),
                width,
                item.count
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let tx_rows = report
        .tx_summaries
        .iter()
        .map(|tx| {
            format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                tx.tx_index, tx.wave, tx.conflict_degree, tx.read_count, tx.write_count
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let warning_items = report
        .warnings
        .iter()
        .map(|warning| format!("<li>{}</li>", escape_html(&warning_label(warning))))
        .collect::<Vec<_>>()
        .join("");
    let command = command.map(escape_html).unwrap_or_else(|| {
        "see README.md for the exact command that generated this report".to_owned()
    });
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>Parallelism Report</title><style>body{{font-family:system-ui,sans-serif;margin:32px;line-height:1.4}}.cards{{display:grid;grid-template-columns:repeat(auto-fit,minmax(160px,1fr));gap:12px}}.card{{border:1px solid #ddd;padding:12px;border-radius:6px}}table{{border-collapse:collapse;width:100%;margin-top:16px}}td,th{{border:1px solid #ddd;padding:6px;text-align:left}}code{{font-size:12px}}</style></head><body><h1>Parallelism Report</h1><p>{} block {}</p><div class=\"cards\"><div class=\"card\"><b>txs</b><br>{}</div><div class=\"card\"><b>conflicts</b><br>{}</div><div class=\"card\"><b>waves</b><br>{}</div><div class=\"card\"><b>max width</b><br>{}</div><div class=\"card\"><b>ceiling</b><br>{:.3}x</div></div><h2>Hot Contracts</h2>{}<h2>Hot Storage Slots</h2>{}<h2>Waves</h2><table><tr><th>tx</th><th>wave</th><th>degree</th><th>reads</th><th>writes</th></tr>{}</table><h2>Warnings</h2><ul>{}</ul><h2>Commands</h2><pre>{}</pre></body></html>",
        escape_html(&report.chain),
        report.block.number,
        report.tx_count,
        report.conflict_pair_count,
        report.wave_count,
        report.max_wave_width,
        report.theoretical_parallelism_ceiling,
        contract_bars,
        storage_bars,
        tx_rows,
        warning_items,
        command
    )
}

pub fn schedule_trace_json(report: &ParallelismReport) -> serde_json::Value {
    let events = report
        .tx_summaries
        .iter()
        .map(|tx| {
            serde_json::json!({
                "name": format!("tx-{}", tx.tx_index),
                "cat": "parallelism-schedule",
                "ph": "X",
                "pid": 1,
                "tid": format!("wave-{}", tx.wave),
                "ts": (tx.wave as u64) * 1_000,
                "dur": 900,
                "args": {
                    "tx_hash": tx.tx_hash,
                    "conflict_degree": tx.conflict_degree
                }
            })
        })
        .collect::<Vec<_>>();
    serde_json::json!({ "traceEvents": events })
}

pub(crate) fn conflict_pairs(trace: &BlockAccessTrace) -> Vec<ConflictPair> {
    let sets = trace
        .transactions
        .iter()
        .map(|tx| (tx.tx_index, tx.access_set()))
        .collect::<Vec<_>>();
    let mut pairs = Vec::new();
    for left_pos in 0..sets.len() {
        for right_pos in (left_pos + 1)..sets.len() {
            let (left_index, left) = &sets[left_pos];
            let (right_index, right) = &sets[right_pos];
            let mut keys = BTreeSet::new();
            keys.extend(left.writes.intersection(&right.writes).cloned());
            keys.extend(left.writes.intersection(&right.reads).cloned());
            keys.extend(left.reads.intersection(&right.writes).cloned());
            if !keys.is_empty() {
                pairs.push(ConflictPair {
                    earlier: *left_index,
                    later: *right_index,
                    keys: keys.into_iter().collect(),
                });
            }
        }
    }
    pairs
}

pub(crate) fn dependency_graph(conflicts: &[ConflictPair]) -> BTreeMap<TxIndex, BTreeSet<TxIndex>> {
    let mut graph = BTreeMap::<TxIndex, BTreeSet<TxIndex>>::new();
    for conflict in conflicts {
        graph
            .entry(conflict.later)
            .or_default()
            .insert(conflict.earlier);
    }
    graph
}

pub(crate) fn build_waves(
    trace: &BlockAccessTrace,
    dependencies: &BTreeMap<TxIndex, BTreeSet<TxIndex>>,
) -> Vec<Vec<TxIndex>> {
    let mut remaining = trace
        .transactions
        .iter()
        .map(|tx| tx.tx_index)
        .collect::<BTreeSet<_>>();
    let mut assigned = BTreeSet::new();
    let mut waves = Vec::new();

    while !remaining.is_empty() {
        let wave = remaining
            .iter()
            .copied()
            .filter(|tx| {
                dependencies
                    .get(tx)
                    .map(|deps| deps.iter().all(|dep| assigned.contains(dep)))
                    .unwrap_or(true)
            })
            .collect::<Vec<_>>();
        let wave = if wave.is_empty() {
            vec![*remaining.iter().next().expect("remaining is non-empty")]
        } else {
            wave
        };
        for tx in &wave {
            remaining.remove(tx);
        }
        assigned.extend(wave.iter().copied());
        waves.push(wave);
    }
    waves
}

pub(crate) fn tx_wave_map(waves: &[Vec<TxIndex>]) -> BTreeMap<TxIndex, usize> {
    let mut out = BTreeMap::new();
    for (wave_idx, wave) in waves.iter().enumerate() {
        for tx in wave {
            out.insert(*tx, wave_idx);
        }
    }
    out
}

pub(crate) fn critical_path_length(
    trace: &BlockAccessTrace,
    dependencies: &BTreeMap<TxIndex, BTreeSet<TxIndex>>,
) -> usize {
    let mut depth = BTreeMap::<TxIndex, usize>::new();
    for tx in &trace.transactions {
        let value = dependencies
            .get(&tx.tx_index)
            .map(|deps| {
                deps.iter()
                    .filter_map(|dep| depth.get(dep).copied())
                    .max()
                    .unwrap_or(0)
                    + 1
            })
            .unwrap_or(1);
        depth.insert(tx.tx_index, value);
    }
    depth.values().copied().max().unwrap_or(0)
}

pub(crate) fn conflict_degrees(conflicts: &[ConflictPair]) -> BTreeMap<TxIndex, u64> {
    let mut degrees = BTreeMap::new();
    for conflict in conflicts {
        *degrees.entry(conflict.earlier).or_insert(0) += 1;
        *degrees.entry(conflict.later).or_insert(0) += 1;
    }
    degrees
}

fn top_hot_contracts(trace: &BlockAccessTrace, limit: usize) -> Vec<HotItem> {
    let mut counts = BTreeMap::<String, u64>::new();
    for tx in &trace.transactions {
        for access in &tx.accesses {
            *counts.entry(access.key.address().to_string()).or_insert(0) += 1;
        }
    }
    top_items(counts, limit)
}

fn top_hot_storage_slots(trace: &BlockAccessTrace, limit: usize) -> Vec<HotItem> {
    let mut counts = BTreeMap::<String, u64>::new();
    for tx in &trace.transactions {
        for access in &tx.accesses {
            if let Some((address, slot)) = access.key.storage_slot() {
                *counts.entry(format!("{address}:{slot}")).or_insert(0) += 1;
            }
        }
    }
    top_items(counts, limit)
}

fn top_items(counts: BTreeMap<String, u64>, limit: usize) -> Vec<HotItem> {
    let mut items = counts
        .into_iter()
        .map(|(key, count)| HotItem { key, count })
        .collect::<Vec<_>>();
    items.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.key.cmp(&right.key))
    });
    items.truncate(limit);
    items
}

fn access_histogram(trace: &BlockAccessTrace) -> BTreeMap<String, u64> {
    let mut out = BTreeMap::new();
    for tx in &trace.transactions {
        for access in &tx.accesses {
            let prefix = match access.kind {
                TraceAccessKind::Read => "read",
                TraceAccessKind::Write => "write",
                TraceAccessKind::ReadWrite => "read_write",
            };
            *out.entry(prefix.to_owned()).or_insert(0) += 1;
        }
    }
    out
}

fn deterministic_hash(report: &ParallelismReport) -> String {
    let mut clone = report.clone();
    clone.deterministic_hash.clear();
    let bytes = serde_json::to_vec(&clone).expect("report serialization is infallible");
    format!("{:016x}", stable_fnv1a64(&bytes))
}

pub(crate) fn stable_fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

pub(crate) fn pair_count(len: usize) -> u64 {
    if len < 2 {
        0
    } else {
        (len as u64 * (len as u64 - 1)) / 2
    }
}

pub(crate) fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn warning_label(warning: &TraceParseWarning) -> String {
    warning
        .tx_index
        .map(|tx_index| format!("tx_index {}: {}", tx_index.0, warning.message))
        .unwrap_or_else(|| warning.message.clone())
}

#[cfg(test)]
mod tests {
    use parallel_revm_lab_trace_model::{
        Address, BlockRef, ChainKind, StorageKey, TraceAccess, TraceAccessKind, TxHash, TxTrace,
    };

    use super::*;

    #[test]
    fn independent_txs_form_one_wide_wave() {
        let trace = trace_with_slots(&[1, 2, 3]);
        let report = analyze_block_trace(&trace, true, false);
        assert_eq!(report.conflict_pair_count, 0);
        assert_eq!(report.wave_count, 1);
        assert_eq!(report.max_wave_width, 3);
    }

    #[test]
    fn hot_slot_txs_form_serial_waves() {
        let trace = trace_with_slots(&[1, 1, 1]);
        let report = analyze_block_trace(&trace, true, false);
        assert_eq!(report.conflict_pair_count, 3);
        assert_eq!(report.wave_count, 3);
        assert_eq!(report.critical_path_length, 3);
    }

    #[test]
    fn report_is_stable_across_runs() {
        let trace = trace_with_slots(&[1, 2, 1, 3]);
        let left = analyze_block_trace(&trace, true, false);
        let right = analyze_block_trace(&trace, true, false);
        assert_eq!(left.deterministic_hash, right.deterministic_hash);
        assert_eq!(left.top_hot_storage_slots, right.top_hot_storage_slots);
    }

    #[test]
    fn committed_fixture_produces_expected_summary() {
        let mut trace: BlockAccessTrace =
            serde_json::from_str(include_str!("../../../fixtures/base-mini-trace.json")).unwrap();
        trace.normalize();
        let report = analyze_block_trace(&trace, true, false);

        assert_eq!(report.tx_count, 12);
        assert_eq!(report.conflict_pair_count, 7);
        assert_eq!(report.wave_count, 3);
        assert_eq!(report.max_wave_width, 7);
        assert_eq!(report.critical_path_length, 3);
        assert_eq!(
            report.top_hot_contracts[0].key,
            "0x1111111111111111111111111111111111111111"
        );
        assert!(report
            .warnings
            .iter()
            .any(|warning| warning.message.contains("lower bound")));
    }

    #[test]
    fn waves_partition_txs_and_respect_conflict_dependencies() {
        let mut trace: BlockAccessTrace =
            serde_json::from_str(include_str!("../../../fixtures/base-mini-trace.json")).unwrap();
        trace.normalize();
        let conflicts = conflict_pairs(&trace);
        let dependencies = dependency_graph(&conflicts);
        let waves = build_waves(&trace, &dependencies);
        let tx_to_wave = tx_wave_map(&waves);
        let assigned = waves.iter().flatten().copied().collect::<BTreeSet<_>>();
        let expected = trace
            .transactions
            .iter()
            .map(|tx| tx.tx_index)
            .collect::<BTreeSet<_>>();

        assert_eq!(assigned, expected);
        for conflict in conflicts {
            assert!(tx_to_wave[&conflict.earlier] < tx_to_wave[&conflict.later]);
        }
    }

    fn trace_with_slots(slots: &[u64]) -> BlockAccessTrace {
        BlockAccessTrace {
            chain: ChainKind::new("fixture"),
            block: BlockRef {
                number: 1,
                hash: None,
            },
            transactions: slots
                .iter()
                .enumerate()
                .map(|(idx, slot)| TxTrace {
                    tx_index: TxIndex(idx as u64),
                    tx_hash: TxHash(format!("0x{idx:064x}")),
                    from: Address::canonical(format!("0x{:040x}", idx + 1)),
                    to: Some(Address::canonical(
                        "0x1111111111111111111111111111111111111111",
                    )),
                    read_info_complete: true,
                    accesses: vec![TraceAccess {
                        kind: TraceAccessKind::ReadWrite,
                        key: TraceAccessKey::Storage {
                            address: Address::canonical(
                                "0x1111111111111111111111111111111111111111",
                            ),
                            slot: StorageKey::canonical(format!("0x{slot:064x}")),
                        },
                    }],
                    warnings: Vec::new(),
                })
                .collect(),
            warnings: Vec::new(),
        }
    }
}
