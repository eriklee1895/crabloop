# CrabLoop — contributor and agent guide

## Project goal

CrabLoop is a small coding-agent harness built from scratch in Rust for hands-on learning.

- v1: approximately 300 lines of core Rust, with read_file, write_file, edit_file, and bash.
- v2: approximately 500 lines total, adding context management and summary compaction.
- Use the OpenAI native Responses API and a hand-written agent loop.
- Grow the same implementation through small, explainable steps. Preserve runnable milestones with v1 and v2 Git tags.

Line counts cover all production Rust under src/. Tests, documentation, fixtures, and development scripts are separate. Keep normal formatting and necessary error handling; do not move implementation elsewhere to reduce the count.

## Current state and layout

Implementation is active under the approved v1 → v2 plan. Build the reference implementation and the Chinese tutorial together, in small verified increments. The Cargo package lives at the repository root, with src/ and tests/ beside Cargo.toml.

The Responses client, CLI configuration, and three bounded UTF-8 file tools are implemented. Bash execution and the loop are next. Preferred live model: DeepSeek V4.1 Flash, API id deepseek-flash at https://api.deepseek.com/responses. Test fallback: gpt-6-luna through the user's TeamoRouter endpoint. Both passed a one-response marker probe; tool-loop and compaction evidence are separate later gates.

Runtime configuration reads environment variables. OPENAI_API_KEY is the key for the selected endpoint; CRABLOOP_MODEL chooses the model. CRABLOOP_API_URL is a full endpoint; otherwise OPENAI_BASE_URL is a prefix to which /responses is appended. Local .env is not executed or loaded by the Rust runtime.

Tutorial chapters live under tutorial/v1/ and tutorial/v2/. Actual development records live under docs/devlog/. Keep one Rust implementation at the repository root; use commits and tags to identify the code corresponding to each chapter.

Read the relevant design and task before implementation:

- [Design](docs/design.md)
- [Implementation and learning plan](docs/plan.md)

The user's latest instructions take precedence over those documents.

## How to develop

- Keep AGENTS.md current throughout implementation, as requested by the user. Update it when repository state, interfaces, commands, or durable decisions change. Put detailed history and evidence in docs/devlog/ rather than accumulating a session transcript here.
- Continue the approved plan without asking again between tasks. Maintain its progress ledger and resume from completed commits after compaction.

- Work in small increments, each with an observable behavior, an appropriate check, and a short explanation of what it teaches.
- Keep the current milestone focused. Introduce abstractions when a concrete need appears.
- Prefer concrete functions and small types. Use standard Rust and basic HTTP/JSON/async libraries.
- Maintain one implementation. Use Git commits and milestone tags to show its evolution.
- Write tutorials and explanations in Chinese, with Rust identifiers and protocol names in their original form.
- Tutorials explain a reproducible learning path. Development records capture actual decisions, changes, failures, verification results, and relevant commits.

## Tutorial voice and progression

Read [the writing guide](docs/writing-guide.md) before writing or revising a chapter.

- Use patient, connected Chinese prose. Lead the reader from an empty project through small runnable changes.
- Start with a concrete situation and explain why the next mechanism is needed.
- Introduce Rust details and harness concepts when the reader encounters them in the code.
- Explain each code increment, its file location, how to run it, and what the result means.
- Let observations and failures motivate the next change. Preserve the distinction between expected output and output actually observed.
- Use the chapter requirements as an editorial check, not a rigid sequence of repeated headings.
- Calibrate the voice with a short representative passage before writing many chapters.

## Core contracts

- Name the command tool bash and execute Bash explicitly with a defined working directory.
- edit_file requires a non-empty old_text with exactly one match; failed match/size validation leaves the file unchanged. File writes currently use ordinary filesystem writes, without atomic rollback.
- Return recoverable tool errors to the model with the original call_id.
- Preserve replayable Responses output items and their call/result associations.
- Treat model completion and independently verified task success as separate observations.
- In v2, keep execution records separate from the model's current input; commit compacted history only after validating the candidate.

## Verification and Git

Once the Cargo package exists, use cargo fmt --check, cargo clippy --locked --all-targets -- -D warnings, and cargo test --locked as the baseline checks. Add focused verification for the behavior changed.

Distinguish deterministic protocol tests, real local tool execution, and live-model runs. Report only what the actual evidence establishes.

Commit Cargo.lock for the application. Keep local changes incremental. Push or publish according to the user's authorization.

Keep credentials, local environment files, generated workspaces, and raw session logs out of Git. Publish only deliberately selected and sanitized learning evidence.
