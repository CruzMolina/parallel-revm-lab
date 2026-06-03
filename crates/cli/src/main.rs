use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use clap::{Args, Parser, Subcommand};
use parallel_revm_lab_analyzer::{
    analyze_block_trace, render_html, render_markdown, schedule_trace_json,
};
use parallel_revm_lab_executor::{
    benchmark_report, run_access_list, run_optimistic, run_sequential, trace_json, ExecutionMode,
};
use parallel_revm_lab_trace_model::BlockAccessTrace;
use parallel_revm_lab_workload::{generate_workload, WorkloadConfig, WorkloadKind};

#[derive(Debug, Parser)]
#[command(
    name = "parallel-revm-lab",
    version,
    about = "Deterministic conflict-aware parallel execution workbench"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Bench(BenchArgs),
    Verify(VerifyArgs),
    Inspect(InspectArgs),
    AnalyzeFixture(AnalyzeFixtureArgs),
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

fn main() -> Result<()> {
    match Cli::parse().command {
        Commands::Bench(args) => bench(args),
        Commands::Verify(args) => verify(args),
        Commands::Inspect(args) => inspect(args),
        Commands::AnalyzeFixture(args) => analyze_fixture(args),
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
    write_json(&args.out, &report)?;
    write_text(&args.markdown, &render_markdown(&report))?;
    write_text(&args.html, &render_html(&report))?;
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
