# AI-Assisted Development

AI can help move this personal project faster, but changes should still be understandable, reviewable, and owned by the maintainer.

## Operating Principles

- Ask AI agents to inspect the repo before proposing structure.
- Keep prompts scoped to one toy, one harness improvement, or one documentation update.
- Prefer small patches that can be reviewed in a few minutes.
- Require explanations for new abstractions, new dependencies, unsafe code, or broad rewrites.
- Treat generated code as a draft until it builds, is formatted, and matches the repository's style.

## Review Checklist

Before accepting AI-generated changes, check:

- The change solves the requested problem without unrelated refactoring.
- The code follows existing workspace layout and naming.
- Terminal state is restored after errors and exits.
- Errors are propagated instead of hidden.
- Dependencies are justified.
- Tests or manual verification match the risk of the change.
- Documentation was updated when commands, behavior, or project structure changed.

## Prompt Template

Use this when asking an AI agent for a change:

```text
Inspect the Rust workspace first. Make a small, idiomatic change that fits the current repo style.

Goal:
<what should change>

Constraints:
- Keep the project lightweight.
- Prefer existing patterns and `libs/terminal-toy-kit` for shared terminal behavior.
- Do not add dependencies unless clearly justified.
- Run the relevant cargo checks and summarize the result.
```

## Default Verification

For most changes:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

For visual demos, also run or open the affected toy and verify the visible behavior manually.
