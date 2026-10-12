# CrabLoop — contributor and agent guide

## Project goal

CrabLoop is a small coding-agent harness built from scratch in Rust for hands-on learning.

- v1: approximately 300 lines of core Rust, with read_file, write_file, edit_file, and bash.
- v2: approximately 500 lines total, adding context management and summary compaction.
- v3: approximately 1,000 lines total, adding ~/.crabloop, TOML configuration, provider presets/Custom, credentials, and model catalog discovery/cache. Minimal Skills is optional if comfortably within scope.
- Planned v4: Skills, Tavily Search/Firecrawl Scrape, a config menu sharing TOML files with manual edits, selective .env import and visible config sources. Planned v5: recoverable JSONL sessions and interruption handling under ~/.crabloop. See docs/roadmap.md.
- Basic multi-turn terminal chat belongs in the early learning stage. v6 adds streaming and Ratatui/Crossterm layout; v7 adds scoped memory; optional v8 compares a pinned Rig backend. See docs/roadmap.md.
- Acceptance includes both coding repairs and general local agentic tasks such as reading source documents and producing a verifiable structured report.
- Use the OpenAI native Responses API and a hand-written agent loop.
- Grow the same implementation through small, explainable steps. Preserve runnable milestones with v1 and v2 Git tags.

Line counts guide scope; v1 at 350–360 is accepted. Count all production Rust under src/; tests/docs/fixtures/dev scripts are separate. Keep readable formatting and error handling; never code-golf or hide implementation. Accept correct extra output fields.

## Current state and layout

Seven patches rebuild v1 from scratch; see tutorial/v1/patches/README.md. Keep code and Chinese tutorials together. The Cargo package, src/ and tests/ live at root.

Frozen v1 has 314 code lines (319 physical); frozen v2 has 504 (511), with stable milestone branches and immutable tags. The v1 readability patch yields 352/357. Current main has 813/825 including CLI/UI. v1.1 has 571/581, based on v1 without compaction or journals. Tutorials 00–09 and evidence/v1|v2/ preserve historical results. Checks: 61 Rust + 6 Python, fmt and clippy passed. Multi-turn live/PTY evidence and the genuine README recording are in evidence/interactive/. A minor mixed commentary/final-text display issue remains in evidence/v2/review.md.

Preferred model: DeepSeek V4.1 Flash (deepseek-flash); fallback: gpt-6-luna through TeamoRouter. Both passed v1 tasks. DeepSeek passed v2 with five compactions; Luna v2 is untested. Summaries use reasoning.effort=none and at least 4,096 output tokens; main requests use provider defaults. Request limit: 49,152 bytes; CRABLOOP_CONTEXT_BYTES overrides it (min 4,096). Root AGENTS.md is pinned and must stay below 8,192 bytes. Distinguish offline, live and semantic evidence.

`cargo run` enters multi-turn chat in cwd; `-p/--print TASK` runs once. Positional tasks and piped stdin remain supported; `--chat` forces chat. Rustyline handles editable input/history; /new resets, /exit or Ctrl-D quits, Ctrl-C cancels. Session retains final answers and tool groups. Errors/interruption require /new and file inspection; no automatic effect replay. Empty chat creates no journal or request. UI is scrolling, with whole-response text; streaming/full-screen layout is later.

`src/cli.rs` reads only cwd .env as data with dotenvy; process values win. Prefer DEEPSEEK_API_KEY/deepseek-flash absent explicit generic config; CRABLOOP_PROVIDER=deepseek/openai overrides. Generic mode uses OPENAI_API_KEY/OPENAI_BASE_URL/CRABLOOP_MODEL; CRABLOOP_API_URL overrides the endpoint. Keep provider keys paired. v3 home/TOML/catalog remains planned. Original v1/v2 keep their old CLI.

Chapters: tutorial/v1/ and tutorial/v2/. Records: docs/devlog/. Keep one root implementation; commits/tags identify chapter code.

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

Commit Cargo.lock. The user authorizes merging and pushing every verified iteration to main, not only full milestones. main holds the latest validated tree and the full teaching commit history. Develop on a short-lived branch, then fast-forward or merge into main without squashing learning steps. Preserve milestone/vN and immutable vN/step tags. See docs/git-workflow.md.

Use rustfmt (100 columns, small heuristics Max), but expand protocol JSON for teaching clarity. Count all src/**/*.rs; retain historical milestone counts and report current counts separately. Never hide runtime code in scripts or docs.

Keep credentials, local environment files, generated workspaces, and raw session logs out of Git. Publish only deliberately selected and sanitized learning evidence.
