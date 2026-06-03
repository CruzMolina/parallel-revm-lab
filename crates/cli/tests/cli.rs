use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_parallel-revm-lab")
}

fn workspace_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("cli crate is under crates/cli")
        .to_path_buf()
}

#[test]
fn help_smoke() {
    let output = Command::new(bin()).arg("--help").output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("parallel-revm-lab"));
    assert!(stdout.contains("analyze-trace"));
    assert!(stdout.contains("analyze-trace-pack"));
    assert!(stdout.contains("recommend-access-lists"));
    assert!(!stdout.contains("analyze-block"));
}

#[test]
fn tiny_bench_all_writes_report() {
    let dir = tempfile::tempdir().unwrap();
    let report = dir.path().join("bench.json");
    let trace = dir.path().join("bench.trace.json");
    let output = Command::new(bin())
        .args([
            "bench",
            "--workload",
            "mixed",
            "--txs",
            "20",
            "--conflict",
            "0.5",
            "--mode",
            "all",
            "--threads",
            "2",
            "--seed",
            "42",
            "--vm-steps",
            "8",
            "--out",
            report.to_str().unwrap(),
            "--trace",
            trace.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json = std::fs::read_to_string(report).unwrap();
    assert!(json.contains("\"modes\""));
    assert!(json.contains("\"vm_steps\": 8"));
    assert!(json.contains("\"deterministic_passed\": true"));
    let trace_json = std::fs::read_to_string(trace).unwrap();
    assert!(trace_json.contains("traceEvents"));
}

#[test]
fn verify_smoke() {
    let output = Command::new(bin())
        .args([
            "verify",
            "--workload",
            "mixed",
            "--txs",
            "30",
            "--conflicts",
            "0.0,0.5",
            "--threads",
            "1,2",
            "--seed-start",
            "1",
            "--seed-end",
            "3",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("verified"));
}

#[test]
fn analyze_fixture_smoke_writes_reports() {
    let dir = tempfile::tempdir().unwrap();
    let report = dir.path().join("parallelism.json");
    let markdown = dir.path().join("parallelism.md");
    let html = dir.path().join("parallelism.html");
    let trace = dir.path().join("schedule.trace.json");
    let fixture = workspace_root().join("fixtures/base-mini-trace.json");
    let output = Command::new(bin())
        .args([
            "analyze-fixture",
            "--fixture",
            fixture.to_str().unwrap(),
            "--out",
            report.to_str().unwrap(),
            "--markdown",
            markdown.to_str().unwrap(),
            "--html",
            html.to_str().unwrap(),
            "--trace",
            trace.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json = std::fs::read_to_string(report).unwrap();
    assert!(json.contains("\"tx_count\": 12"));
    assert!(json.contains("\"conflict_pair_count\": 7"));
    assert!(json.contains("\"deterministic_hash\""));
    assert!(std::fs::read_to_string(markdown)
        .unwrap()
        .contains("Parallelism Report"));
    assert!(std::fs::read_to_string(html)
        .unwrap()
        .contains("Parallelism Report"));
    assert!(std::fs::read_to_string(trace)
        .unwrap()
        .contains("traceEvents"));
}

#[test]
fn analyze_trace_geth_struct_logs_smoke_writes_reports() {
    let dir = tempfile::tempdir().unwrap();
    let report = dir.path().join("geth.json");
    let markdown = dir.path().join("geth.md");
    let html = dir.path().join("geth.html");
    let trace = dir.path().join("geth.trace.json");
    let fixture = workspace_root().join("fixtures/geth-mini-struct-logs.json");
    let output = Command::new(bin())
        .args([
            "analyze-trace",
            "--format",
            "geth-struct-logs",
            "--fixture",
            fixture.to_str().unwrap(),
            "--out",
            report.to_str().unwrap(),
            "--markdown",
            markdown.to_str().unwrap(),
            "--html",
            html.to_str().unwrap(),
            "--trace",
            trace.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json = std::fs::read_to_string(report).unwrap();
    assert!(json.contains("\"tx_count\": 3"));
    assert!(json.contains("\"conflict_pair_count\": 1"));
    assert!(json.contains("\"access_model\": \"declared-read-write-lower-bound\""));
    assert!(std::fs::read_to_string(markdown)
        .unwrap()
        .contains("Parallelism Report"));
    assert!(std::fs::read_to_string(html)
        .unwrap()
        .contains("Hot Storage Slots"));
    assert!(std::fs::read_to_string(trace)
        .unwrap()
        .contains("traceEvents"));
}

#[test]
fn analyze_trace_pack_smoke_writes_dossier() {
    let dir = tempfile::tempdir().unwrap();
    let report = dir.path().join("dossier.json");
    let markdown = dir.path().join("dossier.md");
    let html = dir.path().join("dossier.html");
    let trace = dir.path().join("schedule.trace.json");
    let trace_dir = workspace_root().join("trace-packs/demo-mini");
    let output = Command::new(bin())
        .args([
            "analyze-trace-pack",
            "--trace-dir",
            trace_dir.to_str().unwrap(),
            "--workers",
            "1,2,4,8",
            "--out",
            report.to_str().unwrap(),
            "--markdown",
            markdown.to_str().unwrap(),
            "--html",
            html.to_str().unwrap(),
            "--trace",
            trace.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json = std::fs::read_to_string(report).unwrap();
    assert!(json.contains("\"report_version\": \"trace-pack-dossier-v1\""));
    assert!(json.contains("\"tx_count\": 7"));
    assert!(json.contains("\"conflict_pair_count\": 2"));
    assert!(std::fs::read_to_string(markdown)
        .unwrap()
        .contains("Contention Dossier"));
    assert!(std::fs::read_to_string(html)
        .unwrap()
        .contains("Worker Simulation"));
    assert!(std::fs::read_to_string(trace)
        .unwrap()
        .contains("traceEvents"));
    assert!(dir.path().join("hot-contracts.csv").exists());
    assert!(dir.path().join("hot-slots.csv").exists());
    assert!(dir.path().join("worker-simulation.csv").exists());
}

#[test]
fn recommend_access_lists_smoke_writes_observed_hints() {
    let dir = tempfile::tempdir().unwrap();
    let report = dir.path().join("recommendations.json");
    let trace_dir = workspace_root().join("trace-packs/demo-mini");
    let output = Command::new(bin())
        .args([
            "recommend-access-lists",
            "--trace-dir",
            trace_dir.to_str().unwrap(),
            "--out",
            report.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json = std::fs::read_to_string(report).unwrap();
    assert!(json.contains("\"report_version\": \"observed-access-hints-v1\""));
    assert!(json.contains("observed access hints only"));
    assert!(json.contains("not production-ready Ethereum access lists"));
}

#[test]
fn analyze_block_without_rpc_url_is_clear() {
    let dir = tempfile::tempdir().unwrap();
    let output = Command::new(bin())
        .env_remove("BASE_RPC_URL")
        .env_remove("ETH_RPC_URL")
        .args([
            "analyze-block",
            "--chain",
            "base",
            "--block",
            "38014901",
            "--out",
            dir.path().join("out.json").to_str().unwrap(),
            "--markdown",
            dir.path().join("out.md").to_str().unwrap(),
            "--html",
            dir.path().join("out.html").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("missing RPC URL for analyze-block"));
    assert!(!stderr.contains("http"));
}
