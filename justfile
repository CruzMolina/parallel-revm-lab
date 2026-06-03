fmt:
    cargo fmt --all

clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
    cargo test --workspace --all-features

verify:
    cargo run -p parallel-revm-lab -- verify --workload mixed --txs 100 --conflicts 0.0,0.2,0.5,0.7,0.95 --threads 1,2,4 --seed-start 1 --seed-end 20

analyze-fixture:
    cargo run -p parallel-revm-lab -- analyze-fixture --fixture fixtures/base-mini-trace.json --out reports/base-mini-trace.parallelism.json --markdown reports/base-mini-trace.md --html reports/base-mini-trace.html --trace reports/base-mini-trace.schedule.trace.json

analyze-trace:
    cargo run -p parallel-revm-lab -- analyze-trace --format geth-struct-logs --fixture fixtures/geth-mini-struct-logs.json --out reports/geth-mini.parallelism.json --markdown reports/geth-mini.md --html reports/geth-mini.html --trace reports/geth-mini.schedule.trace.json

analyze-trace-pack:
    cargo run -p parallel-revm-lab -- analyze-trace-pack --trace-dir trace-packs/demo-mini --workers 1,2,4,8 --out reports/demo-dossier.json --markdown reports/demo-dossier.md --html reports/demo-dossier.html --trace reports/demo-schedule.trace.json

dossier-demo:
    cargo run -p parallel-revm-lab -- analyze-trace-pack --trace-dir trace-packs/demo-mini --workers 1,2,4,8 --out case-studies/demo-trace-pack/dossier.json --markdown case-studies/demo-trace-pack/summary.md --html case-studies/demo-trace-pack/dossier.html --trace case-studies/demo-trace-pack/schedule.trace.json

recommend-access-lists:
    cargo run -p parallel-revm-lab -- recommend-access-lists --trace-dir trace-packs/demo-mini --out reports/access-list-recommendations.json

collect-base-dry-run:
    cargo run -p parallel-revm-lab -- collect-block-range --chain base --start-block 38014901 --end-block 38014910 --tracer geth-js-storage --out trace-packs/base-38014901-38014910 --dry-run

revm-trace-pack-smoke:
    cargo run -p parallel-revm-lab-revm-smoke --example emit_trace_pack

analyze-fixture-open: analyze-fixture
    open reports/base-mini-trace.html

revm-smoke:
    cargo test -p parallel-revm-lab-revm-smoke --all-features

validate-dossier: analyze-trace-pack dossier-demo recommend-access-lists

validate: fmt clippy test verify analyze-fixture analyze-trace validate-dossier revm-smoke

validate-full: validate revm-trace-pack-smoke bench-heavy-smoke

bench-smoke:
    cargo run --release -p parallel-revm-lab -- bench --workload mixed --txs 1000 --conflict 0.5 --mode all --threads 4 --seed 42 --out reports/mixed-c50.json --trace reports/mixed-c50.trace.json

bench-heavy-smoke:
    cargo run --release -p parallel-revm-lab -- bench --workload storage --txs 1000 --conflict 0.0 --mode all --threads 4 --seed 42 --vm-steps 50000 --out reports/storage-c0-vmsteps.json

bench-all:
    cargo run --release -p parallel-revm-lab -- bench --workload erc20 --txs 1000 --conflict 0.0 --mode all --threads 4 --seed 42 --out reports/erc20-c0.json
    cargo run --release -p parallel-revm-lab -- bench --workload erc20 --txs 1000 --conflict 0.2 --mode all --threads 4 --seed 42 --out reports/erc20-c20.json
    cargo run --release -p parallel-revm-lab -- bench --workload erc20 --txs 1000 --conflict 0.5 --mode all --threads 4 --seed 42 --out reports/erc20-c50.json
    cargo run --release -p parallel-revm-lab -- bench --workload hot-pool --txs 1000 --conflict 0.7 --mode all --threads 4 --seed 42 --out reports/hot-pool-c70.json
    cargo run --release -p parallel-revm-lab -- bench --workload hot-pool --txs 1000 --conflict 0.95 --mode all --threads 4 --seed 42 --out reports/hot-pool-c95.json
    cargo run --release -p parallel-revm-lab -- bench --workload mixed --txs 1000 --conflict 0.2 --mode all --threads 4 --seed 42 --out reports/mixed-c20.json
    cargo run --release -p parallel-revm-lab -- bench --workload mixed --txs 1000 --conflict 0.5 --mode all --threads 4 --seed 42 --out reports/mixed-c50.json
    cargo run --release -p parallel-revm-lab -- bench --workload storage --txs 1000 --conflict 0.0 --mode all --threads 4 --seed 42 --vm-steps 50000 --out reports/storage-c0-vmsteps.json

trace-smoke:
    cargo run --release -p parallel-revm-lab -- bench --workload mixed --txs 200 --conflict 0.5 --mode all --threads 2 --seed 7 --out reports/tmp-trace-smoke.json --trace reports/tmp-trace-smoke.trace.json

clean-reports:
    rm -f reports/*.json
