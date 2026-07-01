# Terminal Toys

Small Rust terminal experiments for learning the language through visual, playful programs.

## Run

```bash
cargo run -p flutter-bird
cargo run -p bouncing-ball
cargo run -p matrix-rain
cargo run -p black-hole
```

Open `web/gargantua/index.html` for the browser-based black hole visualizer.

## Check Everything

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Layout

```text
crates/              Runnable terminal toys
libs/                Shared helper crates
web/                 Browser-based visual demos
```

Each app should start small and stay understandable. If two or three apps need the same drawing, timing, input, or cleanup logic, move that logic into `libs/terminal-toy-kit`.
