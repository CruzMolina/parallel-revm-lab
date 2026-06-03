use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use parallel_revm_lab_analyzer::{
    analyze_block_trace, analyze_trace_pack as analyze_trace_pack_report, apply_contract_labels,
    dossier_schedule_trace_json, recommend_access_lists, render_access_hints_markdown,
    render_dossier_html, render_dossier_markdown, render_hot_contracts_csv, render_hot_slots_csv,
    render_html_with_command, render_markdown, render_scheduler_ablation_csv,
    render_worker_simulation_csv, schedule_trace_json,
};
use parallel_revm_lab_executor::{
    benchmark_report, run_access_list, run_optimistic, run_sequential, trace_json, BenchmarkReport,
    ExecutionMode,
};
use parallel_revm_lab_model::{AccessKey, AccountId, ContractId, SlotId, State, Tx, TxKind};
use parallel_revm_lab_trace_model::{
    Address, BlockAccessTrace, ChainKind, TracePack, TracePackAccess, TracePackAccessKind,
    TracePackBlock, TracePackManifest, TracePackTx, TxHash, TxIndex, TRACE_PACK_SCHEMA_VERSION,
};
use parallel_revm_lab_workload::{
    count_conflict_pairs, generate_workload, Workload, WorkloadConfig, WorkloadKind,
};

#[derive(Debug, Parser)]
#[command(
    name = "parallel-revm-lab",
    version,
    about = "Deterministic parallel execution and EVM trace analysis workbench"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Bench(BenchArgs),
    BenchTracePack(BenchTracePackArgs),
    Verify(VerifyArgs),
    Inspect(InspectArgs),
    AnalyzeFixture(AnalyzeFixtureArgs),
    AnalyzeTrace(AnalyzeTraceArgs),
    AnalyzeTracePack(AnalyzeTracePackArgs),
    RecommendAccessLists(RecommendAccessListsArgs),
    RpcCapabilityCheck(RpcCapabilityCheckArgs),
    CollectBlockRange(CollectBlockRangeArgs),
    #[command(hide = true)]
    AnalyzeBlock(AnalyzeBlockArgs),
}

#[derive(Debug, Args)]
struct BenchArgs {
    #[arg(long, value_parser = parse_workload, default_value = "mixed")]
    workload: WorkloadKind,
    #[arg(long, default_value_t = 1_000)]
    txs: usize,
    #[arg(long, value_parser = parse_conflict, default_value_t = 0.5)]
    conflict: f64,
    #[arg(long, value_parser = parse_mode, default_value = "all")]
    mode: ExecutionMode,
    #[arg(long, default_value_t = 4)]
    threads: usize,
    #[arg(long, default_value_t = 42)]
    seed: u64,
    #[arg(long, default_value_t = 0)]
    vm_steps: u64,
    #[arg(long)]
    out: PathBuf,
    #[arg(long)]
    trace: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct BenchTracePackArgs {
    #[arg(long)]
    trace_dir: PathBuf,
    #[arg(long, value_parser = parse_mode, default_value = "all")]
    mode: ExecutionMode,
    #[arg(long, default_value = "1,2,4,8,16")]
    threads: String,
    #[arg(long, default_value_t = 0)]
    vm_steps_per_gas: u64,
    #[arg(long)]
    out: PathBuf,
}

#[derive(Debug, Args)]
struct VerifyArgs {
    #[arg(long, value_parser = parse_workload, default_value = "mixed")]
    workload: WorkloadKind,
    #[arg(long, default_value_t = 100)]
    txs: usize,
    #[arg(long, default_value = "0.0,0.5")]
    conflicts: String,
    #[arg(long, default_value = "1,2")]
    threads: String,
    #[arg(long, default_value_t = 1)]
    seed_start: u64,
    #[arg(long, default_value_t = 5)]
    seed_end: u64,
    #[arg(long, default_value_t = 0)]
    vm_steps: u64,
}

#[derive(Debug, Args)]
struct InspectArgs {
    #[arg(long, value_parser = parse_workload, default_value = "mixed")]
    workload: WorkloadKind,
    #[arg(long, default_value_t = 20)]
    txs: usize,
    #[arg(long, value_parser = parse_conflict, default_value_t = 0.5)]
    conflict: f64,
    #[arg(long, default_value_t = 42)]
    seed: u64,
    #[arg(long, default_value_t = 0)]
    vm_steps: u64,
}

#[derive(Debug, Args)]
struct AnalyzeFixtureArgs {
    #[arg(long)]
    fixture: PathBuf,
    #[arg(long)]
    out: PathBuf,
    #[arg(long)]
    markdown: PathBuf,
    #[arg(long)]
    html: PathBuf,
    #[arg(long)]
    trace: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum TraceFormat {
    GethStructLogs,
}

impl std::fmt::Display for TraceFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraceFormat::GethStructLogs => write!(f, "geth-struct-logs"),
        }
    }
}

#[derive(Debug, Args)]
struct AnalyzeTraceArgs {
    #[arg(long, value_enum)]
    format: TraceFormat,
    #[arg(long)]
    fixture: PathBuf,
    #[arg(long)]
    out: PathBuf,
    #[arg(long)]
    markdown: PathBuf,
    #[arg(long)]
    html: PathBuf,
    #[arg(long)]
    trace: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct AnalyzeTracePackArgs {
    #[arg(long)]
    trace_dir: PathBuf,
    #[arg(long, default_value = "1,2,4,8")]
    workers: String,
    #[arg(long)]
    out: PathBuf,
    #[arg(long)]
    markdown: PathBuf,
    #[arg(long)]
    html: PathBuf,
    #[arg(long)]
    trace: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct RecommendAccessListsArgs {
    #[arg(long)]
    trace_dir: PathBuf,
    #[arg(long)]
    out: PathBuf,
    #[arg(long)]
    markdown: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct RpcCapabilityCheckArgs {
    #[arg(long)]
    chain: String,
    #[arg(long)]
    block: u64,
    #[arg(long)]
    rpc_url: Option<String>,
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum CollectorTracer {
    GethJsStorage,
}

impl std::fmt::Display for CollectorTracer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectorTracer::GethJsStorage => write!(f, "geth-js-storage"),
        }
    }
}

#[derive(Debug, Args)]
struct CollectBlockRangeArgs {
    #[arg(long)]
    chain: String,
    #[arg(long)]
    start_block: u64,
    #[arg(long)]
    end_block: u64,
    #[arg(long)]
    rpc_url: Option<String>,
    #[arg(long, value_enum, default_value = "geth-js-storage")]
    tracer: CollectorTracer,
    #[arg(long)]
    out: PathBuf,
    #[arg(long)]
    max_transactions: Option<usize>,
    #[arg(long)]
    resume: bool,
    #[arg(long)]
    dry_run: bool,
}

#[derive(Debug, Args)]
struct AnalyzeBlockArgs {
    #[arg(long)]
    chain: String,
    #[arg(long)]
    block: u64,
    #[arg(long)]
    rpc_url: Option<String>,
    #[arg(long)]
    out: PathBuf,
    #[arg(long)]
    markdown: PathBuf,
    #[arg(long)]
    html: PathBuf,
}

#[derive(Debug, serde::Serialize)]
struct RpcCapabilityReport {
    report_version: String,
    chain: String,
    block: u64,
    block_available: bool,
    block_hash: Option<String>,
    tx_count: usize,
    sample_tx_hash: Option<String>,
    receipt_available: bool,
    debug_trace_transaction_available: bool,
    custom_js_tracer_available: bool,
    struct_logs_available: bool,
    status: String,
    failures: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
struct TracePackBenchmarkReport {
    report_version: String,
    trace_dir: String,
    chain: String,
    start_block: u64,
    end_block: u64,
    tx_count: usize,
    source_tx_count: Option<usize>,
    tx_coverage_percentage: Option<f64>,
    observed_conflict: f64,
    conflict_pairs: u64,
    vm_steps_per_gas: u64,
    access_topology: String,
    caveats: Vec<String>,
    runs: Vec<BenchmarkReport>,
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Commands::Bench(args) => bench(args),
        Commands::BenchTracePack(args) => bench_trace_pack(args),
        Commands::Verify(args) => verify(args),
        Commands::Inspect(args) => inspect(args),
        Commands::AnalyzeFixture(args) => analyze_fixture(args),
        Commands::AnalyzeTrace(args) => analyze_trace(args),
        Commands::AnalyzeTracePack(args) => analyze_trace_pack(args),
        Commands::RecommendAccessLists(args) => recommend_access_lists_command(args),
        Commands::RpcCapabilityCheck(args) => rpc_capability_check(args),
        Commands::CollectBlockRange(args) => collect_block_range(args),
        Commands::AnalyzeBlock(args) => analyze_block(args),
    }
}

fn bench(args: BenchArgs) -> Result<()> {
    let mut config = WorkloadConfig::new(args.workload, args.txs, args.conflict, args.seed);
    config.vm_steps = args.vm_steps;
    let workload = generate_workload(config);
    let report = benchmark_report(
        &workload,
        args.mode,
        args.threads,
        rust_version().filter(|version| !version.is_empty()),
    )?;

    if report.modes.iter().any(|mode| !mode.deterministic_passed) {
        bail!("parallel execution produced a state mismatch; report was not written");
    }

    write_json(&args.out, &report)?;
    if let Some(trace_path) = args.trace {
        write_json(&trace_path, &trace_json(&report))?;
    }

    println!(
        "wrote {} mode(s) for {} txs to {}",
        report.modes.len(),
        report.tx_count,
        args.out.display()
    );
    for mode in &report.modes {
        println!(
            "{} hash={} tx/s={:.2} speedup={}",
            mode.mode,
            mode.state_hash,
            mode.tx_per_sec,
            mode.speedup_vs_sequential
                .map(|value| format!("{value:.3}x"))
                .unwrap_or_else(|| "baseline".to_owned())
        );
    }
    Ok(())
}

fn bench_trace_pack(args: BenchTracePackArgs) -> Result<()> {
    let pack = TracePack::load_dir(&args.trace_dir)
        .with_context(|| format!("failed to load trace pack {}", args.trace_dir.display()))?;
    let workers = parse_thread_list(&args.threads)?;
    let workload = trace_pack_to_workload(&pack, args.vm_steps_per_gas);
    let mut runs = Vec::new();
    for threads in workers {
        let report = benchmark_report(
            &workload,
            args.mode,
            threads,
            rust_version().filter(|version| !version.is_empty()),
        )?;
        if report.modes.iter().any(|mode| !mode.deterministic_passed) {
            bail!(
                "trace-derived benchmark produced a state mismatch at {} thread(s); report was not written",
                threads
            );
        }
        runs.push(report);
    }

    let source_tx_count = trace_pack_source_tx_count(&pack);
    let tx_count = workload.txs.len();
    let report = TracePackBenchmarkReport {
        report_version: "trace-derived-benchmark-v1".to_owned(),
        trace_dir: args.trace_dir.display().to_string(),
        chain: pack.manifest.chain.to_string(),
        start_block: pack.manifest.start_block,
        end_block: pack.manifest.end_block,
        tx_count,
        source_tx_count,
        tx_coverage_percentage: source_tx_count
            .map(|source| percentage(tx_count as u64, source as u64)),
        observed_conflict: workload.observed_conflict,
        conflict_pairs: workload.conflict_pairs,
        vm_steps_per_gas: args.vm_steps_per_gas,
        access_topology: "observed trace-pack access keys mapped to deterministic synthetic model keys".to_owned(),
        caveats: vec![
            "synthetic execution benchmark; not EVM replay and not production TPS/Ggas/s".to_owned(),
            "access-list scheduling uses observed trace-pack read/write sets; toy transactions do not reproduce contract semantics".to_owned(),
            "optimistic mode validation is over the toy execution outcome, not full trace-derived EVM read validation".to_owned(),
        ],
        runs,
    };
    write_json(&args.out, &report)?;
    println!(
        "wrote trace-derived benchmark for {} txs from {} to {}",
        report.tx_count,
        args.trace_dir.display(),
        args.out.display()
    );
    println!(
        "conflicts={} observed_conflict={:.6} runs={}",
        report.conflict_pairs,
        report.observed_conflict,
        report.runs.len()
    );
    Ok(())
}

fn verify(args: VerifyArgs) -> Result<()> {
    if args.seed_start > args.seed_end {
        bail!("--seed-start must be <= --seed-end");
    }

    let conflicts = parse_conflict_list(&args.conflicts)?;
    let thread_counts = parse_thread_list(&args.threads)?;
    let mut checked = 0_u64;

    for seed in args.seed_start..=args.seed_end {
        for conflict in &conflicts {
            let mut config = WorkloadConfig::new(args.workload, args.txs, *conflict, seed);
            config.vm_steps = args.vm_steps;
            let workload = generate_workload(config);
            let sequential = run_sequential(&workload.initial_state, &workload.txs);
            for threads in &thread_counts {
                let access = run_access_list(&workload.initial_state, &workload.txs, *threads)?;
                if access.final_state != sequential.final_state {
                    bail!(
                        "access-list mismatch workload={} seed={} conflict={} threads={} sequential={} access-list={}",
                        args.workload,
                        seed,
                        conflict,
                        threads,
                        sequential.state_hash,
                        access.state_hash
                    );
                }

                let optimistic = run_optimistic(&workload.initial_state, &workload.txs, *threads)?;
                if optimistic.final_state != sequential.final_state {
                    bail!(
                        "optimistic mismatch workload={} seed={} conflict={} threads={} sequential={} optimistic={}",
                        args.workload,
                        seed,
                        conflict,
                        threads,
                        sequential.state_hash,
                        optimistic.state_hash
                    );
                }
                checked += 1;
            }
        }
    }

    println!(
        "verified {} workload/thread combinations for workload={} txs={} seeds={}..={}",
        checked, args.workload, args.txs, args.seed_start, args.seed_end
    );
    Ok(())
}

fn inspect(args: InspectArgs) -> Result<()> {
    let mut config = WorkloadConfig::new(args.workload, args.txs, args.conflict, args.seed);
    config.vm_steps = args.vm_steps;
    let workload = generate_workload(config);
    println!("workload: {}", workload.config.kind);
    println!("txs: {}", workload.txs.len());
    println!("seed: {}", workload.config.seed);
    println!("vm_steps: {}", workload.config.vm_steps);
    println!(
        "requested_conflict: {:.3}",
        workload.config.requested_conflict
    );
    println!("observed_conflict: {:.6}", workload.observed_conflict);
    println!(
        "initial_state_hash: {}",
        workload.initial_state.state_hash()
    );
    println!("sample:");
    for tx in workload.txs.iter().take(10) {
        println!("tx {} {:?}", tx.id, tx.kind);
        println!("  reads: {}", format_accesses(&tx.declared_reads));
        println!("  writes: {}", format_accesses(&tx.declared_writes));
    }
    Ok(())
}

fn analyze_fixture(args: AnalyzeFixtureArgs) -> Result<()> {
    let trace = BlockAccessTrace::from_fixture_path(&args.fixture)
        .with_context(|| format!("failed to load fixture {}", args.fixture.display()))?;
    let report = analyze_block_trace(&trace, true, false);
    let command = analyze_fixture_command(&args);
    write_json(&args.out, &report)?;
    write_text(&args.markdown, &render_markdown(&report))?;
    write_text(
        &args.html,
        &render_html_with_command(&report, Some(&command)),
    )?;
    if let Some(trace_path) = args.trace {
        write_json(&trace_path, &schedule_trace_json(&report))?;
    }
    println!(
        "analyzed {} txs from {} into {}",
        report.tx_count,
        args.fixture.display(),
        args.out.display()
    );
    println!(
        "conflicts={} waves={} max_width={} ceiling={:.3}x hash={}",
        report.conflict_pair_count,
        report.wave_count,
        report.max_wave_width,
        report.theoretical_parallelism_ceiling,
        report.deterministic_hash
    );
    Ok(())
}

fn analyze_trace(args: AnalyzeTraceArgs) -> Result<()> {
    let trace = match args.format {
        TraceFormat::GethStructLogs => BlockAccessTrace::from_geth_struct_logs_path(&args.fixture)
            .with_context(|| {
                format!(
                    "failed to load {} trace fixture {}",
                    args.format,
                    args.fixture.display()
                )
            })?,
    };
    let report = analyze_block_trace(&trace, true, false);
    let command = analyze_trace_command(&args);
    write_json(&args.out, &report)?;
    write_text(&args.markdown, &render_markdown(&report))?;
    write_text(
        &args.html,
        &render_html_with_command(&report, Some(&command)),
    )?;
    if let Some(trace_path) = args.trace {
        write_json(&trace_path, &schedule_trace_json(&report))?;
    }
    println!(
        "analyzed {} {} txs from {} into {}",
        args.format,
        report.tx_count,
        args.fixture.display(),
        args.out.display()
    );
    println!(
        "conflicts={} waves={} max_width={} ceiling={:.3}x hash={}",
        report.conflict_pair_count,
        report.wave_count,
        report.max_wave_width,
        report.theoretical_parallelism_ceiling,
        report.deterministic_hash
    );
    Ok(())
}

fn analyze_trace_pack(args: AnalyzeTracePackArgs) -> Result<()> {
    let pack = TracePack::load_dir(&args.trace_dir)
        .with_context(|| format!("failed to load trace pack {}", args.trace_dir.display()))?;
    let workers = parse_thread_list(&args.workers)?;
    let mut dossier = analyze_trace_pack_report(&pack, &workers);
    let labels = load_optional_contract_labels(Path::new("labels/base-known-contracts.json"))?;
    apply_contract_labels(&mut dossier, &labels);
    let command = analyze_trace_pack_command(&args);
    write_json(&args.out, &dossier)?;
    write_text(&args.markdown, &render_dossier_markdown(&dossier))?;
    write_text(&args.html, &render_dossier_html(&dossier, &command))?;
    if let Some(trace_path) = args.trace {
        write_json(&trace_path, &dossier_schedule_trace_json(&dossier))?;
    }
    if let Some(parent) = args.out.parent() {
        write_text(
            &parent.join("hot-contracts.csv"),
            &render_hot_contracts_csv(&dossier),
        )?;
        write_text(
            &parent.join("hot-slots.csv"),
            &render_hot_slots_csv(&dossier),
        )?;
        write_text(
            &parent.join("worker-simulation.csv"),
            &render_worker_simulation_csv(&dossier),
        )?;
        write_text(
            &parent.join("scheduler-ablation.csv"),
            &render_scheduler_ablation_csv(&dossier),
        )?;
    }
    println!(
        "analyzed trace pack {} into {}",
        args.trace_dir.display(),
        args.out.display()
    );
    println!(
        "blocks={} txs={} conflicts={} ({:.3}%) tx_ceiling={:.3}x gas_ceiling={}",
        dossier.block_count,
        dossier.tx_count,
        dossier.conflict_pair_count,
        dossier.conflict_percentage,
        dossier.theoretical_parallelism_ceiling_by_tx,
        dossier
            .theoretical_parallelism_ceiling_by_gas
            .map(|value| format!("{value:.3}x"))
            .unwrap_or_else(|| "unavailable".to_owned())
    );
    Ok(())
}

fn recommend_access_lists_command(args: RecommendAccessListsArgs) -> Result<()> {
    let pack = TracePack::load_dir(&args.trace_dir)
        .with_context(|| format!("failed to load trace pack {}", args.trace_dir.display()))?;
    let report = recommend_access_lists(&pack);
    write_json(&args.out, &report)?;
    if let Some(markdown) = args.markdown {
        write_text(&markdown, &render_access_hints_markdown(&report))?;
    }
    println!(
        "wrote {} observed access hint(s) to {}",
        report.tx_hints.len(),
        args.out.display()
    );
    Ok(())
}

fn rpc_capability_check(args: RpcCapabilityCheckArgs) -> Result<()> {
    let rpc_url = resolve_rpc_url(&args.chain, args.rpc_url).map_err(|err| {
        anyhow!(
            "{}",
            redact_rpc_url(&err.to_string()).replace(
                "missing RPC URL for collect-block-range",
                "missing RPC URL for rpc-capability-check"
            )
        )
    })?;
    let mut report = RpcCapabilityReport {
        report_version: "rpc-capability-v1".to_owned(),
        chain: args.chain.clone(),
        block: args.block,
        block_available: false,
        block_hash: None,
        tx_count: 0,
        sample_tx_hash: None,
        receipt_available: false,
        debug_trace_transaction_available: false,
        custom_js_tracer_available: false,
        struct_logs_available: false,
        status: "unsupported".to_owned(),
        failures: Vec::new(),
    };

    match rpc_call(
        &rpc_url,
        "eth_getBlockByNumber",
        serde_json::json!([hex_u64(args.block), true]),
    ) {
        Ok(block) if !block.is_null() => {
            report.block_available = true;
            report.block_hash = value_string(block.get("hash"));
            let tx_values = block
                .get("transactions")
                .and_then(serde_json::Value::as_array)
                .cloned()
                .unwrap_or_default();
            report.tx_count = tx_values.len();
            report.sample_tx_hash = tx_values.iter().find_map(|tx| value_string(tx.get("hash")));
        }
        Ok(_) => report
            .failures
            .push(format!("block {} returned null", args.block)),
        Err(err) => report
            .failures
            .push(capability_failure("eth_getBlockByNumber", &err)),
    }

    if let Some(tx_hash) = &report.sample_tx_hash {
        match rpc_call(
            &rpc_url,
            "eth_getTransactionReceipt",
            serde_json::json!([tx_hash]),
        ) {
            Ok(receipt) if !receipt.is_null() => {
                report.receipt_available = true;
            }
            Ok(_) => report
                .failures
                .push("eth_getTransactionReceipt returned null".to_owned()),
            Err(err) => report
                .failures
                .push(capability_failure("eth_getTransactionReceipt", &err)),
        }

        match rpc_call(
            &rpc_url,
            "debug_traceTransaction",
            serde_json::json!([
                tx_hash,
                {
                    "tracer": "{ step: function(log, db) {}, fault: function(log, db) {}, result: function(ctx, db) { return { ok: true }; } }",
                    "timeout": "20s"
                }
            ]),
        ) {
            Ok(_) => {
                report.debug_trace_transaction_available = true;
                report.custom_js_tracer_available = true;
            }
            Err(err) => report
                .failures
                .push(capability_failure("custom JS debug_traceTransaction", &err)),
        }

        match rpc_call(
            &rpc_url,
            "debug_traceTransaction",
            serde_json::json!([
                tx_hash,
                {
                    "disableMemory": true,
                    "disableStorage": true,
                    "disableStack": false,
                    "timeout": "20s"
                }
            ]),
        ) {
            Ok(trace) => {
                report.debug_trace_transaction_available = true;
                report.struct_logs_available = trace.get("structLogs").is_some();
                if !report.struct_logs_available {
                    report
                        .failures
                        .push("debug_traceTransaction returned no structLogs field".to_owned());
                }
            }
            Err(err) => report.failures.push(capability_failure(
                "structLogs debug_traceTransaction",
                &err,
            )),
        }
    } else if report.block_available {
        report
            .failures
            .push("block has no transactions to trace".to_owned());
    }

    report.status = if report.block_available
        && report.tx_count > 0
        && report.receipt_available
        && report.custom_js_tracer_available
    {
        "ok".to_owned()
    } else {
        "unsupported".to_owned()
    };

    if let Some(path) = args.out {
        write_json(&path, &report)?;
    }
    println!(
        "rpc capability {} block {} status={} txs={} receipts={} js_tracer={} struct_logs={}",
        report.chain,
        report.block,
        report.status,
        report.tx_count,
        report.receipt_available,
        report.custom_js_tracer_available,
        report.struct_logs_available
    );
    for failure in &report.failures {
        println!("capability warning: {failure}");
    }
    Ok(())
}

fn collect_block_range(args: CollectBlockRangeArgs) -> Result<()> {
    if args.start_block > args.end_block {
        bail!("--start-block must be <= --end-block");
    }
    let rpc_url = resolve_rpc_url(&args.chain, args.rpc_url)?;
    if args.dry_run {
        for block_number in block_numbers(args.start_block, args.end_block) {
            let block = rpc_call(
                &rpc_url,
                "eth_getBlockByNumber",
                serde_json::json!([hex_u64(block_number), false]),
            )?;
            if block.is_null() {
                bail!(
                    "dry-run {} block {} returned null",
                    args.chain,
                    block_number
                );
            }
            let tx_count = block
                .get("transactions")
                .and_then(serde_json::Value::as_array)
                .map(Vec::len)
                .unwrap_or(0);
            println!(
                "dry-run {} block {} hash={} txs={}",
                args.chain,
                block_number,
                value_string(block.get("hash")).unwrap_or_else(|| "unknown".to_owned()),
                tx_count
            );
        }
        return Ok(());
    }

    let per_block_limit = args.max_transactions.unwrap_or(usize::MAX);
    let mut blocks = Vec::new();
    for block_number in block_numbers(args.start_block, args.end_block) {
        let block_path = trace_pack_block_path(&args.out, block_number);
        if args.resume && block_path.exists() {
            blocks.push(read_resumed_block(
                &block_path,
                block_number,
                ChainKind::new(&args.chain),
            )?);
            println!("resume: kept existing block file for {block_number}");
            continue;
        }

        let block_value = rpc_call(
            &rpc_url,
            "eth_getBlockByNumber",
            serde_json::json!([hex_u64(block_number), true]),
        )?;
        if block_value.is_null() {
            bail!("eth_getBlockByNumber returned null for block {block_number}");
        }
        let tx_values = block_value
            .get("transactions")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| anyhow!("eth_getBlockByNumber returned no transactions array"))?;
        let take = tx_values.len().min(per_block_limit);
        let mut transactions = Vec::new();
        let mut block_warnings = Vec::new();
        if take < tx_values.len() {
            block_warnings.push(format!(
                "collection truncated to {take} of {} transactions by --max-transactions",
                tx_values.len()
            ));
        }
        for (idx, tx_value) in tx_values.iter().take(take).enumerate() {
            let tx_hash = required_string(tx_value, "hash")?;
            let from = value_string(tx_value.get("from")).map(Address::canonical);
            let to = value_string(tx_value.get("to")).map(Address::canonical);
            let mut tx_warnings = Vec::new();
            let receipt = match rpc_call(
                &rpc_url,
                "eth_getTransactionReceipt",
                serde_json::json!([tx_hash]),
            ) {
                Ok(value) => Some(value),
                Err(err) => {
                    tx_warnings.push(format!(
                        "receipt unavailable; gas/status omitted: {}",
                        redact_rpc_url(&err.to_string())
                    ));
                    None
                }
            };
            let gas_used = receipt
                .as_ref()
                .and_then(|value| value.get("gasUsed"))
                .and_then(serde_json::Value::as_str)
                .and_then(parse_hex_u64);
            let status = receipt
                .as_ref()
                .and_then(|value| value.get("status"))
                .and_then(serde_json::Value::as_str)
                .map(ToOwned::to_owned);
            let trace = rpc_call(
                &rpc_url,
                "debug_traceTransaction",
                serde_json::json!([
                    tx_hash,
                    {
                        "tracer": geth_storage_access_tracer_source(),
                        "timeout": "20s"
                    }
                ]),
            )
            .with_context(|| {
                format!(
                    "debug_traceTransaction failed for block {block_number} tx_index {idx}; provider may not support debug tracing or JavaScript tracers"
                )
            })?;
            let mut accesses = trace_accesses_from_rpc(&trace)?;
            accesses.sort();
            accesses.dedup();
            tx_warnings.extend(trace_warnings_from_rpc(&trace));
            transactions.push(TracePackTx {
                tx_index: TxIndex(idx as u64),
                tx_hash: TxHash(tx_hash.to_owned()),
                from,
                to,
                gas_used,
                status,
                accesses,
                warnings: tx_warnings,
            });
        }
        let total_gas_used = transactions
            .iter()
            .map(|tx| tx.gas_used)
            .collect::<Option<Vec<_>>>()
            .map(|gas| gas.into_iter().sum());
        let block = TracePackBlock {
            chain: ChainKind::new(&args.chain),
            block_number,
            block_hash: value_string(block_value.get("hash")),
            parent_hash: value_string(block_value.get("parentHash")),
            tx_count: transactions.len(),
            source_tx_count: Some(tx_values.len()),
            total_gas_used,
            transactions,
            warnings: block_warnings,
        };
        persist_trace_pack_block(&args.out, &block)?;
        blocks.push(block);
        println!(
            "collected {} transaction(s) for {} block {}",
            take, args.chain, block_number
        );
    }

    let mut pack = TracePack {
        manifest: TracePackManifest {
            schema_version: TRACE_PACK_SCHEMA_VERSION.to_owned(),
            chain: ChainKind::new(&args.chain),
            source: "rpc-debug_traceTransaction".to_owned(),
            provenance: "user-collected RPC trace pack".to_owned(),
            start_block: args.start_block,
            end_block: args.end_block,
            created_by_tool_version: env!("CARGO_PKG_VERSION").to_owned(),
            tracer_kind: args.tracer.to_string(),
            notes: vec![format!(
                "Collected with parallel-revm-lab collect-block-range over {} blocks {}-{}",
                args.chain, args.start_block, args.end_block
            )],
            warnings: vec![
                "Provider trace completeness varies; verify tracer support before making claims"
                    .to_owned(),
            ],
        },
        blocks,
    };
    pack.normalize();
    pack.validate()?;
    pack.write_dir(&args.out)?;
    println!(
        "wrote trace pack for {} blocks {}-{} to {}",
        args.chain,
        args.start_block,
        args.end_block,
        args.out.display()
    );
    Ok(())
}

fn block_numbers(start: u64, end: u64) -> std::ops::RangeInclusive<u64> {
    start..=end
}

fn trace_pack_block_path(root: &Path, block_number: u64) -> PathBuf {
    root.join("blocks").join(format!("{block_number}.json"))
}

fn read_resumed_block(
    path: &Path,
    expected_block_number: u64,
    expected_chain: ChainKind,
) -> Result<TracePackBlock> {
    let mut block: TracePackBlock = read_json(path)?;
    block.normalize();
    block
        .validate()
        .with_context(|| format!("invalid resumed block file {}", path.display()))?;
    if block.block_number != expected_block_number {
        bail!(
            "resumed block file {} contains block {}, expected {}",
            path.display(),
            block.block_number,
            expected_block_number
        );
    }
    if block.chain != expected_chain {
        bail!(
            "resumed block file {} has chain {}, expected {}",
            path.display(),
            block.chain,
            expected_chain
        );
    }
    Ok(block)
}

fn persist_trace_pack_block(root: &Path, block: &TracePackBlock) -> Result<()> {
    let mut block = block.clone();
    block.normalize();
    block.validate()?;
    write_json(&trace_pack_block_path(root, block.block_number), &block)
}

fn analyze_fixture_command(args: &AnalyzeFixtureArgs) -> String {
    let mut command = format!(
        "cargo run -p parallel-revm-lab -- analyze-fixture --fixture {} --out {} --markdown {} --html {}",
        args.fixture.display(),
        args.out.display(),
        args.markdown.display(),
        args.html.display()
    );
    if let Some(trace) = &args.trace {
        command.push_str(&format!(" --trace {}", trace.display()));
    }
    command
}

fn analyze_trace_command(args: &AnalyzeTraceArgs) -> String {
    let mut command = format!(
        "cargo run -p parallel-revm-lab -- analyze-trace --format {} --fixture {} --out {} --markdown {} --html {}",
        args.format,
        args.fixture.display(),
        args.out.display(),
        args.markdown.display(),
        args.html.display()
    );
    if let Some(trace) = &args.trace {
        command.push_str(&format!(" --trace {}", trace.display()));
    }
    command
}

fn analyze_trace_pack_command(args: &AnalyzeTracePackArgs) -> String {
    let mut command = format!(
        "cargo run -p parallel-revm-lab -- analyze-trace-pack --trace-dir {} --workers {} --out {} --markdown {} --html {}",
        args.trace_dir.display(),
        args.workers,
        args.out.display(),
        args.markdown.display(),
        args.html.display()
    );
    if let Some(trace) = &args.trace {
        command.push_str(&format!(" --trace {}", trace.display()));
    }
    command
}

fn trace_pack_to_workload(pack: &TracePack, vm_steps_per_gas: u64) -> Workload {
    let mut contracts = std::collections::BTreeSet::<String>::new();
    let mut slots = std::collections::BTreeSet::<(String, String)>::new();
    let mut accounts = std::collections::BTreeSet::<String>::new();
    for block in &pack.blocks {
        for tx in &block.transactions {
            for access in &tx.accesses {
                let address = access.address.to_string();
                if let Some(slot) = &access.slot {
                    contracts.insert(address.clone());
                    slots.insert((address, slot.to_string()));
                } else {
                    accounts.insert(address);
                }
            }
        }
    }

    let contract_ids = contracts
        .into_iter()
        .enumerate()
        .map(|(index, address)| (address, ContractId(index as u64)))
        .collect::<std::collections::BTreeMap<_, _>>();
    let account_ids = accounts
        .into_iter()
        .enumerate()
        .map(|(index, address)| (address, AccountId(index as u64)))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut next_slot_by_contract = std::collections::BTreeMap::<String, u64>::new();
    let mut slot_ids = std::collections::BTreeMap::<(String, String), SlotId>::new();
    for (address, slot) in slots {
        let next = next_slot_by_contract.entry(address.clone()).or_insert(0);
        slot_ids.insert((address, slot), SlotId(*next));
        *next += 1;
    }

    let mut txs = Vec::new();
    for block in &pack.blocks {
        for tx in &block.transactions {
            let mut declared_reads = std::collections::BTreeSet::new();
            let mut declared_writes = std::collections::BTreeSet::new();
            for access in &tx.accesses {
                let key =
                    trace_pack_access_to_model_key(access, &contract_ids, &slot_ids, &account_ids);
                match access.kind {
                    TracePackAccessKind::Read | TracePackAccessKind::Call => {
                        declared_reads.insert(key);
                    }
                    TracePackAccessKind::Write
                    | TracePackAccessKind::Create
                    | TracePackAccessKind::Selfdestruct => {
                        declared_writes.insert(key);
                    }
                    TracePackAccessKind::ReadWrite | TracePackAccessKind::AccountTouch => {
                        declared_reads.insert(key.clone());
                        declared_writes.insert(key);
                    }
                }
            }
            txs.push(Tx {
                id: tx.tx_index.0,
                kind: TxKind::Noop,
                vm_steps: tx
                    .gas_used
                    .unwrap_or(1)
                    .max(1)
                    .saturating_mul(vm_steps_per_gas),
                declared_reads,
                declared_writes,
            });
        }
    }

    let conflict_pairs = count_conflict_pairs(&txs);
    let possible_pairs = pair_count(txs.len());
    let observed_conflict = if possible_pairs == 0 {
        0.0
    } else {
        conflict_pairs as f64 / possible_pairs as f64
    };
    let config = WorkloadConfig {
        kind: WorkloadKind::Mixed,
        tx_count: txs.len(),
        requested_conflict: observed_conflict,
        vm_steps: vm_steps_per_gas,
        accounts: account_ids.len().max(2) as u64,
        contracts: contract_ids.len().max(1) as u64,
        hot_slots: slot_ids.len().max(1) as u64,
        seed: 0,
    };
    Workload {
        config,
        initial_state: State::new(),
        txs,
        conflict_pairs,
        observed_conflict,
    }
}

fn trace_pack_access_to_model_key(
    access: &TracePackAccess,
    contract_ids: &std::collections::BTreeMap<String, ContractId>,
    slot_ids: &std::collections::BTreeMap<(String, String), SlotId>,
    account_ids: &std::collections::BTreeMap<String, AccountId>,
) -> AccessKey {
    let address = access.address.to_string();
    if let Some(slot) = &access.slot {
        let contract = *contract_ids
            .get(&address)
            .expect("contract ids are precomputed from all storage accesses");
        let slot = *slot_ids
            .get(&(address, slot.to_string()))
            .expect("slot ids are precomputed from all storage accesses");
        AccessKey::storage(contract, slot)
    } else {
        let account = *account_ids
            .get(&address)
            .expect("account ids are precomputed from all non-storage accesses");
        AccessKey::account_balance(account)
    }
}

fn trace_pack_source_tx_count(pack: &TracePack) -> Option<usize> {
    let mut total = 0_usize;
    let mut present = false;
    for block in &pack.blocks {
        if let Some(source) = block.source_tx_count {
            total += source;
            present = true;
        } else {
            total += block.transactions.len();
        }
    }
    present.then_some(total)
}

fn percentage(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        (numerator as f64 / denominator as f64) * 100.0
    }
}

fn pair_count(len: usize) -> u64 {
    if len < 2 {
        0
    } else {
        (len as u64 * (len as u64 - 1)) / 2
    }
}

fn analyze_block(args: AnalyzeBlockArgs) -> Result<()> {
    let env_key = match args.chain.as_str() {
        "base" => "BASE_RPC_URL",
        _ => "ETH_RPC_URL",
    };
    let rpc_url = args
        .rpc_url
        .or_else(|| std::env::var(env_key).ok())
        .or_else(|| std::env::var("ETH_RPC_URL").ok());

    if rpc_url.is_none() {
        bail!(
            "missing RPC URL for analyze-block: pass --rpc-url or set {} / ETH_RPC_URL",
            env_key
        );
    }

    bail!(
        "live RPC trace normalization is not implemented yet for chain={} block={}; use analyze-fixture with a normalized fixture. RPC URLs are intentionally not printed.",
        args.chain,
        args.block
    )
}

fn resolve_rpc_url(chain: &str, explicit: Option<String>) -> Result<String> {
    if let Some(url) = explicit {
        return Ok(url);
    }
    let env_key = match chain {
        "base" => "BASE_RPC_URL",
        "ethereum" => "ETH_RPC_URL",
        _ => "ETH_RPC_URL",
    };
    std::env::var(env_key)
        .or_else(|_| std::env::var("ETH_RPC_URL"))
        .with_context(|| {
            format!(
                "missing RPC URL for collect-block-range: pass --rpc-url or set {env_key} / ETH_RPC_URL"
            )
        })
}

fn rpc_call(url: &str, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params,
    });
    let response = ureq::post(url)
        .set("Content-Type", "application/json")
        .send_string(&body.to_string())
        .map_err(|err| {
            anyhow!(
                "RPC call {method} failed: {}",
                redact_rpc_url(&err.to_string())
            )
        })?;
    let text = response
        .into_string()
        .map_err(|err| anyhow!("RPC call {method} response read failed: {err}"))?;
    let value: serde_json::Value = serde_json::from_str(&text)
        .with_context(|| format!("RPC call {method} returned non-JSON response"))?;
    if let Some(error) = value.get("error") {
        bail!(
            "RPC call {} returned error: {}",
            method,
            redact_rpc_url(&error.to_string())
        );
    }
    Ok(value
        .get("result")
        .cloned()
        .unwrap_or(serde_json::Value::Null))
}

fn trace_accesses_from_rpc(value: &serde_json::Value) -> Result<Vec<TracePackAccess>> {
    let access_value = value
        .get("accesses")
        .unwrap_or(value)
        .as_array()
        .ok_or_else(|| anyhow!("debug tracer result did not contain an accesses array"))?;
    access_value
        .iter()
        .cloned()
        .map(|value| {
            serde_json::from_value(value)
                .context("debug tracer returned an access entry that does not match schema")
        })
        .collect()
}

fn trace_warnings_from_rpc(value: &serde_json::Value) -> Vec<String> {
    let mut warnings = value
        .get("warnings")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if let Some(faults) = value.get("faults").and_then(serde_json::Value::as_array) {
        if !faults.is_empty() {
            warnings.push(format!("debug tracer reported {} fault(s)", faults.len()));
        }
    }
    warnings.sort();
    warnings.dedup();
    warnings
}

fn required_string<'a>(value: &'a serde_json::Value, field: &str) -> Result<&'a str> {
    value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("expected transaction field `{field}`"))
}

fn value_string(value: Option<&serde_json::Value>) -> Option<String> {
    value
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned)
}

fn parse_hex_u64(value: &str) -> Option<u64> {
    u64::from_str_radix(value.strip_prefix("0x").unwrap_or(value), 16).ok()
}

fn hex_u64(value: u64) -> String {
    format!("0x{value:x}")
}

fn geth_storage_access_tracer_source() -> &'static str {
    include_str!("../../../tracers/geth-storage-access-tracer.js")
}

fn redact_rpc_url(value: &str) -> String {
    let mut out = value.to_owned();
    for scheme in ["https://", "http://"] {
        while let Some(start) = out.find(scheme) {
            let end = out[start..]
                .find(|ch: char| ch.is_whitespace() || ch == '"' || ch == '\'')
                .map(|offset| start + offset)
                .unwrap_or(out.len());
            out.replace_range(start..end, "<redacted-rpc-url>");
        }
    }
    redact_after_marker(&mut out, "Bearer ", "<redacted-bearer-token>");
    for marker in ["api_key=", "apikey=", "token=", "access_token=", "key="] {
        redact_after_marker(&mut out, marker, "<redacted-secret>");
    }
    out
}

fn redact_after_marker(out: &mut String, marker: &str, replacement: &str) {
    let mut search_start = 0;
    while let Some(relative_start) = out[search_start..].find(marker) {
        let start = search_start + relative_start + marker.len();
        let end = out[start..]
            .find(|ch: char| {
                ch.is_whitespace() || ch == '"' || ch == '\'' || ch == '&' || ch == ','
            })
            .map(|offset| start + offset)
            .unwrap_or(out.len());
        if start < end {
            out.replace_range(start..end, replacement);
            search_start = start + replacement.len();
        } else {
            search_start = end;
        }
    }
}

fn capability_failure(label: &str, err: &anyhow::Error) -> String {
    format!("{label}: {}", redact_rpc_url(&err.to_string()))
}

fn parse_workload(input: &str) -> std::result::Result<WorkloadKind, String> {
    input
        .parse()
        .map_err(|err: parallel_revm_lab_workload::WorkloadError| err.to_string())
}

fn parse_mode(input: &str) -> std::result::Result<ExecutionMode, String> {
    input
        .parse()
        .map_err(|err: parallel_revm_lab_executor::ExecutorError| err.to_string())
}

fn parse_conflict(input: &str) -> std::result::Result<f64, String> {
    let value: f64 = input
        .parse()
        .map_err(|_| format!("invalid conflict ratio `{input}`"))?;
    if (0.0..=1.0).contains(&value) {
        Ok(value)
    } else {
        Err(format!("conflict ratio `{input}` must be in 0.0..=1.0"))
    }
}

fn parse_conflict_list(input: &str) -> Result<Vec<f64>> {
    parse_csv(input, parse_conflict).context("invalid --conflicts list")
}

fn parse_thread_list(input: &str) -> Result<Vec<usize>> {
    parse_csv(input, |value| {
        let threads: usize = value
            .parse()
            .map_err(|_| format!("invalid thread count `{value}`"))?;
        if threads == 0 {
            Err("thread count must be > 0".to_owned())
        } else {
            Ok(threads)
        }
    })
    .context("invalid --threads list")
}

fn parse_csv<T>(
    input: &str,
    parser: impl Fn(&str) -> std::result::Result<T, String>,
) -> Result<Vec<T>> {
    let values: Vec<T> = input
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| parser(part).map_err(anyhow::Error::msg))
        .collect::<Result<Vec<_>>>()?;
    if values.is_empty() {
        bail!("list must not be empty");
    }
    Ok(values)
}

fn write_json(path: &Path, value: &impl serde::Serialize) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let file =
        File::create(path).with_context(|| format!("failed to create {}", path.display()))?;
    serde_json::to_writer_pretty(file, value)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

fn read_json<T>(path: &Path) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let file = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    serde_json::from_reader(file).with_context(|| format!("failed to read {}", path.display()))
}

fn load_optional_contract_labels(
    path: &Path,
) -> Result<std::collections::BTreeMap<String, String>> {
    if !path.exists() {
        return Ok(std::collections::BTreeMap::new());
    }
    let raw: std::collections::BTreeMap<String, String> = read_json(path)
        .with_context(|| format!("failed to load contract labels {}", path.display()))?;
    Ok(raw
        .into_iter()
        .map(|(address, label)| (address.to_ascii_lowercase(), label))
        .collect())
}

fn write_text(path: &Path, value: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    std::fs::write(path, value).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

fn rust_version() -> Option<String> {
    let output = Command::new("rustc").arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|version| version.trim().to_owned())
}

fn format_accesses(
    accesses: &std::collections::BTreeSet<parallel_revm_lab_model::AccessKey>,
) -> String {
    if accesses.is_empty() {
        return "[]".to_owned();
    }
    let joined = accesses
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{joined}]")
}

#[cfg(test)]
mod tests {
    use parallel_revm_lab_trace_model::StorageKey;

    use super::*;

    #[test]
    fn collect_redacts_rpc_urls() {
        let url = ["https://example.com/base?", "key", "=", "redactme"].concat();
        let bearer = ["Bearer", "very-secret"].join(" ");
        let api_key = ["api_key", "also-secret"].join("=");
        let input = format!(
            "failed: {url} and http://credential.example.org Authorization: {bearer} {api_key}"
        );
        let redacted = redact_rpc_url(&input);

        assert!(!redacted.contains("redactme"));
        assert!(!redacted.contains("credential.example"));
        assert!(!redacted.contains("very-secret"));
        assert!(!redacted.contains("also-secret"));
        assert_eq!(redacted.matches("<redacted-rpc-url>").count(), 2);
    }

    #[test]
    fn capability_failure_redacts_tokens() {
        let token_url = ["https://example.com/rpc?", "token", "=", "abc"].concat();
        let bearer = ["Bearer", "def"].join(" ");
        let err = anyhow!("provider said nope at {token_url} with {bearer}");
        let message = capability_failure("debug_traceTransaction", &err);

        assert!(message.contains("debug_traceTransaction"));
        assert!(!message.contains("abc"));
        assert!(!message.contains("def"));
        assert!(message.contains("<redacted-rpc-url>"));
    }

    #[test]
    fn collect_parses_hex_u64() {
        assert_eq!(parse_hex_u64("0x2a"), Some(42));
        assert_eq!(parse_hex_u64("2a"), Some(42));
        assert_eq!(parse_hex_u64("not-hex"), None);
    }

    #[test]
    fn collect_dry_run_numbers_cover_entire_range() {
        let numbers = block_numbers(7, 10).collect::<Vec<_>>();

        assert_eq!(numbers, vec![7, 8, 9, 10]);
    }

    #[test]
    fn collect_persists_valid_block_immediately() {
        let dir = tempfile::tempdir().unwrap();
        let block = test_trace_pack_block(42);

        persist_trace_pack_block(dir.path(), &block).unwrap();

        let path = trace_pack_block_path(dir.path(), 42);
        assert!(path.exists());
        let loaded: TracePackBlock = read_json(&path).unwrap();
        assert_eq!(loaded.block_number, 42);
        assert_eq!(loaded.total_gas_used, Some(7));
    }

    #[test]
    fn collect_rejects_wrong_resumed_block_number() {
        let dir = tempfile::tempdir().unwrap();
        let block = test_trace_pack_block(41);
        persist_trace_pack_block(dir.path(), &block).unwrap();

        let err = read_resumed_block(
            &trace_pack_block_path(dir.path(), 41),
            42,
            ChainKind::new("base"),
        )
        .unwrap_err()
        .to_string();

        assert!(err.contains("expected 42"));
    }

    #[test]
    fn trace_pack_benchmark_workload_preserves_declared_conflicts() {
        let pack = test_trace_pack_for_benchmark();
        let left = trace_pack_to_workload(&pack, 0);
        let right = trace_pack_to_workload(&pack, 0);

        assert_eq!(left.txs, right.txs);
        assert_eq!(left.conflict_pairs, 1);
        assert_eq!(left.observed_conflict, 1.0);
        assert_eq!(left.txs[0].declared_reads.len(), 1);
        assert_eq!(left.txs[0].declared_writes.len(), 1);
        assert_eq!(trace_pack_source_tx_count(&pack), Some(2));
    }

    #[test]
    fn missing_contract_labels_load_as_empty() {
        let dir = tempfile::tempdir().unwrap();
        let labels = load_optional_contract_labels(&dir.path().join("missing.json")).unwrap();

        assert!(labels.is_empty());
    }

    #[test]
    fn contract_labels_are_loaded_with_lowercase_addresses() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("labels.json");
        write_text(
            &path,
            r#"{"0X833589FCD6EDB6E08F4C7C32D4F71B54BDA02913":"Base USDC"}"#,
        )
        .unwrap();

        let labels = load_optional_contract_labels(&path).unwrap();

        assert_eq!(
            labels.get("0x833589fcd6edb6e08f4c7c32d4f71b54bda02913"),
            Some(&"Base USDC".to_owned())
        );
    }

    fn test_trace_pack_block(block_number: u64) -> TracePackBlock {
        TracePackBlock {
            chain: ChainKind::new("base"),
            block_number,
            block_hash: Some(format!("0x{block_number:064x}")),
            parent_hash: Some(format!("0x{:064x}", block_number.saturating_sub(1))),
            tx_count: 1,
            source_tx_count: Some(1),
            total_gas_used: Some(7),
            transactions: vec![TracePackTx {
                tx_index: TxIndex(0),
                tx_hash: TxHash(format!("0x{block_number:064x}")),
                from: Some(Address::canonical(
                    "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                )),
                to: Some(Address::canonical(
                    "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                )),
                gas_used: Some(7),
                status: Some("0x1".to_owned()),
                accesses: Vec::new(),
                warnings: Vec::new(),
            }],
            warnings: Vec::new(),
        }
    }

    fn test_trace_pack_for_benchmark() -> TracePack {
        let address = Address::canonical("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
        let slot = StorageKey::canonical(format!("0x{:064x}", 7));
        let mut block = test_trace_pack_block(50);
        block.tx_count = 2;
        block.source_tx_count = Some(2);
        block.total_gas_used = Some(14);
        block.transactions = vec![
            TracePackTx {
                tx_index: TxIndex(0),
                tx_hash: TxHash(format!("0x{:064x}", 50)),
                from: Some(Address::canonical(
                    "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                )),
                to: Some(address.clone()),
                gas_used: Some(7),
                status: Some("0x1".to_owned()),
                accesses: vec![TracePackAccess {
                    address: address.clone(),
                    slot: Some(slot.clone()),
                    kind: TracePackAccessKind::ReadWrite,
                    op: Some("SSTORE".to_owned()),
                    pc: Some(0),
                    depth: Some(1),
                    gas_remaining: Some(100),
                }],
                warnings: Vec::new(),
            },
            TracePackTx {
                tx_index: TxIndex(1),
                tx_hash: TxHash(format!("0x{:064x}", 51)),
                from: Some(Address::canonical(
                    "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaab",
                )),
                to: Some(address.clone()),
                gas_used: Some(7),
                status: Some("0x1".to_owned()),
                accesses: vec![TracePackAccess {
                    address,
                    slot: Some(slot),
                    kind: TracePackAccessKind::Read,
                    op: Some("SLOAD".to_owned()),
                    pc: Some(0),
                    depth: Some(1),
                    gas_remaining: Some(100),
                }],
                warnings: Vec::new(),
            },
        ];
        TracePack {
            manifest: TracePackManifest {
                schema_version: TRACE_PACK_SCHEMA_VERSION.to_owned(),
                chain: ChainKind::new("synthetic-base-shaped"),
                source: "unit-test".to_owned(),
                provenance: "synthetic trace-pack benchmark fixture".to_owned(),
                start_block: 50,
                end_block: 50,
                created_by_tool_version: "test".to_owned(),
                tracer_kind: "unit-test".to_owned(),
                notes: Vec::new(),
                warnings: Vec::new(),
            },
            blocks: vec![block],
        }
    }
}
