# Retrieval Golden Suites

Use this folder for relevance evaluation datasets.

Rules:
1. One suite per repo profile (`<name>.json`).
2. Keep queries stable; update only when ground truth changes.
3. Prefer 50+ queries for non-toy repos.
4. For tuning work, keep strict split suites:
- `*_tune.json`: allowed for parameter selection.
- `*_holdout.json`: final report only, never used for tuning decisions.

Run one suite:
```bash
cargo run --release -p semanticfs-cli -- --config config/local.toml benchmark relevance --fixture-repo /abs/repo --golden tests/retrieval_golden/<suite>.json
```

Run all suites in this directory:
```bash
cargo run --release -p semanticfs-cli -- --config config/local.toml benchmark relevance --fixture-repo /abs/repo --golden-dir tests/retrieval_golden
```

Repo-specific examples:
1. SemanticFS repo:
```bash
cargo run --release -p semanticfs-cli -- --config config/relevance-real.toml benchmark relevance --fixture-repo /abs/path/semanticFS --golden tests/retrieval_golden/semanticfs_repo.json --history
```
2. ai-testgen repo:
```bash
cargo run --release -p semanticfs-cli -- --config config/relevance-ai-testgen.toml benchmark relevance --fixture-repo /abs/path/ai-testgen --golden tests/retrieval_golden/ai_testgen_repo.json --history
```

Split an exploratory suite into deterministic tune/holdout files:
```bash
python scripts/split_golden_suite.py --input tests/retrieval_golden/<suite>.json --tune-output tests/retrieval_golden/<suite>_tune.json --holdout-output tests/retrieval_golden/<suite>_holdout.json --tune-count 10
```

Suggested naming:
1. `benchmark_repo.json` (fixture)
2. `real_repo_a.json` (your first larger repo)
3. `real_repo_b.json` (your second larger repo)
