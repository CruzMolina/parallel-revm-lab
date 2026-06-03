use std::collections::{BTreeMap, BTreeSet};

use parallel_revm_lab_trace_model::{
    BlockAccessTrace, TraceAccessKey, TracePack, TracePackBlock, TraceParseWarning, TxIndex,
};
use serde::{Deserialize, Serialize};

use crate::{
    build_waves, conflict_degrees, conflict_pairs, critical_path_length, dependency_graph,
    escape_html, pair_count, stable_fnv1a64, tx_wave_map, ConflictPair,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TracePackDossier {
    pub report_version: String,
    pub chain: String,
    pub provenance: String,
    pub source: String,
    pub tracer_kind: String,
    pub start_block: u64,
    pub end_block: u64,
    pub block_count: usize,
    pub tx_count: usize,
    pub source_tx_count: Option<usize>,
    pub tx_coverage_percentage: Option<f64>,
    pub total_gas_used: Option<u64>,
    pub total_accesses: usize,
    pub unique_contracts: usize,
    pub unique_storage_slots: usize,
    pub conflict_pair_count: u64,
    pub conflict_percentage: f64,
    pub overlapping_tx_count: usize,
    pub overlapping_tx_percentage: f64,
    pub wave_count: usize,
    pub max_wave_width: usize,
    pub gas_weighted_conflict_percentage: Option<f64>,
    pub critical_path_length_by_tx: usize,
    pub gas_weighted_critical_path: Option<u64>,
    pub theoretical_parallelism_ceiling_by_tx: f64,
    pub theoretical_parallelism_ceiling_by_gas: Option<f64>,
    pub top_hot_contracts: Vec<HotContractDossier>,
    pub top_hot_storage_slots: Vec<HotSlotDossier>,
    pub contention_concentration: ContentionConcentration,
    pub parallelism_loss_decomposition: ParallelismLossDecomposition,
    pub worst_blocks_by_conflict_percentage: Vec<WorstBlockSummary>,
    pub worst_blocks_by_gas_weighted_critical_path: Vec<WorstBlockSummary>,
    pub worst_serializing_txs: Vec<SerializingTxSummary>,
    pub worker_simulation: Vec<WorkerSimulation>,
    pub blocks: Vec<BlockDossier>,
    pub warnings: Vec<String>,
    pub deterministic_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HotContractDossier {
    pub address: String,
    pub touching_txs: usize,
    pub unique_slots: usize,
    pub gas_of_touching_txs: Option<u64>,
    pub conflict_contribution: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HotSlotDossier {
    pub key: String,
    pub address: String,
    pub slot: String,
    pub touching_txs: usize,
    pub gas_of_touching_txs: Option<u64>,
    pub conflict_contribution: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContentionConcentration {
    pub top_1_conflict_percent: f64,
    pub top_5_conflict_percent: f64,
    pub top_10_conflict_percent: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParallelismLossDecomposition {
    pub hot_slot_conflict_contributions: u64,
    pub account_level_conflict_contributions: u64,
    pub unknown_incomplete_trace_warning_count: usize,
    pub max_worker_idle_percentage: Option<f64>,
    pub interpretation: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorstBlockSummary {
    pub block_number: u64,
    pub tx_count: usize,
    pub conflict_pair_count: u64,
    pub conflict_percentage: f64,
    pub gas_weighted_critical_path: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockDossier {
    pub block_number: u64,
    pub tx_count: usize,
    pub source_tx_count: Option<usize>,
    pub tx_coverage_percentage: Option<f64>,
    pub gas_used: Option<u64>,
    pub total_accesses: usize,
    pub conflict_pair_count: u64,
    pub conflict_percentage: f64,
    pub overlapping_tx_count: usize,
    pub overlapping_tx_percentage: f64,
    pub gas_weighted_conflict_percentage: Option<f64>,
    pub wave_count: usize,
    pub max_wave_width: usize,
    pub critical_path_length: usize,
    pub gas_critical_path: Option<u64>,
    pub ceiling_by_tx: f64,
    pub ceiling_by_gas: Option<f64>,
    pub top_hot_slots: Vec<HotSlotDossier>,
    pub top_hot_contracts: Vec<HotContractDossier>,
    pub worker_simulation: Vec<WorkerSimulation>,
    pub tx_summaries: Vec<DossierTxSummary>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DossierTxSummary {
    pub tx_index: u64,
    pub tx_hash: String,
    pub wave: usize,
    pub conflict_degree: u64,
    pub duration_units: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SerializingTxSummary {
    pub block_number: u64,
    pub tx_index: u64,
    pub tx_hash: String,
    pub wave: usize,
    pub conflict_degree: u64,
    pub duration_units: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkerSimulation {
    pub workers: usize,
    pub makespan: u64,
    pub speedup_vs_one_worker: f64,
    pub idle_percentage: f64,
    pub critical_path_bound: u64,
    pub interpretation: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccessListRecommendationReport {
    pub report_version: String,
    pub chain: String,
    pub provenance: String,
    pub tx_hints: Vec<TxAccessHint>,
    pub top_conflict_keys: Vec<ConflictKeyHint>,
    pub scheduling_helpful_txs: Vec<SchedulingTxHint>,
    pub warnings: Vec<String>,
    pub deterministic_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TxAccessHint {
    pub block_number: u64,
    pub tx_index: u64,
    pub tx_hash: String,
    pub observed_contracts: Vec<String>,
    pub observed_storage_keys: Vec<String>,
    pub warning: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConflictKeyHint {
    pub key: String,
    pub conflict_contribution: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SchedulingTxHint {
    pub block_number: u64,
    pub tx_index: u64,
    pub tx_hash: String,
    pub conflict_degree: u64,
}

#[derive(Clone, Debug)]
struct ScheduleStats {
    makespan: u64,
    idle_percentage: f64,
}

#[derive(Clone, Debug, Default)]
struct HotAccumulator {
    contract_txs: BTreeMap<String, BTreeSet<(u64, u64)>>,
    contract_slots: BTreeMap<String, BTreeSet<String>>,
    contract_gas: BTreeMap<String, u64>,
    slot_txs: BTreeMap<String, BTreeSet<(u64, u64)>>,
    slot_gas: BTreeMap<String, u64>,
    contract_conflicts: BTreeMap<String, u64>,
    slot_conflicts: BTreeMap<String, u64>,
    key_conflicts: BTreeMap<String, u64>,
}

pub fn analyze_trace_pack(pack: &TracePack, workers: &[usize]) -> TracePackDossier {
    let mut normalized = pack.clone();
    normalized.normalize();
    let workers = normalized_workers(workers);
    let mut hot = HotAccumulator::default();
    let mut blocks = Vec::new();
    let mut range_simulation = BTreeMap::<usize, u64>::new();
    let mut range_duration = BTreeMap::<usize, u64>::new();
    let mut total_conflicts = 0_u64;
    let mut total_pairs = 0_u64;
    let mut total_tx = 0_usize;
    let mut source_tx_total = 0_usize;
    let mut source_tx_present = false;
    let mut overlapping_tx_total = 0_usize;
    let mut wave_count = 0_usize;
    let mut max_wave_width = 0_usize;
    let mut total_accesses = 0_usize;
    let mut total_gas = 0_u64;
    let mut gas_all = true;
    let mut weighted_conflict_num = 0_u128;
    let mut weighted_conflict_den = 0_u128;
    let mut critical_path_by_tx = 0_usize;
    let mut gas_critical_path = 0_u64;
    let mut warnings = normalized.manifest.warnings.clone();

    for block in &normalized.blocks {
        let block_dossier = analyze_trace_pack_block(block, &workers, &mut hot);
        total_conflicts += block_dossier.conflict_pair_count;
        total_pairs += pair_count(block_dossier.tx_count);
        total_tx += block_dossier.tx_count;
        source_tx_total += block_dossier
            .source_tx_count
            .unwrap_or(block_dossier.tx_count);
        source_tx_present |= block_dossier.source_tx_count.is_some();
        overlapping_tx_total += block_dossier.overlapping_tx_count;
        wave_count += block_dossier.wave_count;
        max_wave_width = max_wave_width.max(block_dossier.max_wave_width);
        total_accesses += block_dossier.total_accesses;
        critical_path_by_tx += block_dossier.critical_path_length;
        if let Some(gas_used) = block.total_gas_used {
            total_gas += gas_used;
        } else {
            gas_all = false;
        }
        if let Some(path) = block_dossier.gas_critical_path {
            gas_critical_path += path;
        } else {
            gas_all = false;
        }
        if let Some((num, den)) = block_weighted_conflict_parts(block) {
            weighted_conflict_num += num;
            weighted_conflict_den += den;
        } else {
            gas_all = false;
        }
        for warning in &block_dossier.warnings {
            warnings.push(format!("block {}: {warning}", block.block_number));
        }
        for simulation in &block_dossier.worker_simulation {
            *range_simulation.entry(simulation.workers).or_insert(0) += simulation.makespan;
            *range_duration.entry(simulation.workers).or_insert(0) += block_duration(block);
        }
        blocks.push(block_dossier);
    }

    if !gas_all {
        warnings.push(
            "one or more blocks/transactions are missing gas; gas-weighted range metrics are unavailable"
                .to_owned(),
        );
    }
    warnings.sort();
    warnings.dedup();

    let worker_simulation = range_worker_simulations(
        &workers,
        &range_simulation,
        &range_duration,
        if gas_all {
            gas_critical_path
        } else {
            critical_path_by_tx as u64
        },
    );
    let top_hot_contracts = hot.top_contracts(10, gas_all);
    let top_hot_storage_slots = hot.top_slots(10, gas_all);
    let concentration = contention_concentration(&hot.key_conflicts);
    let max_worker_idle = worker_simulation
        .iter()
        .max_by_key(|simulation| simulation.workers)
        .map(|simulation| simulation.idle_percentage);
    let conflict_percentage = percentage(total_conflicts, total_pairs);
    let tx_coverage_percentage =
        source_tx_present.then(|| percentage(total_tx as u64, source_tx_total as u64));
    let overlapping_tx_percentage = percentage(overlapping_tx_total as u64, total_tx as u64);
    let worst_serializing_txs = worst_serializing_txs(&blocks, 10);
    let mut dossier = TracePackDossier {
        report_version: "trace-pack-dossier-v1".to_owned(),
        chain: normalized.manifest.chain.to_string(),
        provenance: normalized.manifest.provenance.clone(),
        source: normalized.manifest.source.clone(),
        tracer_kind: normalized.manifest.tracer_kind.clone(),
        start_block: normalized.manifest.start_block,
        end_block: normalized.manifest.end_block,
        block_count: normalized.blocks.len(),
        tx_count: total_tx,
        source_tx_count: source_tx_present.then_some(source_tx_total),
        tx_coverage_percentage,
        total_gas_used: gas_all.then_some(total_gas),
        total_accesses,
        unique_contracts: hot.contract_txs.len(),
        unique_storage_slots: hot.slot_txs.len(),
        conflict_pair_count: total_conflicts,
        conflict_percentage,
        overlapping_tx_count: overlapping_tx_total,
        overlapping_tx_percentage,
        wave_count,
        max_wave_width,
        gas_weighted_conflict_percentage: if gas_all {
            weighted_percentage(weighted_conflict_num, weighted_conflict_den)
        } else {
            None
        },
        critical_path_length_by_tx: critical_path_by_tx,
        gas_weighted_critical_path: gas_all.then_some(gas_critical_path),
        theoretical_parallelism_ceiling_by_tx: if critical_path_by_tx == 0 {
            0.0
        } else {
            total_tx as f64 / critical_path_by_tx as f64
        },
        theoretical_parallelism_ceiling_by_gas: if gas_all && gas_critical_path > 0 {
            Some(total_gas as f64 / gas_critical_path as f64)
        } else {
            None
        },
        top_hot_contracts,
        top_hot_storage_slots,
        contention_concentration: concentration,
        parallelism_loss_decomposition: ParallelismLossDecomposition {
            hot_slot_conflict_contributions: hot.slot_conflicts.values().sum(),
            account_level_conflict_contributions: hot
                .key_conflicts
                .iter()
                .filter(|(key, _)| !key.starts_with("storage:"))
                .map(|(_, count)| *count)
                .sum(),
            unknown_incomplete_trace_warning_count: warnings
                .iter()
                .filter(|warning| warning.contains("incomplete"))
                .count(),
            max_worker_idle_percentage: max_worker_idle,
            interpretation: "loss decomposition is based on observed access keys and theoretical list scheduling; incomplete traces may hide additional dependencies".to_owned(),
        },
        worst_blocks_by_conflict_percentage: worst_by_conflict(&blocks, 5),
        worst_blocks_by_gas_weighted_critical_path: worst_by_gas_path(&blocks, 5),
        worst_serializing_txs,
        worker_simulation,
        blocks,
        warnings,
        deterministic_hash: String::new(),
    };
    dossier.deterministic_hash = dossier_hash(&dossier);
    dossier
}

pub fn render_dossier_markdown(dossier: &TracePackDossier) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# Contention Dossier: {} blocks {}-{}\n\n",
        dossier.chain, dossier.start_block, dossier.end_block
    ));
    out.push_str(&format!(
        "**Provenance:** `{}`\n\n**Tracer:** `{}`\n\n",
        dossier.provenance, dossier.tracer_kind
    ));
    out.push_str("## Summary\n\n");
    out.push_str(&format!("- Blocks: {}\n", dossier.block_count));
    out.push_str(&format!("- Transactions: {}\n", dossier.tx_count));
    if let (Some(source_tx_count), Some(coverage)) =
        (dossier.source_tx_count, dossier.tx_coverage_percentage)
    {
        out.push_str(&format!(
            "- Source transactions covered: {} of {} ({:.3}%)\n",
            dossier.tx_count, source_tx_count, coverage
        ));
    }
    out.push_str(&format!("- Accesses: {}\n", dossier.total_accesses));
    out.push_str(&format!(
        "- Conflict pairs: {} ({:.3}%)\n",
        dossier.conflict_pair_count, dossier.conflict_percentage
    ));
    out.push_str(&format!(
        "- Overlapping transactions: {} ({:.3}%)\n",
        dossier.overlapping_tx_count, dossier.overlapping_tx_percentage
    ));
    out.push_str(&format!("- Waves: {}\n", dossier.wave_count));
    out.push_str(&format!("- Max wave width: {}\n", dossier.max_wave_width));
    out.push_str(&format!(
        "- Critical path by tx count: {}\n",
        dossier.critical_path_length_by_tx
    ));
    out.push_str(&format!(
        "- Theoretical ceiling by tx: {:.3}x\n",
        dossier.theoretical_parallelism_ceiling_by_tx
    ));
    if let (Some(gas), Some(path), Some(ceiling)) = (
        dossier.total_gas_used,
        dossier.gas_weighted_critical_path,
        dossier.theoretical_parallelism_ceiling_by_gas,
    ) {
        out.push_str(&format!("- Total gas covered: {gas}\n"));
        out.push_str(&format!("- Gas-weighted critical path: {path}\n"));
        out.push_str(&format!("- Theoretical ceiling by gas: {ceiling:.3}x\n"));
    } else {
        out.push_str("- Gas-weighted metrics: unavailable because gas is missing\n");
    }
    out.push_str("\n## Worker Simulation\n\n| workers | makespan | speedup | idle | interpretation |\n| ---: | ---: | ---: | ---: | --- |\n");
    for simulation in &dossier.worker_simulation {
        out.push_str(&format!(
            "| {} | {} | {:.3}x | {:.2}% | {} |\n",
            simulation.workers,
            simulation.makespan,
            simulation.speedup_vs_one_worker,
            simulation.idle_percentage,
            simulation.interpretation
        ));
    }
    if !dossier.worst_serializing_txs.is_empty() {
        out.push_str("\n## Worst Serializing Transactions\n\n| block | tx | wave | conflicts | duration | tx hash |\n| ---: | ---: | ---: | ---: | ---: | --- |\n");
        for tx in &dossier.worst_serializing_txs {
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} | `{}` |\n",
                tx.block_number,
                tx.tx_index,
                tx.wave,
                tx.conflict_degree,
                tx.duration_units,
                tx.tx_hash
            ));
        }
    }
    out.push_str("\n## Hot Contracts\n\n| contract | txs | unique slots | gas covered | conflict contribution |\n| --- | ---: | ---: | ---: | ---: |\n");
    for item in &dossier.top_hot_contracts {
        out.push_str(&format!(
            "| `{}` | {} | {} | {} | {} |\n",
            item.address,
            item.touching_txs,
            item.unique_slots,
            option_u64(item.gas_of_touching_txs),
            item.conflict_contribution
        ));
    }
    out.push_str("\n## Hot Storage Slots\n\n| slot | txs | gas covered | conflict contribution |\n| --- | ---: | ---: | ---: |\n");
    for item in &dossier.top_hot_storage_slots {
        out.push_str(&format!(
            "| `{}` | {} | {} | {} |\n",
            item.key,
            item.touching_txs,
            option_u64(item.gas_of_touching_txs),
            item.conflict_contribution
        ));
    }
    if !dossier.warnings.is_empty() {
        out.push_str("\n## Warnings\n\n");
        for warning in &dossier.warnings {
            out.push_str(&format!("- {warning}\n"));
        }
    }
    out.push_str("\n## What This Proves\n\nThis dossier shows deterministic access-contention structure, hot-state concentration, gas-weighted theoretical scheduling bounds where gas is available, and worker-count sensitivity for the supplied trace pack.\n\n## What This Does Not Prove\n\nIt is not production TPS, not Ggas/s, not full block replay, and not proof that observed access hints are complete Ethereum access lists.\n");
    out
}

pub fn render_dossier_html(dossier: &TracePackDossier, command: &str) -> String {
    let worker_rows = dossier
        .worker_simulation
        .iter()
        .map(|simulation| {
            format!(
                "<tr><td>{}</td><td>{}</td><td>{:.3}x</td><td>{:.2}%</td><td>{}</td></tr>",
                simulation.workers,
                simulation.makespan,
                simulation.speedup_vs_one_worker,
                simulation.idle_percentage,
                escape_html(&simulation.interpretation)
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let contract_rows = dossier
        .top_hot_contracts
        .iter()
        .map(|item| {
            format!(
                "<tr><td><code>{}</code></td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                escape_html(&item.address),
                item.touching_txs,
                item.unique_slots,
                option_u64(item.gas_of_touching_txs),
                item.conflict_contribution
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let slot_rows = dossier
        .top_hot_storage_slots
        .iter()
        .map(|item| {
            format!(
                "<tr><td><code>{}</code></td><td>{}</td><td>{}</td><td>{}</td></tr>",
                escape_html(&item.key),
                item.touching_txs,
                option_u64(item.gas_of_touching_txs),
                item.conflict_contribution
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let warnings = dossier
        .warnings
        .iter()
        .map(|warning| format!("<li>{}</li>", escape_html(warning)))
        .collect::<Vec<_>>()
        .join("");
    let serializing_rows = dossier
        .worst_serializing_txs
        .iter()
        .map(|tx| {
            format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td><code>{}</code></td></tr>",
                tx.block_number,
                tx.tx_index,
                tx.wave,
                tx.conflict_degree,
                tx.duration_units,
                escape_html(&tx.tx_hash)
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>Contention Dossier</title><style>body{{font-family:system-ui,sans-serif;margin:32px;line-height:1.45;max-width:1180px}}.badge{{display:inline-block;border:1px solid #999;padding:3px 8px;border-radius:4px}}.cards{{display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:12px;margin:18px 0}}.card{{border:1px solid #ddd;padding:12px;border-radius:6px}}table{{border-collapse:collapse;width:100%;margin:12px 0 24px}}td,th{{border:1px solid #ddd;padding:6px;text-align:left;vertical-align:top}}code{{font-size:12px}}</style></head><body><h1>Contention Dossier</h1><p><span class=\"badge\">{}</span></p><p>{} blocks {}-{}</p><div class=\"cards\"><div class=\"card\"><b>txs</b><br>{}</div><div class=\"card\"><b>coverage</b><br>{}</div><div class=\"card\"><b>conflicts</b><br>{} ({:.3}%)</div><div class=\"card\"><b>overlap</b><br>{} ({:.3}%)</div><div class=\"card\"><b>waves</b><br>{} / max width {}</div><div class=\"card\"><b>tx ceiling</b><br>{:.3}x</div><div class=\"card\"><b>gas ceiling</b><br>{}</div></div><h2>Worker Simulation</h2><table><tr><th>workers</th><th>makespan</th><th>speedup</th><th>idle</th><th>interpretation</th></tr>{}</table><h2>Worst Serializing Transactions</h2><table><tr><th>block</th><th>tx</th><th>wave</th><th>conflicts</th><th>duration</th><th>tx hash</th></tr>{}</table><h2>Hot Contracts</h2><table><tr><th>contract</th><th>txs</th><th>unique slots</th><th>gas covered</th><th>conflicts</th></tr>{}</table><h2>Hot Storage Slots</h2><table><tr><th>slot</th><th>txs</th><th>gas covered</th><th>conflicts</th></tr>{}</table><h2>Warnings</h2><ul>{}</ul><h2>Commands</h2><pre>{}</pre><h2>What This Does Not Prove</h2><p>This is a theoretical scheduling and contention model over observed trace-pack accesses. It is not production TPS, not Ggas/s, not full block replay, and not a complete Ethereum access-list generator.</p></body></html>",
        escape_html(&dossier.provenance),
        escape_html(&dossier.chain),
        dossier.start_block,
        dossier.end_block,
        dossier.tx_count,
        dossier
            .tx_coverage_percentage
            .map(|value| format!("{value:.3}%"))
            .unwrap_or_else(|| "n/a".to_owned()),
        dossier.conflict_pair_count,
        dossier.conflict_percentage,
        dossier.overlapping_tx_count,
        dossier.overlapping_tx_percentage,
        dossier.wave_count,
        dossier.max_wave_width,
        dossier.theoretical_parallelism_ceiling_by_tx,
        dossier
            .theoretical_parallelism_ceiling_by_gas
            .map(|value| format!("{value:.3}x"))
            .unwrap_or_else(|| "unavailable".to_owned()),
        worker_rows,
        serializing_rows,
        contract_rows,
        slot_rows,
        warnings,
        escape_html(command)
    )
}

pub fn dossier_schedule_trace_json(dossier: &TracePackDossier) -> serde_json::Value {
    let mut events = Vec::new();
    for block in &dossier.blocks {
        for tx in &block.tx_summaries {
            events.push(serde_json::json!({
                "name": format!("block-{}-tx-{}", block.block_number, tx.tx_index),
                "cat": "trace-pack-schedule",
                "ph": "X",
                "pid": block.block_number,
                "tid": format!("wave-{}", tx.wave),
                "ts": (block.block_number * 1_000_000) + (tx.wave as u64 * 1_000),
                "dur": tx.duration_units.max(1),
                "args": {
                    "tx_hash": tx.tx_hash,
                    "conflict_degree": tx.conflict_degree
                }
            }));
        }
    }
    serde_json::json!({ "traceEvents": events })
}

pub fn render_hot_contracts_csv(dossier: &TracePackDossier) -> String {
    let mut out =
        "address,touching_txs,unique_slots,gas_of_touching_txs,conflict_contribution\n".to_owned();
    for item in &dossier.top_hot_contracts {
        out.push_str(&format!(
            "{},{},{},{},{}\n",
            item.address,
            item.touching_txs,
            item.unique_slots,
            option_u64(item.gas_of_touching_txs),
            item.conflict_contribution
        ));
    }
    out
}

pub fn render_hot_slots_csv(dossier: &TracePackDossier) -> String {
    let mut out =
        "key,address,slot,touching_txs,gas_of_touching_txs,conflict_contribution\n".to_owned();
    for item in &dossier.top_hot_storage_slots {
        out.push_str(&format!(
            "{},{},{},{},{},{}\n",
            item.key,
            item.address,
            item.slot,
            item.touching_txs,
            option_u64(item.gas_of_touching_txs),
            item.conflict_contribution
        ));
    }
    out
}

pub fn render_worker_simulation_csv(dossier: &TracePackDossier) -> String {
    let mut out = "workers,makespan,speedup_vs_one_worker,idle_percentage,critical_path_bound,interpretation\n".to_owned();
    for item in &dossier.worker_simulation {
        out.push_str(&format!(
            "{},{},{:.6},{:.6},{},{}\n",
            item.workers,
            item.makespan,
            item.speedup_vs_one_worker,
            item.idle_percentage,
            item.critical_path_bound,
            item.interpretation
        ));
    }
    out
}

pub fn recommend_access_lists(pack: &TracePack) -> AccessListRecommendationReport {
    let dossier = analyze_trace_pack(pack, &[1]);
    let mut tx_hints = Vec::new();
    let mut scheduling_txs = Vec::new();
    for block in &pack.blocks {
        let trace = block.to_block_trace();
        let conflicts = conflict_pairs(&trace);
        let degrees = conflict_degrees(&conflicts);
        for tx in &block.transactions {
            let mut contracts = BTreeSet::new();
            let mut slots = BTreeSet::new();
            for access in &tx.accesses {
                contracts.insert(access.address.to_string());
                if let Some(slot) = &access.slot {
                    slots.insert(format!("{}:{slot}", access.address));
                }
            }
            tx_hints.push(TxAccessHint {
                block_number: block.block_number,
                tx_index: tx.tx_index.0,
                tx_hash: tx.tx_hash.0.clone(),
                observed_contracts: contracts.into_iter().collect(),
                observed_storage_keys: slots.into_iter().collect(),
                warning:
                    "observed access hints only; dynamic or unobserved accesses may be missing"
                        .to_owned(),
            });
            let degree = *degrees.get(&tx.tx_index).unwrap_or(&0);
            if degree > 0 {
                scheduling_txs.push(SchedulingTxHint {
                    block_number: block.block_number,
                    tx_index: tx.tx_index.0,
                    tx_hash: tx.tx_hash.0.clone(),
                    conflict_degree: degree,
                });
            }
        }
    }
    scheduling_txs.sort_by(|left, right| {
        right
            .conflict_degree
            .cmp(&left.conflict_degree)
            .then_with(|| left.block_number.cmp(&right.block_number))
            .then_with(|| left.tx_index.cmp(&right.tx_index))
    });
    let mut top_conflict_keys = dossier
        .top_hot_storage_slots
        .iter()
        .map(|slot| ConflictKeyHint {
            key: slot.key.clone(),
            conflict_contribution: slot.conflict_contribution,
        })
        .filter(|hint| hint.conflict_contribution > 0)
        .collect::<Vec<_>>();
    top_conflict_keys.sort_by(|left, right| {
        right
            .conflict_contribution
            .cmp(&left.conflict_contribution)
            .then_with(|| left.key.cmp(&right.key))
    });
    top_conflict_keys.truncate(10);
    let mut report = AccessListRecommendationReport {
        report_version: "observed-access-hints-v1".to_owned(),
        chain: pack.manifest.chain.to_string(),
        provenance: pack.manifest.provenance.clone(),
        tx_hints,
        top_conflict_keys,
        scheduling_helpful_txs: scheduling_txs,
        warnings: vec![
            "observed access hints are not production-ready Ethereum access lists".to_owned(),
            "incomplete traces can miss dynamic storage keys and account/code/balance reads"
                .to_owned(),
        ],
        deterministic_hash: String::new(),
    };
    report.deterministic_hash = recommendation_hash(&report);
    report
}

pub fn render_access_hints_markdown(report: &AccessListRecommendationReport) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Observed Access Hints: {}\n\n", report.chain));
    out.push_str(&format!("**Provenance:** `{}`\n\n", report.provenance));
    out.push_str("These are observed access hints, not complete production Ethereum access lists. Dynamic access and incomplete trace caveats apply.\n\n");
    out.push_str("## Candidate Conflict Keys\n\n| key | conflict contribution |\n| --- | ---: |\n");
    for key in &report.top_conflict_keys {
        out.push_str(&format!(
            "| `{}` | {} |\n",
            key.key, key.conflict_contribution
        ));
    }
    out.push_str("\n## Scheduling-Helpful Transactions\n\n| block | tx | conflicts | tx hash |\n| ---: | ---: | ---: | --- |\n");
    for tx in report.scheduling_helpful_txs.iter().take(25) {
        out.push_str(&format!(
            "| {} | {} | {} | `{}` |\n",
            tx.block_number, tx.tx_index, tx.conflict_degree, tx.tx_hash
        ));
    }
    out.push_str("\n## Per-Transaction Observations\n\n| block | tx | contracts | storage keys | warning |\n| ---: | ---: | ---: | ---: | --- |\n");
    for tx in &report.tx_hints {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            tx.block_number,
            tx.tx_index,
            tx.observed_contracts.len(),
            tx.observed_storage_keys.len(),
            tx.warning
        ));
    }
    if !report.warnings.is_empty() {
        out.push_str("\n## Warnings\n\n");
        for warning in &report.warnings {
            out.push_str(&format!("- {warning}\n"));
        }
    }
    out
}

fn analyze_trace_pack_block(
    block: &TracePackBlock,
    workers: &[usize],
    hot: &mut HotAccumulator,
) -> BlockDossier {
    let trace = block.to_block_trace();
    let conflicts = conflict_pairs(&trace);
    let dependencies = dependency_graph(&conflicts);
    let waves = build_waves(&trace, &dependencies);
    let tx_to_wave = tx_wave_map(&waves);
    let degrees = conflict_degrees(&conflicts);
    let gas_complete = block.has_complete_gas();
    let durations = duration_map(block, gas_complete);
    let gas_critical_path =
        gas_complete.then(|| weighted_critical_path(&trace, &dependencies, &durations));
    let critical_path = critical_path_length(&trace, &dependencies);
    let tx_indices = trace
        .transactions
        .iter()
        .map(|tx| tx.tx_index)
        .collect::<Vec<_>>();
    let worker_simulation = worker_simulations(&tx_indices, &dependencies, &durations, workers);
    let tx_summaries = trace
        .transactions
        .iter()
        .map(|tx| DossierTxSummary {
            tx_index: tx.tx_index.0,
            tx_hash: tx.tx_hash.0.clone(),
            wave: *tx_to_wave.get(&tx.tx_index).unwrap_or(&0),
            conflict_degree: *degrees.get(&tx.tx_index).unwrap_or(&0),
            duration_units: *durations.get(&tx.tx_index).unwrap_or(&1),
        })
        .collect::<Vec<_>>();

    hot.observe_block(block, &conflicts);
    let block_hot = HotAccumulator::from_block(block, &conflicts);
    let pair_count = pair_count(block.transactions.len());
    let overlapping_tx_count = overlapping_tx_count(&trace);
    BlockDossier {
        block_number: block.block_number,
        tx_count: block.transactions.len(),
        source_tx_count: block.source_tx_count,
        tx_coverage_percentage: block
            .source_tx_count
            .map(|source| percentage(block.transactions.len() as u64, source as u64)),
        gas_used: gas_complete.then_some(block.total_gas_used.unwrap_or(0)),
        total_accesses: block.transactions.iter().map(|tx| tx.accesses.len()).sum(),
        conflict_pair_count: conflicts.len() as u64,
        conflict_percentage: percentage(conflicts.len() as u64, pair_count),
        overlapping_tx_count,
        overlapping_tx_percentage: percentage(
            overlapping_tx_count as u64,
            block.transactions.len() as u64,
        ),
        gas_weighted_conflict_percentage: block_weighted_conflict_parts(block)
            .and_then(|(num, den)| weighted_percentage(num, den)),
        wave_count: waves.len(),
        max_wave_width: waves.iter().map(Vec::len).max().unwrap_or(0),
        critical_path_length: critical_path,
        gas_critical_path,
        ceiling_by_tx: if critical_path == 0 {
            0.0
        } else {
            block.transactions.len() as f64 / critical_path as f64
        },
        ceiling_by_gas: match (gas_complete, gas_critical_path) {
            (true, Some(path)) if path > 0 => {
                Some(block.total_gas_used.unwrap_or(0) as f64 / path as f64)
            }
            _ => None,
        },
        top_hot_slots: block_hot.top_slots(5, gas_complete),
        top_hot_contracts: block_hot.top_contracts(5, gas_complete),
        worker_simulation,
        tx_summaries,
        warnings: trace_pack_block_warnings(&trace),
    }
}

fn trace_pack_block_warnings(trace: &BlockAccessTrace) -> Vec<String> {
    let mut warnings = trace
        .warnings
        .iter()
        .map(trace_warning_message)
        .collect::<Vec<_>>();
    for tx in &trace.transactions {
        warnings.extend(tx.warnings.iter().map(trace_warning_message));
    }
    warnings.sort();
    warnings.dedup();
    warnings
}

fn trace_warning_message(warning: &TraceParseWarning) -> String {
    match warning.tx_index {
        Some(tx_index) => format!("tx_index {}: {}", tx_index.0, warning.message),
        None => warning.message.clone(),
    }
}

fn overlapping_tx_count(trace: &BlockAccessTrace) -> usize {
    let keys_by_tx = trace
        .transactions
        .iter()
        .map(|tx| {
            let set = tx.access_set();
            let keys = set
                .reads
                .union(&set.writes)
                .cloned()
                .collect::<BTreeSet<_>>();
            (tx.tx_index, keys)
        })
        .collect::<Vec<_>>();
    let mut overlapping = BTreeSet::new();
    for left in 0..keys_by_tx.len() {
        for right in (left + 1)..keys_by_tx.len() {
            if keys_by_tx[left]
                .1
                .iter()
                .any(|key| keys_by_tx[right].1.contains(key))
            {
                overlapping.insert(keys_by_tx[left].0);
                overlapping.insert(keys_by_tx[right].0);
            }
        }
    }
    overlapping.len()
}

fn worst_serializing_txs(blocks: &[BlockDossier], limit: usize) -> Vec<SerializingTxSummary> {
    let mut out = blocks
        .iter()
        .flat_map(|block| {
            block.tx_summaries.iter().map(|tx| SerializingTxSummary {
                block_number: block.block_number,
                tx_index: tx.tx_index,
                tx_hash: tx.tx_hash.clone(),
                wave: tx.wave,
                conflict_degree: tx.conflict_degree,
                duration_units: tx.duration_units,
            })
        })
        .filter(|tx| tx.conflict_degree > 0)
        .collect::<Vec<_>>();
    out.sort_by(|left, right| {
        right
            .conflict_degree
            .cmp(&left.conflict_degree)
            .then_with(|| right.duration_units.cmp(&left.duration_units))
            .then_with(|| left.block_number.cmp(&right.block_number))
            .then_with(|| left.tx_index.cmp(&right.tx_index))
    });
    out.truncate(limit);
    out
}

fn duration_map(block: &TracePackBlock, gas_complete: bool) -> BTreeMap<TxIndex, u64> {
    block
        .transactions
        .iter()
        .map(|tx| {
            (
                tx.tx_index,
                if gas_complete {
                    tx.gas_used.unwrap_or(1).max(1)
                } else {
                    1
                },
            )
        })
        .collect()
}

fn block_duration(block: &TracePackBlock) -> u64 {
    if block.has_complete_gas() {
        block.transactions.iter().filter_map(|tx| tx.gas_used).sum()
    } else {
        block.transactions.len() as u64
    }
}

fn weighted_critical_path(
    trace: &parallel_revm_lab_trace_model::BlockAccessTrace,
    dependencies: &BTreeMap<TxIndex, BTreeSet<TxIndex>>,
    durations: &BTreeMap<TxIndex, u64>,
) -> u64 {
    let mut depth = BTreeMap::<TxIndex, u64>::new();
    for tx in &trace.transactions {
        let prefix = dependencies
            .get(&tx.tx_index)
            .map(|deps| {
                deps.iter()
                    .filter_map(|dep| depth.get(dep).copied())
                    .max()
                    .unwrap_or(0)
            })
            .unwrap_or(0);
        depth.insert(
            tx.tx_index,
            prefix + durations.get(&tx.tx_index).copied().unwrap_or(1),
        );
    }
    depth.values().copied().max().unwrap_or(0)
}

fn worker_simulations(
    tx_indices: &[TxIndex],
    dependencies: &BTreeMap<TxIndex, BTreeSet<TxIndex>>,
    durations: &BTreeMap<TxIndex, u64>,
    workers: &[usize],
) -> Vec<WorkerSimulation> {
    let one = simulate_stats(tx_indices, dependencies, durations, 1);
    let critical_path = weighted_critical_path_for_indices(tx_indices, dependencies, durations);
    workers
        .iter()
        .map(|workers| {
            let stats = simulate_stats(tx_indices, dependencies, durations, *workers);
            WorkerSimulation {
                workers: *workers,
                makespan: stats.makespan,
                speedup_vs_one_worker: if stats.makespan == 0 {
                    0.0
                } else {
                    one.makespan as f64 / stats.makespan as f64
                },
                idle_percentage: stats.idle_percentage,
                critical_path_bound: critical_path,
                interpretation: schedule_interpretation(
                    stats.makespan,
                    critical_path,
                    stats.idle_percentage,
                ),
            }
        })
        .collect()
}

fn range_worker_simulations(
    workers: &[usize],
    makespans: &BTreeMap<usize, u64>,
    durations: &BTreeMap<usize, u64>,
    critical_path: u64,
) -> Vec<WorkerSimulation> {
    let one = *makespans.get(&1).unwrap_or(&0);
    workers
        .iter()
        .map(|workers| {
            let makespan = *makespans.get(workers).unwrap_or(&0);
            let duration = *durations.get(workers).unwrap_or(&0);
            let idle = idle_percentage(duration, makespan, *workers);
            WorkerSimulation {
                workers: *workers,
                makespan,
                speedup_vs_one_worker: if makespan == 0 {
                    0.0
                } else {
                    one as f64 / makespan as f64
                },
                idle_percentage: idle,
                critical_path_bound: critical_path,
                interpretation: schedule_interpretation(makespan, critical_path, idle),
            }
        })
        .collect()
}

fn weighted_critical_path_for_indices(
    tx_indices: &[TxIndex],
    dependencies: &BTreeMap<TxIndex, BTreeSet<TxIndex>>,
    durations: &BTreeMap<TxIndex, u64>,
) -> u64 {
    let mut depth = BTreeMap::<TxIndex, u64>::new();
    for tx in tx_indices {
        let prefix = dependencies
            .get(tx)
            .map(|deps| {
                deps.iter()
                    .filter_map(|dep| depth.get(dep).copied())
                    .max()
                    .unwrap_or(0)
            })
            .unwrap_or(0);
        depth.insert(*tx, prefix + durations.get(tx).copied().unwrap_or(1));
    }
    depth.values().copied().max().unwrap_or(0)
}

fn simulate_stats(
    tx_indices: &[TxIndex],
    dependencies: &BTreeMap<TxIndex, BTreeSet<TxIndex>>,
    durations: &BTreeMap<TxIndex, u64>,
    workers: usize,
) -> ScheduleStats {
    let workers = workers.max(1);
    let total_duration = tx_indices
        .iter()
        .map(|tx| durations.get(tx).copied().unwrap_or(1))
        .sum::<u64>();
    let mut unscheduled = tx_indices.iter().copied().collect::<BTreeSet<_>>();
    let mut completed = BTreeSet::new();
    let mut running = BTreeMap::<TxIndex, u64>::new();
    let mut now = 0_u64;

    while !unscheduled.is_empty() || !running.is_empty() {
        let ready = unscheduled
            .iter()
            .copied()
            .filter(|tx| deps_done(*tx, dependencies, &completed))
            .collect::<Vec<_>>();
        for tx in ready {
            if running.len() >= workers {
                break;
            }
            unscheduled.remove(&tx);
            running.insert(tx, now + durations.get(&tx).copied().unwrap_or(1));
        }
        if running.is_empty() {
            break;
        }
        let next = running.values().copied().min().unwrap_or(now);
        now = next;
        let finished = running
            .iter()
            .filter_map(|(tx, finish)| (*finish == now).then_some(*tx))
            .collect::<Vec<_>>();
        for tx in finished {
            running.remove(&tx);
            completed.insert(tx);
        }
    }

    ScheduleStats {
        makespan: now,
        idle_percentage: idle_percentage(total_duration, now, workers),
    }
}

fn deps_done(
    tx: TxIndex,
    dependencies: &BTreeMap<TxIndex, BTreeSet<TxIndex>>,
    completed: &BTreeSet<TxIndex>,
) -> bool {
    dependencies
        .get(&tx)
        .map(|deps| deps.iter().all(|dep| completed.contains(dep)))
        .unwrap_or(true)
}

fn idle_percentage(total_duration: u64, makespan: u64, workers: usize) -> f64 {
    let capacity = makespan.saturating_mul(workers as u64);
    if capacity == 0 {
        0.0
    } else {
        ((capacity.saturating_sub(total_duration)) as f64 / capacity as f64) * 100.0
    }
}

fn schedule_interpretation(makespan: u64, critical_path: u64, idle: f64) -> String {
    if makespan == critical_path {
        "dependency-bound: makespan is at the critical-path lower bound".to_owned()
    } else if idle < 5.0 {
        "worker-bound: workers stay mostly occupied under observed dependencies".to_owned()
    } else {
        "mixed dependency/worker-bound: dependencies and idle capacity both matter".to_owned()
    }
}

fn block_weighted_conflict_parts(block: &TracePackBlock) -> Option<(u128, u128)> {
    if !block.has_complete_gas() {
        return None;
    }
    let trace = block.to_block_trace();
    let conflicts = conflict_pairs(&trace);
    let gas = block
        .transactions
        .iter()
        .map(|tx| (tx.tx_index, u128::from(tx.gas_used.unwrap_or(0))))
        .collect::<BTreeMap<_, _>>();
    let conflict_set = conflicts
        .iter()
        .map(|pair| (pair.earlier, pair.later))
        .collect::<BTreeSet<_>>();
    let indices = block
        .transactions
        .iter()
        .map(|tx| tx.tx_index)
        .collect::<Vec<_>>();
    let mut num = 0_u128;
    let mut den = 0_u128;
    for left in 0..indices.len() {
        for right in (left + 1)..indices.len() {
            let pair_weight = gas[&indices[left]] + gas[&indices[right]];
            den += pair_weight;
            if conflict_set.contains(&(indices[left], indices[right])) {
                num += pair_weight;
            }
        }
    }
    Some((num, den))
}

impl HotAccumulator {
    fn from_block(block: &TracePackBlock, conflicts: &[ConflictPair]) -> Self {
        let mut acc = Self::default();
        acc.observe_block(block, conflicts);
        acc
    }

    fn observe_block(&mut self, block: &TracePackBlock, conflicts: &[ConflictPair]) {
        for tx in &block.transactions {
            let tx_id = (block.block_number, tx.tx_index.0);
            let gas = tx.gas_used.unwrap_or(0);
            let mut tx_contracts = BTreeSet::new();
            let mut tx_slots = BTreeSet::new();
            for access in &tx.accesses {
                let address = access.address.to_string();
                tx_contracts.insert(address.clone());
                if let Some(slot) = &access.slot {
                    let key = format!("{}:{slot}", access.address);
                    tx_slots.insert((address.clone(), key));
                }
            }
            for contract in tx_contracts {
                self.contract_txs
                    .entry(contract.clone())
                    .or_default()
                    .insert(tx_id);
                *self.contract_gas.entry(contract).or_insert(0) += gas;
            }
            for (contract, slot_key) in tx_slots {
                self.slot_txs
                    .entry(slot_key.clone())
                    .or_default()
                    .insert(tx_id);
                self.contract_slots
                    .entry(contract)
                    .or_default()
                    .insert(slot_key.clone());
                *self.slot_gas.entry(slot_key).or_insert(0) += gas;
            }
        }
        for conflict in conflicts {
            for key in &conflict.keys {
                let key_string = access_key_string(key);
                *self.key_conflicts.entry(key_string.clone()).or_insert(0) += 1;
                *self
                    .contract_conflicts
                    .entry(key.address().to_string())
                    .or_insert(0) += 1;
                if let Some((address, slot)) = key.storage_slot() {
                    *self
                        .slot_conflicts
                        .entry(format!("{address}:{slot}"))
                        .or_insert(0) += 1;
                }
            }
        }
    }

    fn top_contracts(&self, limit: usize, gas_available: bool) -> Vec<HotContractDossier> {
        let mut items = self
            .contract_txs
            .iter()
            .map(|(address, txs)| HotContractDossier {
                address: address.clone(),
                touching_txs: txs.len(),
                unique_slots: self
                    .contract_slots
                    .get(address)
                    .map(BTreeSet::len)
                    .unwrap_or(0),
                gas_of_touching_txs: gas_available
                    .then_some(*self.contract_gas.get(address).unwrap_or(&0)),
                conflict_contribution: *self.contract_conflicts.get(address).unwrap_or(&0),
            })
            .collect::<Vec<_>>();
        items.sort_by(|left, right| {
            right
                .conflict_contribution
                .cmp(&left.conflict_contribution)
                .then_with(|| right.touching_txs.cmp(&left.touching_txs))
                .then_with(|| left.address.cmp(&right.address))
        });
        items.truncate(limit);
        items
    }

    fn top_slots(&self, limit: usize, gas_available: bool) -> Vec<HotSlotDossier> {
        let mut items = self
            .slot_txs
            .iter()
            .map(|(key, txs)| {
                let (address, slot) = key.split_once(':').unwrap_or((key, ""));
                HotSlotDossier {
                    key: key.clone(),
                    address: address.to_owned(),
                    slot: slot.to_owned(),
                    touching_txs: txs.len(),
                    gas_of_touching_txs: gas_available
                        .then_some(*self.slot_gas.get(key).unwrap_or(&0)),
                    conflict_contribution: *self.slot_conflicts.get(key).unwrap_or(&0),
                }
            })
            .collect::<Vec<_>>();
        items.sort_by(|left, right| {
            right
                .conflict_contribution
                .cmp(&left.conflict_contribution)
                .then_with(|| right.touching_txs.cmp(&left.touching_txs))
                .then_with(|| left.key.cmp(&right.key))
        });
        items.truncate(limit);
        items
    }
}

fn access_key_string(key: &TraceAccessKey) -> String {
    match key {
        TraceAccessKey::Storage { address, slot } => format!("storage:{address}:{slot}"),
        TraceAccessKey::Account { address } => format!("account:{address}"),
        TraceAccessKey::Balance { address } => format!("balance:{address}"),
        TraceAccessKey::Nonce { address } => format!("nonce:{address}"),
        TraceAccessKey::Code { address } => format!("code:{address}"),
    }
}

fn contention_concentration(key_conflicts: &BTreeMap<String, u64>) -> ContentionConcentration {
    let mut counts = key_conflicts.values().copied().collect::<Vec<_>>();
    counts.sort_by(|left, right| right.cmp(left));
    let total = counts.iter().sum::<u64>();
    ContentionConcentration {
        top_1_conflict_percent: top_n_percent(&counts, total, 1),
        top_5_conflict_percent: top_n_percent(&counts, total, 5),
        top_10_conflict_percent: top_n_percent(&counts, total, 10),
    }
}

fn top_n_percent(counts: &[u64], total: u64, n: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        (counts.iter().take(n).sum::<u64>() as f64 / total as f64) * 100.0
    }
}

fn worst_by_conflict(blocks: &[BlockDossier], limit: usize) -> Vec<WorstBlockSummary> {
    let mut out = blocks.iter().map(worst_summary).collect::<Vec<_>>();
    out.sort_by(|left, right| {
        right
            .conflict_percentage
            .total_cmp(&left.conflict_percentage)
            .then_with(|| left.block_number.cmp(&right.block_number))
    });
    out.truncate(limit);
    out
}

fn worst_by_gas_path(blocks: &[BlockDossier], limit: usize) -> Vec<WorstBlockSummary> {
    let mut out = blocks.iter().map(worst_summary).collect::<Vec<_>>();
    out.sort_by(|left, right| {
        right
            .gas_weighted_critical_path
            .unwrap_or(0)
            .cmp(&left.gas_weighted_critical_path.unwrap_or(0))
            .then_with(|| left.block_number.cmp(&right.block_number))
    });
    out.truncate(limit);
    out
}

fn worst_summary(block: &BlockDossier) -> WorstBlockSummary {
    WorstBlockSummary {
        block_number: block.block_number,
        tx_count: block.tx_count,
        conflict_pair_count: block.conflict_pair_count,
        conflict_percentage: block.conflict_percentage,
        gas_weighted_critical_path: block.gas_critical_path,
    }
}

fn normalized_workers(workers: &[usize]) -> Vec<usize> {
    let mut out = workers
        .iter()
        .copied()
        .filter(|workers| *workers > 0)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if !out.contains(&1) {
        out.insert(0, 1);
    }
    out
}

fn percentage(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        (numerator as f64 / denominator as f64) * 100.0
    }
}

fn weighted_percentage(numerator: u128, denominator: u128) -> Option<f64> {
    (denominator > 0).then_some((numerator as f64 / denominator as f64) * 100.0)
}

fn option_u64(value: Option<u64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unavailable".to_owned())
}

fn dossier_hash(dossier: &TracePackDossier) -> String {
    let mut clone = dossier.clone();
    clone.deterministic_hash.clear();
    let bytes = serde_json::to_vec(&clone).expect("dossier serialization is infallible");
    format!("{:016x}", stable_fnv1a64(&bytes))
}

fn recommendation_hash(report: &AccessListRecommendationReport) -> String {
    let mut clone = report.clone();
    clone.deterministic_hash.clear();
    let bytes = serde_json::to_vec(&clone).expect("recommendation serialization is infallible");
    format!("{:016x}", stable_fnv1a64(&bytes))
}

#[cfg(test)]
mod tests {
    use parallel_revm_lab_trace_model::{
        Address, ChainKind, StorageKey, TracePackAccess, TracePackAccessKind, TracePackManifest,
        TracePackTx, TxHash, TRACE_PACK_SCHEMA_VERSION,
    };

    use super::*;

    #[test]
    fn gas_weighted_critical_path_on_known_graph() {
        let pack = demo_pack();
        let dossier = analyze_trace_pack(&pack, &[1, 2, 4]);

        assert_eq!(dossier.tx_count, 4);
        assert_eq!(dossier.conflict_pair_count, 3);
        assert_eq!(dossier.critical_path_length_by_tx, 3);
        assert_eq!(dossier.gas_weighted_critical_path, Some(160));
        assert!((dossier.gas_weighted_conflict_percentage.unwrap() - 59.259).abs() < 0.01);
    }

    #[test]
    fn worker_simulation_known_dag() {
        let pack = demo_pack();
        let dossier = analyze_trace_pack(&pack, &[1, 2, 4]);
        let workers_1 = dossier
            .worker_simulation
            .iter()
            .find(|simulation| simulation.workers == 1)
            .unwrap();
        let workers_2 = dossier
            .worker_simulation
            .iter()
            .find(|simulation| simulation.workers == 2)
            .unwrap();

        assert_eq!(workers_1.makespan, 180);
        assert_eq!(workers_2.makespan, 160);
        assert!(workers_2.speedup_vs_one_worker > 1.0);
    }

    #[test]
    fn worker_simulation_invariants_hold() {
        let pack = demo_pack();
        let dossier = analyze_trace_pack(&pack, &[1, 2, 4, 8]);
        let mut previous = u64::MAX;
        for simulation in &dossier.worker_simulation {
            assert!(simulation.makespan <= previous);
            assert!(simulation.critical_path_bound <= simulation.makespan);
            previous = simulation.makespan;
        }
        assert_eq!(dossier.worker_simulation[0].makespan, 180);
    }

    #[test]
    fn gas_missing_falls_back_to_unit_duration() {
        let mut pack = demo_pack();
        pack.blocks[0].total_gas_used = None;
        pack.blocks[0].transactions[0].gas_used = None;
        let dossier = analyze_trace_pack(&pack, &[1, 2]);

        assert_eq!(dossier.gas_weighted_critical_path, None);
        assert!(dossier
            .warnings
            .iter()
            .any(|warning| warning.contains("gas-weighted")));
        assert_eq!(dossier.worker_simulation[0].makespan, 4);
    }

    #[test]
    fn hot_slot_ranking_is_stable_and_correct() {
        let dossier = analyze_trace_pack(&demo_pack(), &[1, 2]);

        assert_eq!(dossier.top_hot_storage_slots[0].touching_txs, 3);
        assert_eq!(dossier.top_hot_storage_slots[0].conflict_contribution, 3);
        assert_eq!(
            dossier.contention_concentration.top_1_conflict_percent,
            100.0
        );
    }

    #[test]
    fn overlap_coverage_and_worst_txs_are_reported() {
        let mut pack = demo_pack();
        pack.blocks[0].source_tx_count = Some(8);

        let dossier = analyze_trace_pack(&pack, &[1, 2]);

        assert_eq!(dossier.source_tx_count, Some(8));
        assert_eq!(dossier.tx_coverage_percentage, Some(50.0));
        assert_eq!(dossier.overlapping_tx_count, 3);
        assert_eq!(dossier.overlapping_tx_percentage, 75.0);
        assert_eq!(dossier.wave_count, 3);
        assert_eq!(dossier.max_wave_width, 2);
        assert_eq!(dossier.worst_serializing_txs[0].tx_index, 2);
        assert_eq!(dossier.worst_serializing_txs[0].duration_units, 60);
    }

    #[test]
    fn access_hints_are_deterministic_and_render_markdown() {
        let pack = demo_pack();
        let left = recommend_access_lists(&pack);
        let right = recommend_access_lists(&pack);

        assert_eq!(left.deterministic_hash, right.deterministic_hash);
        let markdown = render_access_hints_markdown(&left);
        assert!(markdown.contains("Observed Access Hints"));
        assert!(markdown.contains("not complete production Ethereum access lists"));
    }

    #[test]
    fn tx_warnings_are_preserved_in_dossier() {
        let mut pack = demo_pack();
        pack.blocks[0].transactions[0]
            .warnings
            .push("provider returned truncated trace".to_owned());

        let dossier = analyze_trace_pack(&pack, &[1]);

        assert!(dossier.blocks[0]
            .warnings
            .iter()
            .any(|warning| warning.contains("tx_index 0: provider returned truncated trace")));
        assert!(dossier
            .warnings
            .iter()
            .any(|warning| warning
                .contains("block 1: tx_index 0: provider returned truncated trace")));
        assert!(
            dossier
                .parallelism_loss_decomposition
                .unknown_incomplete_trace_warning_count
                > 0
        );
    }

    fn demo_pack() -> TracePack {
        let contract = Address::canonical("0x1111111111111111111111111111111111111111");
        let slot = StorageKey::canonical(format!("0x{:064x}", 1));
        TracePack {
            manifest: TracePackManifest {
                schema_version: TRACE_PACK_SCHEMA_VERSION.to_owned(),
                chain: ChainKind::new("base"),
                source: "unit-test".to_owned(),
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
                tx_count: 4,
                source_tx_count: Some(4),
                total_gas_used: Some(180),
                transactions: vec![
                    tx(0, 50, &contract, Some(&slot)),
                    tx(1, 20, &contract, None),
                    tx(2, 60, &contract, Some(&slot)),
                    tx(3, 50, &contract, Some(&slot)),
                ],
                warnings: Vec::new(),
            }],
        }
    }

    fn tx(index: u64, gas: u64, contract: &Address, slot: Option<&StorageKey>) -> TracePackTx {
        TracePackTx {
            tx_index: TxIndex(index),
            tx_hash: TxHash(format!("0x{index:064x}")),
            from: Some(Address::canonical(format!("0x{:040x}", index + 1))),
            to: Some(contract.clone()),
            gas_used: Some(gas),
            status: Some("0x1".to_owned()),
            accesses: slot
                .map(|slot| {
                    vec![TracePackAccess {
                        address: contract.clone(),
                        slot: Some(slot.clone()),
                        kind: TracePackAccessKind::ReadWrite,
                        op: Some("SSTORE".to_owned()),
                        pc: Some(index),
                        depth: Some(1),
                        gas_remaining: Some(1000 - gas),
                    }]
                })
                .unwrap_or_default(),
            warnings: Vec::new(),
        }
    }
}
