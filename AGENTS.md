# CrabLoop — contributor and agent guide

## Project goal

CrabLoop is a small coding-agent harness built from scratch in Rust for hands-on learning.

Keep the project focused on a mini coding agent and its Harness; do not reframe it as a Workbench.

- v1: hard cap 400 code lines, sweet spot 300-350. Teach only the hand-written model/tool/feedback/stop loop with four tools; cut everything else to later versions.
- v2: about 500-600 lines total, adding context management and summary compaction.
- v3: 1,000+ lines total (full-featured starts here), adding ~/.crabloop, TOML configuration, provider presets/Custom, credentials, and model catalog discovery/cache. Minimal Skills is optional if comfortably within scope.
- v4: Skills and provider-independent web_fetch/web_search. v5: recoverable JSONL sessions and interruption handling under ~/.crabloop. See docs/roadmap.md; these stages are planned, not implemented.
- Acceptance includes both coding repairs and general local agentic tasks such as reading source documents and producing a verifiable structured report.
- Use the OpenAI native Responses API and a hand-written agent loop.
- Grow the same implementation through small, explainable steps. Preserve runnable milestones with v1 and v2 Git tags.

## Current state and layout

The only v1 learning code is milestone/v1. Six additive tutorial patches (one per chapter 01-06) rebuild the same src tree from an empty directory; scripts/check-lessons.py verifies each stage (fmt + tests) and that every patched file equals the working tree. Historical tags preserve older steps and evidence, not alternative learning entries.

v1 has 304 code / 350 physical lines in four files: main.rs (stdin entry, owns history), lib.rs (Config, Stop, run_turn), model.rs (request/respond, serde Response/Item, raw output kept for replay), tools.rs (schemas, execute, four tools). examples/probe.rs sends one request for chapter 01. No Session, editor, persistence, SSE, .env or provider presets.

Each non-empty stdin line is one task, terminal or pipe alike; main keeps one Vec<Value> until EOF and run_turn borrows it. stdout carries only final answers; stderr shows the crab> prompt, `turn N` and one `→ name args · ok=` line per tool. MaxTurns/ContextLimit exit with code 2; errors end the process. Limits: 16 turns per task, request 4 MiB, output 131,072 tokens, file 64 KiB, read 8 KiB, pipes 4 KiB each, bash 30 s.

Config is OPENAI_API_KEY, OPENAI_BASE_URL (default https://api.openai.com/v1) and CRABLOOP_MODEL (required); workspace is cwd; arguments are ignored. DeepSeek: base https://api.deepseek.com, model deepseek-flash. The bash child does not inherit OPENAI_API_KEY. scripts/run-live.py reads .env for the driver only. Never execute or print .env contents.

Offline baseline: 29 Rust + 6 Python tests, fmt/clippy, check-lessons. Live DeepSeek evidence for the slim version: evidence/v1/slim-acceptance.md (docs/devlog/38-v1-slim-live.md). Code strings/comments are English; tutorials are Chinese. CLI Help remains planned in docs/cli-help-plan.md. Read docs/v1-scope.md for scope; docs/design.md and docs/plan.md preserve the original learning sequence.

## How to develop

- Keep AGENTS.md current with repository state, interfaces and durable decisions; put detailed history and evidence in docs/devlog/.
- Keep v1 focused on the smallest readable loop; richer interaction/configuration belongs later. v1/v2 stay non-streaming; basic SSE moves to v3.

- Work in small increments, each with an observable behavior, an appropriate check, and a short explanation of what it teaches.
- Keep the current milestone focused. Introduce abstractions when a concrete need appears.
- Prefer concrete functions and small types. Use standard Rust and basic HTTP/JSON/async libraries.
- Maintain one implementation. Use Git commits and milestone tags to show its evolution.
- Write tutorials and explanations in Chinese, with Rust identifiers and protocol names in their original form.
- Prefer English industry terms: Harness, context, compaction, provider, tool calling, session; explain in Chinese as needed.
- Keep the v1 README concise: use "300+ lines" for size, retain startup and learning entrypoints, and omit verification reports, budget parameters and development history. Detailed contributor information belongs in dedicated documents.
- Tutorials explain a reproducible learning path. Development records capture actual decisions, changes, failures, verification results, and relevant commits.

## Tutorial voice and progression

Read [the writing guide](docs/writing-guide.md) before writing or revising a chapter. Chapters teach final interfaces; patches only add code.

- Use patient, connected Chinese prose. Lead the reader from an empty project through small runnable changes.
- Keep prose concise. Roadmaps describe goals/scope; devlogs hold decisions and verification details.
- Start with a concrete situation and explain why the next mechanism is needed.
- Introduce Rust details and harness concepts when the reader encounters them in the code.
- Explain each code increment, its file location, how to run it, and what the result means.
- Let observations and failures motivate the next change. Preserve the distinction between expected output and output actually observed.
- Use the chapter requirements as an editorial check, not a rigid sequence of repeated headings.
- The user approved chapter 04 as the prose/diagram model. Apply its concrete experiment, data-flow diagrams and observable failure/recovery approach across the v1 tutorial; keep stage interfaces explicit.

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

Commit Cargo.lock. The user authorizes verified increments to be committed and pushed, with main holding the latest validated tree. The user requested one v1 branch: milestone/v1 is the small teaching core with in-process terminal chat. Keep main and milestone/v2 separately, and preserve historical v1/v2 and step tags. Do not create another interaction-version branch/tag or merge this older backend over main.

Separate top-level items and logical phases within functions with one blank line; keep related statements together. Blank lines do not count toward the code-line budget. Formatting uses rustfmt with max_width = 100 and use_small_heuristics = Max; expand protocol JSON for teaching clarity. Count all src/**/*.rs under this same public style for every milestone; never hide runtime code in scripts or documentation.

Keep credentials, local environment files, generated workspaces, and raw session logs out of Git. Publish only deliberately selected and sanitized learning evidence.
