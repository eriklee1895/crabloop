# CrabLoop — contributor and agent guide

## Project goal

CrabLoop is a small coding-agent harness built from scratch in Rust for hands-on learning.

- v1: approximately 300 lines of core Rust, with read_file, write_file, edit_file, and bash.
- v2: approximately 500 lines total, adding context management and summary compaction.
- Use the OpenAI native Responses API and a hand-written agent loop.
- Grow the same implementation through small, explainable steps. Preserve runnable milestones with v1 and v2 Git tags.

Line counts cover all production Rust under src/. Tests, documentation, fixtures, and development scripts are separate. Keep normal formatting and necessary error handling; do not move implementation elsewhere to reduce the count.

## Current state and layout

The repository currently contains planning documents. The Cargo package will live at the repository root, with src/ and tests/ beside Cargo.toml.

The tutorial and development-log layout is still under discussion. Paths such as lessons/ and evidence/ in the existing plan are provisional. Keep documentation links consistent when the layout is settled.

Read the relevant design and task before implementation:

- [Design](docs/superpowers/specs/2026-10-11-crabloop-design.md)
- [Implementation and learning plan](docs/superpowers/plans/2026-10-11-crabloop-v1-v2.md)

The user's latest instructions take precedence over those documents.

## How to develop

- Work in small increments, each with an observable behavior, an appropriate check, and a short explanation of what it teaches.
- Keep the current milestone focused. Introduce abstractions when a concrete need appears.
- Prefer concrete functions and small types. Use standard Rust and basic HTTP/JSON/async libraries.
- Maintain one implementation. Use Git commits and milestone tags to show its evolution.
- Write tutorials and explanations in Chinese, with Rust identifiers and protocol names in their original form.
- Tutorials explain a reproducible learning path. Development records capture actual decisions, changes, failures, verification results, and relevant commits.

## Core contracts

- Name the command tool bash and execute Bash explicitly with a defined working directory.
- edit_file requires a non-empty old_text with exactly one match; failed edits leave the file unchanged.
- Return recoverable tool errors to the model with the original call_id.
- Preserve replayable Responses output items and their call/result associations.
- Treat model completion and independently verified task success as separate observations.
- In v2, keep execution records separate from the model's current input; commit compacted history only after validating the candidate.

## Verification and Git

Once the Cargo package exists, use cargo fmt --check, cargo clippy --locked --all-targets -- -D warnings, and cargo test --locked as the baseline checks. Add focused verification for the behavior changed.

Distinguish deterministic protocol tests, real local tool execution, and live-model runs. Report only what the actual evidence establishes.

Commit Cargo.lock for the application. Keep local changes incremental. Push or publish according to the user's authorization.

Keep credentials, local environment files, generated workspaces, and raw session logs out of Git. Publish only deliberately selected and sanitized learning evidence.
