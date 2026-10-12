# CrabLoop — contributor and agent guide

## Project goal

CrabLoop is a small coding-agent harness built from scratch in Rust for hands-on learning.

- v1: target about 300 lines, HARD cap below 400 total production Rust code lines. Teach the hand-written model/tool/feedback/stop loop with four tools.
- v2: approximately 500 lines total, adding context management and summary compaction.
- v3: approximately 1,000 lines total, adding ~/.crabloop, TOML configuration, provider presets/Custom, credentials, and model catalog discovery/cache. Minimal Skills is optional if comfortably within scope.
- Planned v4: Skills, Tavily Search/Firecrawl Scrape, a config menu sharing TOML files with manual edits, selective .env import and visible config sources. Planned v5: recoverable JSONL sessions and interruption handling under ~/.crabloop. See docs/roadmap.md.
- v1/v2 stay non-streaming; v3 adds basic SSE as a separate learning increment. Rich terminal layout is v6, memory v7, optional Rig comparison v8. Early product polish must not expand v1.
- Acceptance includes both coding repairs and general local agentic tasks such as reading source documents and producing a verifiable structured report.
- Use the OpenAI native Responses API and a hand-written agent loop.
- Grow the same implementation through small, explainable steps. Preserve runnable milestones with v1 and v2 Git tags.

The user tightened v1 scope on 2026-10-12: <400 is mandatory. Count all production src/**/*.rs with scripts/count-lines.py after normal rustfmt; report physical lines too. Tests/docs/fixtures/dev scripts are separate. Reduce features, never hide code, code-golf, outsource the loop, or drop necessary error handling. See docs/v1-scope.md.

## Current state and layout

Seven patches rebuild historical v1; see tutorial/v1/patches/README.md. Keep Chinese tutorials with code. Cargo/src/tests live at root.

Historical tags: v1/step-05-v1 314 code / 319 physical; v2 504/511. Main is 813/825. Current milestone/v1 is 398/403: direct run(Config), cwd, fixed budget, no Session/TUI. v1 checks: 40 Rust + 4 Python, live repair and PTY passed. Main checks: 61 Rust + 6 Python, fmt/clippy passed. Historical evidence stays separate. The mixed commentary/final display limitation is in evidence/v2/review.md.

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
- Maintain one implementation. For every completed version (v1 through v5 and future v6/v7), preserve a milestone/vN learning branch and immutable snapshot tags; same-version usability fixes may advance its branch. Use step commits/tags for chapter progression; continue new work on a development branch.
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

Commit Cargo.lock. Merge and push each verified iteration to main, preserving teaching commits. Keep main, milestone/* and immutable v*/step-* tags; remove completed temporary branches after verifying their commits are retained. Run scripts/count-lines.py --max 399 on v1. Develop on a short-lived codex/* branch. No history rewriting for routine cleanup. Keep one learning branch per major stage; see docs/git-workflow.md for branch/tag distinctions.

Use rustfmt (100 columns, small heuristics Max), but expand protocol JSON for teaching clarity. Count all src/**/*.rs; retain historical milestone counts and report current counts separately. Never hide runtime code in scripts or docs.

Keep credentials, local environment files, generated workspaces, and raw session logs out of Git. Publish only deliberately selected and sanitized learning evidence.
