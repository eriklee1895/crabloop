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

This is the consolidated milestone/v1 learning branch, including early terminal interaction. Historical v1 and step-05-v1 tags retain the original 314 code lines / 319 physical lines. This branch has 571 code lines / 581 physical lines across all src/**/*.rs, including the CLI and terminal interface. This exceeds the newly required <400 cap and is pending scope reduction, not an accepted completed v1. No runtime code is hidden outside src/. v2 context projection, compression, journals and AGENTS.md loading are absent here.

The root Cargo package contains the hand-written Responses loop and four bounded tools. `Session` owns the client and full in-memory Responses history across user turns, including final answers and call/result associations. `run(Config)` remains the one-shot wrapper. Context is bounded by serialized request bytes (default 131,072; CRABLOOP_CONTEXT_BYTES overrides it, minimum 4,096); v1 stops at the limit without compaction. Histories disappear on exit. `/new` clears only the conversation, retaining files. Protocol/transport errors and interruptions poison the session until `/new`, without automatic tool replay.

`cargo run` enters a scrolling terminal chat in a TTY. `-p/--print TASK` runs once; positional tasks and whole-stdin one-shot input remain supported. `--chat` forces line-by-line chat, including pipes. rustyline provides input editing and in-memory history. Ctrl-C cancels; Ctrl-D and `/exit` quit. This version has no streaming or full-screen layout.

Configuration reads only the startup directory's .env as data with dotenvy; it neither executes that file nor mutates process environment. Process values win. Prefer DEEPSEEK_API_KEY/deepseek-flash without an explicit generic model/endpoint; CRABLOOP_PROVIDER=deepseek/openai overrides selection. Generic mode uses OPENAI_API_KEY/OPENAI_BASE_URL/CRABLOOP_MODEL; CRABLOOP_API_URL is a full endpoint override. Credentials stay paired with the selected provider. Home/TOML/catalog remain later work.

Original v1 live tasks passed on DeepSeek V4.1 Flash and gpt-6-luna; the interaction changes have offline protocol, real local tool, CLI and session checks. No live-model run was performed for this backport. Current checks: 43 Rust and 3 Python tests, fmt and clippy. Tutorials 00–05 retain the original v1 path; tutorial/v1/06-conversation.md adds the interactive step. Detailed changes and verification are in docs/devlog/06-v1-interactive.md.

Read docs/design.md and docs/plan.md as the frozen v1 → v2 design; the interaction addendum and latest user instructions take precedence. Keep one implementation at the repository root, and identify learning states with Git commits and immutable tags.

## How to develop

- Keep AGENTS.md current throughout implementation, as requested by the user. Update it when repository state, interfaces, commands, or durable decisions change. Put detailed history and evidence in docs/devlog/ rather than accumulating a session transcript here.
- Keep v1 focused on the smallest readable loop. Reduce the existing 571-line scope before declaring it complete; richer interaction/configuration belongs later. v1/v2 stay non-streaming; basic SSE moves to v3.

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

Commit Cargo.lock. The user authorizes verified increments to be committed and pushed, with main holding the latest validated tree. The user requested one v1 branch: milestone/v1 includes the interaction improvements. Keep main and milestone/v2 separately, and preserve historical v1/v2 and step tags. Do not create another interaction-version branch/tag or merge this older backend over main.

Formatting uses rustfmt with max_width = 100 and use_small_heuristics = Max; expand protocol JSON for teaching clarity. Count all src/**/*.rs under this same public style for every milestone; never hide runtime code in scripts or documentation.

Keep credentials, local environment files, generated workspaces, and raw session logs out of Git. Publish only deliberately selected and sanitized learning evidence.
