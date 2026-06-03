use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_parallel-revm-lab")
}

#[test]
fn help_smoke() {
    let output = Command::new(bin()).arg("--help").output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("parallel-revm-lab"));
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
