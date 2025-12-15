# FireDBG Architecture & Context Notes
This is a curated, repo-local “mental model” for FireDBG: what it is, how the major parts interact, and where to look in this codebase.

It is distilled (paraphrased) from SeaQL’s FireDBG documentation/blog posts and related SeaStreamer docs.

## TL;DR
FireDBG is a Rust-focused “record + explore” debugger workflow:
- `firedbg` / **firedbg-cli** is a front-end that behaves like a `cargo` proxy (`cargo run` → `firedbg run`, `cargo test` → `firedbg test`, etc.).
- **parser** scans workspace sources and emits cached per-file symbol tables (`.map` files).
- **debugger** drives the target via LLDB and streams breakpoint events in real time into a trace file.
- The trace file is `.firedbg.ss`, using the SeaStreamer “.ss” container format.
- **indexer** is a streaming processor that converts `.firedbg.ss` into a `.sqlite` index (SeaORM-backed).
- A UI (commonly VS Code) visualizes call trees / stacks / values / timelines from the DB.

## Repository map (where things live)
Top-level directories in this repo roughly match subsystems:
- `command/` — `firedbg` CLI (orchestrates parse → run → index → open)
- `parser/` — parses Rust sources (via `syn`), produces cached “map” files used by the debugger
- `debugger/` — `firedbg-debugger` runtime engine, integrates with LLDB
- `protocol/` — event stream protocol definitions for `.firedbg.ss`
- `indexer/` — converts `.firedbg.ss` streams into `.sqlite` for fast queries
- `codelldb/` — LLDB bindings and related runtime glue

## The main pipeline (what happens on a run)
A typical session looks like:
1. **Target discovery**
   - CLI reads Cargo workspace metadata to find runnable targets (bin/example/test/unit-test).
2. **Parse/cache sources**
   - Parser walks the Rust AST and collects “breakable spans” for functions and methods.
   - Results are cached so incremental runs don’t re-parse unchanged files.
3. **Debug + record**
   - Debugger launches the target under LLDB.
   - Breakpoints are installed based on the cached parse results.
   - When breakpoints hit, FireDBG captures structured runtime data (calls/returns/values/panic/etc.) and appends it to a trace.
4. **Index**
   - Indexer reads the trace stream and builds a SQLite database optimized for UI queries.
5. **Visualize**
   - UI queries SQLite to render call trees, frames, value views, timelines, etc.

## “Galloping” / selective breakpoints
A central idea described in the architecture post is to keep debugging practical by making each breakpoint hit *as brief as possible*:
- Trace *primarily* “your code” (workspace members) and skip most library/system call noise.
- Resume quickly after collecting data.

This keeps the program-under-debug closer to real-time (important for time-sensitive things like sockets and timers) while still providing a “big picture” call-tree view.

## Breakpoints and events (runtime model)
FireDBG’s debugger engine captures events like:
- Function call
- Function return
- Panic
- Explicit “breakpoint events” (e.g. via macros like `fire::dbg!(...)` in examples)

The debugger maintains a logical stack model per thread.
- On each function call, it assigns a new frame ID.
- The tuple (thread ID, frame ID, function call) identifies a point in execution.

The indexer reconstructs call stacks/call trees (per thread) from the stream and stores them in SQL using self-references.

## Return value capture (hard part)
FireDBG captures parameters and return values.

The described return-value strategy:
- Disassemble a function the first time it’s called.
- Set breakpoints at all `ret` instructions.
- When a `ret` breakpoint hits, attempt to recover the return value (registers/stack) using the return type and ABI heuristics.

This is architecture-specific and inherently tricky. Expect edge cases for complex return types and compiler optimizations.

## Threading model
The debugger must cope with multiple threads:
- Multiple threads can be stopped by a breakpoint.
- Not all stopped threads necessarily correspond to “interesting” breakpoints.
- The engine needs to identify which threads are stopped *at relevant breakpoint sites*, record what’s needed, and then resume.

## Values: reading + serialization
Values are read via LLDB’s type/value APIs (SBType/SBValue) and then serialized.

A practical design choice is:
- Keep trace recording fast and streaming-friendly.
- Defer expensive transformations / pretty printing until indexing.

You’ll see this separation reflected in:
- debugger → writes raw/structured payloads to the stream
- indexer → interprets/pretty-prints payloads and stores query-friendly representations

## Trace file format: `.firedbg.ss` (SeaStreamer container)
The `.firedbg.ss` file is based on the SeaStreamer “.ss” file format (a stream/container format).

Key properties (high level):
- Append-friendly streaming
- Periodic internal indexing (“beacons”) enabling faster seeks
- Integrity checks

FireDBG uses the container to multiplex multiple logical streams (e.g. debugger info, source file table, breakpoints, runtime events) into a single file.

A useful mental model: the debugger streams out “raw facts” as the program runs; the indexer turns those facts into things that are fast to query.

## Indexer + SQLite (why it exists)
The UI wants fast, random-access queries:
- “show me the call tree for thread X”
- “jump to frame Y”
- “search events”
- “show the call chain for this frame”

The indexer converts the stream into a SQLite DB that supports these queries efficiently. It also does some basic analysis like counting breakpoint hits, and it transforms value blobs into pretty-printed Rust-like strings and JSON.

Repo pointers:
- `indexer/src/entity/` — schema representation

## UI mental model (from “Getting Started”)
This matters because the whole system is built around the call tree + frame IDs.

- Each node represents a function call.
- Nodes have a unique frame ID; frame ID is used as the “time” unit in the timebar.
- Edges visually distinguish “call only” vs “call with return value”.
- The timeline view uses different markers for call vs return.

If you’re writing tooling or diagnostics, it helps to treat “frame ID” as the stable handle that ties together:
- call tree node
- timeline position
- variable/value snapshots

## Parallelism (responsiveness)
The architecture post emphasizes that Debugger, Indexer, and GUI are separate processes, and each has internal producer/consumer parallelism. The UI rendering is incremental: nodes appear as data arrives.

## Practical entrypoints in this repo
If you’re reading code to understand the architecture:
- CLI entrypoint: `command/src/main.rs`
  - orchestrates caching, target selection, debugger invocation, indexer invocation
- CLI status output: `command/src/console.rs`
- Debugger engine: `debugger/`
- Protocol definitions: `protocol/`
- Indexing pipeline + schema: `indexer/`

## Notes for this fork
This fork focuses on “build-it-yourself” workflows:
- `install.sh` defaults to source builds.
- CodeLLDB/LLDB runtime assets are cached under a user cache directory (so repo checkouts stay clean).
- `firedbg doctor` exists to provide actionable diagnostics and setup hints.

## External reading list (primary sources)
- Architecture overview:
  - `https://firedbg.sea-ql.org/blog/2023-12-11-architecture-of-firedbg/`
- Introducing FireDBG (call tree framing, error path idea, return capture discussion, Rust value representation):
  - `https://firedbg.sea-ql.org/blog/2023-12-12-introducing-firedbg/`
- Getting Started (UI controls + CLI workflow walkthrough):
  - `https://firedbg.sea-ql.org/blog/2023-12-13-getting-started/`
- Visualizing Dynamic Programming (examples of how call tree shapes reflect algorithm properties):
  - `https://firedbg.sea-ql.org/blog/2024-01-31-visual-dynamic-program/`
- FizzBuzz Multithreaded (timeline + multi-thread visualization example):
  - `https://firedbg.sea-ql.org/blog/2024-06-30-fizzbuzz-multithread/`
- SeaStreamer file format (container framing):
  - `https://docs.rs/sea-streamer-file/latest/sea_streamer_file/format/index.html`
- SeaStreamer file crate (includes tooling like `ss-decode`):
  - `https://docs.rs/sea-streamer-file/`
