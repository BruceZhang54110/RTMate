<!--
SYNC IMPACT REPORT
- Version change: 1.0.0 → 1.1.0
- Modified principles:
  - II. Workspace Clarity & Shared Contracts: updated from a three-crate workspace to a two-crate workspace; authentication logic is now owned by `rtmate-server` instead of a separate `rtmate-auth` crate.
- Added sections: N/A (no new sections)
- Removed sections: N/A
- Templates requiring updates:
  - .specify/templates/plan-template.md: ✅ reviewed, Constitution Check placeholder remains generic
  - .specify/templates/spec-template.md: ✅ reviewed, no constitution-specific updates required
  - .specify/templates/tasks-template.md: ✅ reviewed, no constitution-specific updates required
  - .specify/templates/commands/*.md: N/A (directory does not exist)
  - README.md: ✅ updated to reflect two-crate architecture and single-service runtime
  - docs/rust-overview.md: ✅ updated to reflect two-crate architecture and unified repository layer
- Follow-up TODOs: none
-->

# RTMate Constitution

## Core Principles

### I. Protocol-First & Unified Envelope

The public WebSocket and HTTP contracts are the primary APIs of RTMate. Every request and response MUST use the unified JSON envelope (`code`, `message`, `data`) documented in `README.md` and future `docs/protocol.md`. New events, error codes, or payload shapes MUST be added to the protocol documentation and covered by contract tests before implementation. Protocol changes that break existing clients require a major version bump.

**Rationale**: A stable, documented protocol lets clients evolve independently and makes RTMate usable as a service boundary rather than an internal crate.

### II. Workspace Clarity & Shared Contracts

The Cargo workspace is organized into two concerns:

- `rtmate-common` owns shared DTOs, error types, claims, and the unified response structure.
- `rtmate-server` owns the WebSocket runtime, connection management, handlers, and authentication logic.

Cross-crate dependencies MUST go through public APIs exposed by `rtmate-common`. `rtmate-server` MUST NOT leak its internal runtime details into `rtmate-common`, and `rtmate-common` MUST NOT depend on runtime-specific code. All database access within `rtmate-server` MUST be centralized in the repository layer under `infrastructure/persistence`, and all HTTP/WebSocket handlers MUST use the unified response envelope provided by `rtmate-common`.

**Rationale**: Clear crate boundaries and a single deployable runtime prevent tight coupling, reduce operational complexity, and mirror production Rust service organization where shared contracts live in a dedicated crate.

### III. Test-First Development

New behavior MUST be preceded by failing tests. Integration tests cover connection lifecycle, authentication flow, pub/sub contracts, and error paths. Contract tests verify the JSON envelope shape and event semantics. Unit tests cover pure logic such as token validation and response mapping.

**Rationale**: Tests define the intended contract before implementation, guard regressions in concurrent async code, and make refactors safe.

### IV. Minimal Abstractions / YAGNI

Prefer explicit, readable Rust over premature frameworks or organizational-only modules. A new crate, trait layer, or abstraction MUST solve at least two concrete use cases or be required by a concrete feature. Complexity MUST be justified in the implementation plan under the Complexity Tracking section.

**Rationale**: RTMate aims to be a minimal, readable realtime kernel. Unnecessary abstraction obscures ownership and concurrency patterns that are central to learning from this codebase.

### V. Observability & Structured Logging

All external responses MUST use the unified JSON envelope. Internal state changes (connect, authenticate, subscribe, unsubscribe, disconnect, errors) MUST emit structured `tracing` logs with sufficient context (client_id, app_id, channel_id where applicable) to reconstruct a connection's lifecycle. Logs MUST NOT include secrets or full tokens.

**Rationale**: WebSocket systems are hard to debug after the fact. Structured logs and a consistent response envelope make failures observable without leaking sensitive data.

## Technology Stack & Dependencies

- **Language & Runtime**: Rust stable >= 1.79 with Tokio asynchronous runtime.
- **Web Framework**: Axum for HTTP routing and WebSocket upgrade handling.
- **Serialization**: `serde` and `serde_json` for all JSON payloads.
- **Concurrency**: `tokio` channels and `dashmap`/`dashset` for shared in-memory state.
- **Observability**: `tracing` and `tracing-subscriber` for structured logging.
- **Configuration**: Environment-based configuration via `config` and `dotenvy`.
- **Workspace**: Multi-crate Cargo workspace (`rtmate-common`, `rtmate-auth`, `rtmate-server`).
- **Quality Gates**: `cargo test` and `cargo clippy` MUST pass before a change is considered complete.

## Development Workflow

1. **Spec-Driven**: Feature work follows the speckit workflow: `spec.md` → `plan.md` → `tasks.md` → implementation.
2. **Independent User Stories**: Each user story MUST be independently testable and deliverable as a vertical slice.
3. **Test First**: Write failing tests before implementation; keep tests green after each task.
4. **Code Review**: Every change MUST be reviewed for compliance with this constitution, especially crate boundaries, protocol envelope usage, and logging discipline.
5. **Documentation**: Public protocol changes, new environment variables, and new crates require updates to `README.md`, `docs/`, or inline crate documentation.
6. **Branching**: Feature branches follow the `###-feature-name` convention defined by the speckit git workflow.

## Governance

This constitution supersedes local conventions and style opinions. Amendments require:

1. A documented change to `.specify/memory/constitution.md` with an updated Sync Impact Report.
2. Version bump according to semantic versioning rules:
   - **MAJOR**: Backward-incompatible governance changes or principle removals/redefinitions.
   - **MINOR**: New principle/section added or materially expanded guidance.
   - **PATCH**: Clarifications, wording improvements, typo fixes, or non-semantic refinements.
3. Propagation to dependent templates and runtime guidance docs when applicable.
4. A compliance review that verifies no unexplained placeholder tokens remain and dates are in `YYYY-MM-DD` format.

**Version**: 1.1.0 | **Ratified**: 2026-06-18 | **Last Amended**: 2026-08-04
