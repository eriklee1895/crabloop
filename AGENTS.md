# CrabLoop — contributor and agent guide

## Project goal

CrabLoop is a small coding-agent harness built from scratch in Rust for hands-on learning.

- v1: target about 300 lines, HARD cap below 400 total production Rust code lines. Teach the hand-written model/tool/feedback/stop loop with four tools.
- v2: approximately 500 lines total, adding context management and summary compaction.
- v3: approximately 1,000 lines total, adding ~/.crabloop, TOML configuration, provider presets/Custom, credentials, and model catalog discovery/cache. Minimal Skills is optional if comfortably within scope.
- v4: Skills and provider-independent web_fetch/web_search. v5: recoverable JSONL sessions and interruption handling under ~/.crabloop. See docs/roadmap.md; these stages are planned, not implemented.
- Acceptance includes both coding repairs and general local agentic tasks such as reading source documents and producing a verifiable structured report.
- Use the OpenAI native Responses API and a hand-written agent loop.
- Grow the same implementation through small, explainable steps. Preserve runnable milestones with v1 and v2 Git tags.

The user tightened v1 scope on 2026-10-12: <400 is mandatory, superseding earlier expansion for interactivity. Count all production src/**/*.rs with scripts/count-lines.py after normal rustfmt; report physical lines too. Tests/docs/fixtures/dev scripts are separate. Reduce features, never hide code, code-golf, outsource the loop, or drop necessary error handling. See docs/v1-scope.md.

## Current state and layout

The milestone/v1 implementation has been reduced to 398 code lines / 403 physical lines under the established counting rule. The old 571-line interaction expansion remains in Git history and main. Original v1 / step-05-v1 tags retain 314/319. Production Rust lives in five files; run(Config) directly owns history and the serial model/tool loop. No Session, Progress, terminal editor, cross-user-turn chat or SSE remains in v1.

Startup is a single task: cargo run prompts for one line in a terminal, reads all piped stdin, or accepts -p/--print TASK (legacy positional TASK also works). The workspace is cwd; change directories before invoking the binary. Empty input exits without a model request. --chat and --workspace are not supported here. The request budget is fixed at 131,072 bytes; other limits and errors remain intact.

Only the startup directory's .env is read as data; process environment wins. CRABLOOP_PROVIDER defaults to deepseek (DEEPSEEK_API_KEY, deepseek-flash). Explicit openai uses OPENAI_API_KEY, OPENAI_BASE_URL (default https://api.openai.com/v1) and default gpt-6-luna. CRABLOOP_MODEL and full CRABLOOP_API_URL override the selected profile. No model-name, hostname or key-presence inference. The live test driver explicitly selects openai while supplying the chosen provider's matching key/base/model; it runs the binary from the fixture directory. Never execute or print .env contents.

Current offline verification: 40 Rust and 4 Python tests; fmt/clippy and scripts/count-lines.py --max 399 passed. DeepSeek live repair passed (9 tool calls, independent verifier green), and the final whole-change review found no runtime blockers. Review cleanup also rejects unsupported live-driver v2/context flags and clarifies the historical chapter checkout. Historical live evidence remains distinct. Tests for removed chat functionality remain on main; v1 retains core protocol, bounded tools, feedback/error loops and single-task CLI coverage.

Read docs/v1-scope.md and docs/v1-reduction-plan.md for this accepted reduction; docs/design.md and docs/plan.md preserve the original learning sequence. Tutorial 06-conversation is a later interaction experiment preserved through its historical commit, not a current v1 capability.

## How to develop

- Keep AGENTS.md current throughout implementation, as requested by the user. Update it when repository state, interfaces, commands, or durable decisions change. Put detailed history and evidence in docs/devlog/ rather than accumulating a session transcript here.
- Keep v1 focused on the smallest readable loop. Keep all production Rust below 400 lines; richer interaction/configuration belongs later. v1/v2 stay non-streaming; basic SSE moves to v3.

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

Commit Cargo.lock. The user authorizes verified increments to be committed and pushed, with main holding the latest validated tree. The user requested one v1 branch: milestone/v1 is the reduced single-task teaching core. Keep main and milestone/v2 separately, and preserve historical v1/v2 and step tags. Do not create another interaction-version branch/tag or merge this older backend over main.

Formatting uses rustfmt with max_width = 100 and use_small_heuristics = Max; expand protocol JSON for teaching clarity. Count all src/**/*.rs under this same public style for every milestone; never hide runtime code in scripts or documentation.

Keep credentials, local environment files, generated workspaces, and raw session logs out of Git. Publish only deliberately selected and sanitized learning evidence.
