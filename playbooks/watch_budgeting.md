# Watch Budgeting

Phase 4 keeps watch load bounded as domain count grows.

Core rule:
1. Watch explicit files when a domain only allows exact file paths.
2. Watch full directories only when the domain needs wildcard coverage.
3. Prefer the minimal watch target set that still preserves deterministic ownership.

Operational note:
Per-domain watch budgeting should prevent redundant recursive subscriptions when a narrower target is sufficient.
