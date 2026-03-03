# Domain Intake Checklist

Use this checklist before adding a new filesystem domain to SemanticFS.

Required checks:
1. Define a clear trust label.
2. Keep allow-roots explicit and root-relative.
3. Add at least one expected-path query before broadening the tracked suite.
4. Confirm the new domain does not create silent cross-root ambiguity.

Operational phrase:
Every new domain must carry an explicit expected-path query before it can enter the broadened benchmark suite.
