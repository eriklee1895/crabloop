# CrabLoop — contributor and agent guide

## Project goal

CrabLoop is a small coding-agent harness built from scratch in Rust for hands-on learning.

- v1: target about 300 lines, with size used as a teaching guideline rather than a hard gate. Teach the hand-written model/tool/feedback/stop loop with four tools.
- v2: approximately 500 lines total, adding context management and summary compaction.
- v3: approximately 1,000 lines total, adding ~/.crabloop, TOML configuration, provider presets/Custom, credentials, and model catalog discovery/cache. Minimal Skills is optional if comfortably within scope.
- v4: Skills and provider-independent web_fetch/web_search. v5: recoverable JSONL sessions and interruption handling under ~/.crabloop. See docs/roadmap.md; these stages are planned, not implemented.
- Acceptance includes both coding repairs and general local agentic tasks such as reading source documents and producing a verifiable structured report.
- Use the OpenAI native Responses API and a hand-written agent loop.
- Grow the same implementation through small, explainable steps. Preserve runnable milestones with v1 and v2 Git tags.

2026-10-14: v1 line counts are guidance, not a gate. Keep code small/readable; count-lines.py is optional. Do not hide runtime code, code-golf, outsource the loop, or drop errors to meet a number. See docs/v1-scope.md.

## Current state and layout

The only v1 learning code is milestone/v1. Seven tutorial patches end at the same complete src tree; scripts/check-lessons.py verifies it. Historical tags preserve steps and evidence, not alternative learning entries.

The milestone/v1 implementation has been reduced to 395 code lines / 456 physical lines under the established counting rule. Default tool display shows name, argument, status and a 240-byte preview. Original v1 / step-05-v1 tags retain 314/319. Production Rust lives in five files; run(Config) directly owns history and the serial model/tool loop. No Session, Progress, terminal editor, cross-user-turn chat or SSE remains in v1.

Startup is a single task from stdin: cargo run shows a small crab banner and crab> prompt before reading one line in a terminal, or reads all piped stdin without the banner. All command-line arguments (including positional tasks, -p/--print and -h/--help) are rejected before input/configuration with an English error pointing to README.md. The workspace is cwd; change directories before invoking the binary. Empty input exits before configuration and without a model request. --chat and --workspace are not supported here. The request budget is fixed at 4,194,304 bytes (4 MiB); max_output_tokens is 131,072; other limits and errors remain intact.

Only the startup directory's .env is read as data; process environment wins. CRABLOOP_PROVIDER defaults to deepseek (DEEPSEEK_API_KEY, deepseek-flash). Explicit openai uses OPENAI_API_KEY, OPENAI_BASE_URL (default https://api.openai.com/v1) and default gpt-6-luna. CRABLOOP_MODEL and full CRABLOOP_API_URL override the selected profile. No model-name, hostname or key-presence inference. The live test driver explicitly selects openai while supplying the chosen provider's matching key/base/model; it runs the binary from the fixture directory. Never execute or print .env contents.

CLI Help remains planned in docs/cli-help-plan.md. Code strings/comments are English; tutorials are Chinese.

GA evidence: evidence/v1/ga-review.md.

Offline baseline: 43 Rust + 5 Python tests, fmt/clippy. Live repair/report and PTY results are historical evidence in evidence/v1/ga-review.md. No chat functionality remains in v1.

Read docs/v1-scope.md and docs/v1-reduction-plan.md for this accepted reduction; docs/design.md and docs/plan.md preserve the original learning sequence. Tutorial 06-conversation is a later interaction experiment preserved through its historical commit, not a current v1 capability.

## How to develop

- Keep AGENTS.md current throughout implementation, as requested by the user. Update it when repository state, interfaces, commands, or durable decisions change. Put detailed history and evidence in docs/devlog/ rather than accumulating a session transcript here.
- Keep v1 focused on the smallest readable loop. Use code size to guide scope rather than reject changes; richer interaction/configuration belongs later. v1/v2 stay non-streaming; basic SSE moves to v3.

- Work in small increments, each with an observable behavior, an appropriate check, and a short explanation of what it teaches.
- Keep the current milestone focused. Introduce abstractions when a concrete need appears.
- Prefer concrete functions and small types. Use standard Rust and basic HTTP/JSON/async libraries.
- Maintain one implementation. Use Git commits and milestone tags to show its evolution.
- Write tutorials and explanations in Chinese, with Rust identifiers and protocol names in their original form.
- Keep the v1 README concise: use "300+ lines" for size, retain startup and learning entrypoints, and omit verification reports, budget parameters and development history. Detailed contributor information belongs in dedicated documents.
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

Separate top-level items and logical phases within functions with one blank line; keep related statements together. Blank lines do not count toward the code-line budget. Formatting uses rustfmt with max_width = 100 and use_small_heuristics = Max; expand protocol JSON for teaching clarity. Count all src/**/*.rs under this same public style for every milestone; never hide runtime code in scripts or documentation.

Keep credentials, local environment files, generated workspaces, and raw session logs out of Git. Publish only deliberately selected and sanitized learning evidence.
