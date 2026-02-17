# v1.1.0-rc1 Release Checklist

## Preconditions
1. Clean working tree for release commit scope.
2. `config/semanticfs.sample.toml` reflects locked v1.1 defaults.
3. Benchmark fixture corpus is available at `tests/fixtures/benchmark_repo`.

## Mandatory gates
1. `cargo fmt --check`
2. `cargo test --workspace`
3. `cargo run --release -p semanticfs-cli -- --config config/semanticfs.sample.toml benchmark release-gate --refresh --fixture-repo tests/fixtures/benchmark_repo --soak-seconds 5`
4. `cargo run --release -p semanticfs-cli -- --config config/semanticfs.sample.toml benchmark soak --duration-seconds 1800 --fixture-repo tests/fixtures/benchmark_repo --max-soak-p95-ms 250 --max-errors 0 --max-rss-mb 2048`

## Fast local preflight
1. `bash scripts/rc_preflight.sh config/semanticfs.sample.toml tests/fixtures/benchmark_repo`

## Required artifacts
1. `.semanticfs/bench/latest.json`
2. `.semanticfs/bench/lancedb_tuning.json`
3. `.semanticfs/bench/release_gate.json`
4. `.semanticfs/bench/soak_latest.json`

## RC cut
1. `git add -A`
2. `git commit -m "release: v1.1.0-rc1"`
3. `git tag -a v1.1.0-rc1 -m "SemanticFS v1.1.0-rc1"`
4. `git push origin <branch> --follow-tags`

## Sign-off fields
1. Owner:
2. Date:
3. Release gate pass: yes/no
4. Long soak pass: yes/no
5. Blocking issues:
