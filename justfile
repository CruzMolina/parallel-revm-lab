fmt:
    cargo fmt --all

clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
    cargo test --workspace --all-features

verify:
    cargo run -p parallel-revm-lab -- verify --workload mixed --txs 100 --conflicts 0.0,0.2,0.5,0.7,0.95 --threads 1,2,4 --seed-start 1 --seed-end 20

validate: fmt clippy test verify

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
