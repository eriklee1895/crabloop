# CrabLoop — contributor and agent guide

## Project goal

CrabLoop is a small coding-agent harness built from scratch in Rust for hands-on learning.

- v1: approximately 300 lines of core Rust, with read_file, write_file, edit_file, and bash.
- v2: approximately 500 lines total, adding context management and summary compaction.
- v3: approximately 1,000 lines total, adding ~/.crabloop, TOML configuration, provider presets/Custom, credentials, and model catalog discovery/cache. Minimal Skills is optional if comfortably within scope.
- v4: Skills and provider-independent web_fetch/web_search. v5: recoverable JSONL sessions and interruption handling under ~/.crabloop. See docs/roadmap.md; these stages are planned, not implemented.
- Acceptance includes both coding repairs and general local agentic tasks such as reading source documents and producing a verifiable structured report.
- Use the OpenAI native Responses API and a hand-written agent loop.
- Grow the same implementation through small, explainable steps. Preserve runnable milestones with v1 and v2 Git tags.

Line counts are flexible scope guides: the user explicitly accepts v1 at 350–360 lines. Do not code-golf or reject correct extra output fields just to meet an arbitrary shape. Line counts cover all production Rust under src/. Tests, documentation, fixtures, and development scripts are separate. Keep normal formatting and necessary error handling; do not move implementation elsewhere to reduce the count.

## Current state and layout

Implementation is active under the approved v1 → v2 plan. Build the reference implementation and the Chinese tutorial together, in small verified increments. The Cargo package lives at the repository root, with src/ and tests/ beside Cargo.toml.

The Responses client, CLI configuration, and all four bounded tools, and the serial action loop are implemented. The v1 real coding task passed with DeepSeek and gpt-6-luna; a local-document report also passed after correcting an overly strict verifier. v1 is complete at 314 core code lines (319 physical), with 29 Rust and 3 Python regression tests. v2 context projection, local JSONL journals and bounded summary compaction are implemented. The full suite now contains 46 Rust tests. DeepSeek completed a real 10,240-byte run with five compactions and independent task verification. Tutorials 00–09 and both milestone acceptance records are complete. Final whole-branch review is complete; its budget-provenance finding is fixed with offline regression coverage. All 46 Rust and 6 Python tests pass. A minor mixed-phase final-text presentation issue is documented in evidence/v2/review.md. Current v2 core is 504 code lines / 511 physical lines. Preferred live model: DeepSeek V4.1 Flash, API id deepseek-flash at https://api.deepseek.com/responses. Test fallback: gpt-6-luna through the user's TeamoRouter endpoint. Marker probes and tool-loop evidence are recorded separately; v2 summary calls use reasoning.effort=none and at least 4,096 output tokens; selected models must support that effort. Main requests retain provider-default reasoning. v2 defaults to a 49,152-byte request limit; CRABLOOP_CONTEXT_BYTES overrides it (minimum 4,096). Root AGENTS.md is pinned in instructions.

Runtime configuration reads environment variables. OPENAI_API_KEY is the key for the selected endpoint; CRABLOOP_MODEL chooses the model. CRABLOOP_API_URL is a full endpoint; otherwise OPENAI_BASE_URL is a prefix to which /responses is appended. Local .env is not executed or loaded by the Rust runtime. v3 will introduce home-based TOML and credential files; keep v1/v2 configuration small until then.

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
- Maintain one implementation. For every completed version (v1 through v5 and future v6/v7), preserve a stable milestone/vN learning branch and an immutable vN tag. Use step commits/tags for chapter progression; continue new work on a development branch.
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

Commit Cargo.lock for the application. The user explicitly authorized committing and pushing every verified stage to GitHub. Push completed increments and their checkpoint tags on feat/crabloop-v1-v2; do not bundle unfinished changes into a stage commit.

Formatting uses rustfmt with max_width = 100 and use_small_heuristics = Max. Count all src/**/*.rs under this same public style for every milestone; never hide runtime code in scripts or documentation.

Keep credentials, local environment files, generated workspaces, and raw session logs out of Git. Publish only deliberately selected and sanitized learning evidence.
