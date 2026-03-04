# Trust Boundary Contract

Phase 5 governance rule:
Every domain must declare explicit watch ownership and update priority before it is allowed to scale.

Operational constraints:
1. New roots stay bounded by `allow_roots`.
2. New roots start untrusted unless there is an explicit reason not to.
3. Watch scope should stay intentional instead of defaulting to broad recursive subscriptions.
