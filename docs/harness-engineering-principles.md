# Harness Engineering Principles

A harness is the small amount of shared support code, tooling, and documentation that lets each toy run predictably. In this repository, harness work should stay lightweight and serve the experiments instead of becoming the product.

## What Belongs in the Harness

Good harness code usually handles:

- terminal setup and cleanup
- drawing helpers
- input polling
- timing and frame pacing
- deterministic simulation steps
- common coordinate or layout types
- small test helpers
- repeatable commands for formatting, linting, and testing

Avoid adding harness code for a single speculative use. Let two or three toys prove the pattern first, then extract the shared part.

## Design Rules

### Keep boundaries obvious

- `crates/*` should contain the personality and rules of each toy.
- `libs/*` should contain reusable mechanics with clear names and minimal policy.
- A toy can duplicate a little code while the idea is still changing.
- Extract shared code when duplication starts hiding behavior or causing inconsistent fixes.

### Make cleanup hard to forget

- Prefer RAII guards for terminal modes, cursor visibility, raw mode, alternate screen, and other global terminal state.
- Cleanup should run on ordinary returns and propagated errors.
- If a terminal mode cannot be safely restored in `Drop`, document the limitation and keep the affected scope small.

### Make behavior observable

- Keep run commands in the README or the toy's local README.
- Prefer deterministic state transitions where practical, especially for examples and tests.
- For animations, isolate update logic from rendering when it makes tests or future changes easier.
- Use names that expose intent: `FrameClock`, `TerminalGuard`, `Point`, `Velocity`, `GameState`.

### Keep failures recoverable

- Propagate `io::Result` or typed errors from harness code.
- Avoid panics in shared runtime helpers unless they protect an internal invariant.
- Do not silently swallow setup errors; a failed terminal operation should be visible to the caller.
- Use best-effort cleanup in `Drop`, but do not use `Drop` as the only place that important errors can be reported.

### Stay dependency-conscious

- A shared harness dependency affects every crate that uses it, so add one only when it earns its weight.
- Prefer standard library code for simple terminal toys.
- Reach for proven crates when the domain is non-trivial, such as terminal UI, raw input, async runtimes, or parsing.
- Keep feature flags simple until there is a real need for optional behavior.

## Lightweight Extraction Checklist

Before moving code into `libs/terminal-toy-kit`, check:

- At least two toys need the behavior, or the behavior protects terminal safety.
- The API is smaller than the duplicated code it replaces.
- The helper can be named without explaining the original toy.
- Error behavior is explicit.
- A focused test or example can lock down the important behavior.
- The README still shows simple commands for running the toys.

## Quality Gate

For normal harness changes, run:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

For changes that affect terminal modes, also run at least one affected toy manually and verify that the terminal is restored after exit.
