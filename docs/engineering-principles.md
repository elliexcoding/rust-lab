# Engineering Principles

This repository is a personal Rust lab for small terminal and browser visual experiments. The bar is not enterprise process; the bar is that each experiment remains easy to run, easy to understand, and safe to change.

These principles are adapted for this repo from OpenAI's public Charter principles of broad benefit, long-term safety, technical leadership, and cooperative orientation, plus its safety loop of teach, test, and share.

## Principles

### Build for learning and broad usefulness

- Prefer complete, runnable examples over clever fragments.
- Keep each toy understandable to someone learning Rust, terminal rendering, or visual simulation.
- Favor explicit names, simple state, and local control flow over abstractions that only save a few lines.
- Make demos work from the README commands without hidden setup.

### Keep safety practical

- Terminal apps must leave the terminal in a usable state after normal exit or error.
- Any loop should have a visible exit path, a bounded demo duration, or a clear input control.
- Avoid writing outside the repository unless the user explicitly asks for generated output elsewhere.
- Do not add network calls, background services, telemetry, or persistent local state without a specific reason.
- Treat panic, `unwrap`, and `expect` in production paths as design choices that need a local invariant.

### Choose technical quality over scale

- Start with the smallest working toy, then improve the parts that make future toys easier.
- Put shared terminal, timing, input, and cleanup behavior in `libs/terminal-toy-kit` once at least two or three crates need it.
- Prefer Rust's type system for states, coordinates, timing, and errors when that makes incorrect code harder to write.
- Add dependencies only when they clearly improve correctness, ergonomics, or learning value.
- Keep browser demos dependency-light unless a library is central to the concept being explored.

### Test what matters

- Use `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` as the normal quality gate.
- Test shared library behavior before testing one-off toy rendering details.
- For terminal behavior, prefer small tests around coordinate math, state transitions, parsing, or cleanup helpers.
- For browser demos, keep the app directly openable unless the feature truly needs a build step.

### Share context with future maintainers

- Document why a toy exists, what command runs it, and what tradeoff it intentionally makes.
- Keep comments focused on non-obvious behavior, terminal escape sequences, or safety constraints.
- If an experiment is intentionally approximate, say so near the code or README instead of hiding the limitation.
- Leave the repository easier to navigate after each change.

## OpenAI Alignment Notes

For this repo, "aligned with OpenAI's principles" means:

- **Broad benefit:** demos should be accessible, reproducible, and educational.
- **Long-term safety:** changes should reduce surprise, protect the local environment, and keep failure modes recoverable.
- **Technical leadership:** prefer high-quality, idiomatic implementation even in small experiments.
- **Cooperative orientation:** write code and notes that help future contributors, including future you.
- **Teach, test, share:** make behavior legible, validate important paths, and capture the useful lesson.

References:

- [OpenAI Charter](https://openai.com/charter/)
- [OpenAI Safety](https://openai.com/safety/)
