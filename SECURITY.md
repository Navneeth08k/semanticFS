# Security Policy

## Trust model

SemanticFS is a **read-only** filesystem intelligence layer. It indexes and serves file content for AI agent search — it never writes to, executes, or modifies indexed files.

Key boundaries:

| Boundary | Enforcement |
|---|---|
| File writes | Always disabled (`policy.read_only = true`) |
| Secret redaction | `policy.deny_secret_paths` and `policy.search_result_redaction` redact common API key / token patterns from search results |
| Filesystem scope | `policy-guard` enforces `allow_roots` and `deny_globs` — only files within configured roots are indexed or readable |
| Network scope | The MCP HTTP server binds to `127.0.0.1` only (loopback). It is **not** exposed to the network by default. |
| Subprocess execution | None. SemanticFS does not execute code in indexed repos. |

## `policy-guard` boundary

The `policy-guard` crate is the central enforcement point. It:

- Validates that every file path requested for indexing or reading falls within at least one configured `allow_roots`
- Applies `deny_globs` to reject paths matching exclusion patterns
- Applies `deny_secret_paths` to block paths with common secret-file names (`.env`, `*.pem`, etc.)
- Returns an `AccessDecision` for every path — indexing and retrieval code must check this before processing

Any retrieval result that passes through `policy-guard` with a `Deny` decision is silently dropped before returning to the agent.

## MCP server network exposure

The MCP server is designed for **local use only**:

- Default bind: `127.0.0.1:9464` (loopback, not accessible from other machines)
- No authentication is implemented — assume any process on the local machine can reach it
- Do not change the bind address to `0.0.0.0` in a shared or cloud environment without adding an authentication proxy

## Reporting vulnerabilities

Please report security vulnerabilities privately via GitHub's **Security → Report a vulnerability** feature (private advisory).

Do not open a public issue for security vulnerabilities. We will respond within 5 business days and aim to release a fix within 14 days for critical issues.

## Supported versions

Security fixes are applied to the latest released version only. We do not backport to older releases.
