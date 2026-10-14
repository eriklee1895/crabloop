# CrabLoop — contributor and agent guide

## Project goal

CrabLoop is a small coding-agent harness built from scratch in Rust for hands-on learning.

v2-v3: learn Agent fundamentals in small, readable code; defer reliability hardening. Preserve core contracts. See docs/roadmap.md.

- v1: target about 300 lines, HARD cap below 400 total production Rust code lines. Teach the hand-written model/tool/feedback/stop loop with four tools.
- v2: approximately 500 lines total, adding context management and summary compaction.
- v3: approximately 1,000 lines total: basic SSE, ~/.crabloop, TOML, credentials, provider presets/Custom, model catalog/cache, and context display. Minimal Skills is optional within scope.
- After v3: a separate opt-in telemetry topic using lifecycle observers and Rust OpenTelemetry/OTLP to Langfuse v4. Share event IDs with TUI/journal; export failure must not stop the agent. See docs/roadmap.md.
- Planned v4: Skills, Tavily Search/Firecrawl Scrape, todo_write, TOML config menu and selective .env import with visible sources. v5: ~/.crabloop JSONL/Todo recovery, ask_user_question, pending answers and interruption handling.
- v1/v2 stay non-streaming. v6: role/content styles, Markdown, slash/Todo/question UI, then layout; show step/context/tools. v7: memory. v8: optional Rig comparison; mainline has no Rig prerequisite. Keep polish out of v1.
- Acceptance includes both coding repairs and general local agentic tasks such as reading source documents and producing a verifiable structured report.
- Use the OpenAI native Responses API and a hand-written agent loop.
- Grow the same implementation through small, explainable steps. Preserve runnable milestones with v1 and v2 Git tags.
- Study llm-space's tool contracts and UI/state boundaries; fixed references are in docs/roadmap.md.

2026-10-12 v1 rule: <400 is mandatory. After normal rustfmt, count all production src/**/*.rs with scripts/count-lines.py and report physical lines. Tests/docs/fixtures/dev scripts are separate. Reduce scope; never hide code, code-golf, outsource the loop or omit necessary errors. See docs/v1-scope.md.

## Current state and layout

V1 learning code is milestone/v1; seven patches reproduce it. Historical tags retain evidence. Chinese tutorials accompany root Cargo/src/tests. Debugging: docs/debugging.md.

Historical v1/step-05-v1: 314/319; v2: 504/511. V1 GA: 397/402, 43 Rust + 4 Python, live repair/report and PTY passed; see evidence/v1/ga-review.md. Main: 837/849, 65 Rust + 6 Python, fmt/clippy passed. Counts=code/physical; preserve historical evidence.

Preferred model: DeepSeek V4.1 Flash (deepseek-flash); fallback: gpt-6-luna through TeamoRouter. Both passed v1 tasks. DeepSeek passed v2 with five compactions; Luna v2 is untested. Summaries use reasoning.effort=none and at least 4,096 output tokens; main requests use provider defaults. Request limit: 49,152 bytes; CRABLOOP_CONTEXT_BYTES overrides it (min 4,096). Root AGENTS.md is pinned and must stay below 8,192 bytes. Distinguish offline, live and semantic evidence.

`cargo run`: multi-turn chat in cwd; `-p/--print TASK`: one task. Positional tasks and piped stdin remain supported; `--chat` forces chat. Rustyline edits input/history; /new resets, /exit or Ctrl-D quits, Ctrl-C cancels. Session retains final answers/tool groups. After errors/interruption, inspect files and use /new; no automatic effect replay. Empty chat creates no journal/request. UI copy is English; model/user text keeps its language. Commentary precedes tools; -p sends it to stderr. Final answers stay separate.

`src/cli.rs` reads only cwd .env as data with dotenvy; process values win. Prefer DEEPSEEK_API_KEY/deepseek-flash absent explicit generic config; CRABLOOP_PROVIDER=deepseek/openai overrides. Generic mode uses OPENAI_API_KEY/OPENAI_BASE_URL/CRABLOOP_MODEL; CRABLOOP_API_URL overrides the endpoint. Keep provider keys paired. SSE/home/TOML/catalog and telemetry remain planned; Langfuse ingestion is unverified. Original v1/v2 keep their old CLI.

Chapters: tutorial/v1/ and tutorial/v2/. Records: docs/devlog/. Keep one root implementation; commits/tags identify chapter code.

Read the relevant design and task before implementation:

- [Design](docs/design.md)
- [Implementation and learning plan](docs/plan.md)

The user's latest instructions take precedence over those documents.

## How to develop

- Keep AGENTS.md current. Track requirements/status in docs/todo.md, design in docs/roadmap.md, and history/evidence in docs/devlog/.
- Continue the approved plan without asking again between tasks. Maintain its progress ledger and resume from completed commits after compaction.

- Work in small increments, each with an observable behavior, an appropriate check, and a short explanation of what it teaches.
- Keep the current milestone focused. Introduce abstractions when a concrete need appears.
- Prefer concrete functions and small types. Use standard Rust and basic HTTP/JSON/async libraries.
- Maintain one implementation. Each completed version keeps one milestone/vN learning branch and immutable tags; same-version usability fixes may advance that branch. Use step commits/tags for chapters; continue new work on codex/*.
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
- Treat model/Todo completion and independently verified task success as separate observations.
- In v2, keep execution records separate from the model's current input; commit compacted history only after validating the candidate.

## Verification and Git

Baseline: cargo fmt --check, cargo clippy --locked --all-targets -- -D warnings, cargo test --locked. Add focused checks for changed behavior. GitHub CI runs these plus Python tests on main push/PR; no model keys.

Distinguish deterministic protocol tests, real local tool execution, and live-model runs. Report only what the actual evidence establishes.

Commit Cargo.lock. Verify, merge and push each iteration to main; preserve teaching commits. Keep main, milestone/* and immutable v*/step-* tags. Develop on short-lived codex/* branches; remove them after commit-retention checks. No routine history rewriting. Run scripts/count-lines.py --max 399 on v1. Keep one learning branch per major stage; see docs/git-workflow.md.

Use rustfmt (100 columns, small heuristics Max), but expand protocol JSON for teaching clarity. Count all src/**/*.rs; retain historical milestone counts and report current counts separately. Never hide runtime code in scripts or docs.

Keep credentials, local environment files, generated workspaces, and raw session logs out of Git. Publish only deliberately selected and sanitized learning evidence.
