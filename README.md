# Terminal Toys

Small Rust terminal experiments for learning the language through visual, playful programs.

## Run

```bash
cargo run -p flutter-bird
cargo run -p bouncing-ball
cargo run -p matrix-rain
cargo run -p black-hole
cargo run -p launch-tracker
```

Open `web/gargantua/index.html` for the browser-based black hole visualizer.

`launch-tracker` is a live Ratatui dashboard for the next five rocket launches.
Use the arrow keys or `j`/`k` to select a mission, `r` to refresh, and `q` to
quit. Its countdown bar fills during the final seven days before an exact launch
time. Launches with only an estimated date remain visible and are marked `TBD`.
Launch data comes from the freely available
[RocketLaunch.Live feed](https://fdo.rocketlaunch.live/json/launches/next/5).

## Check Everything

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Engineering Notes

- [Engineering principles](docs/engineering-principles.md)
- [Harness engineering principles](docs/harness-engineering-principles.md)
- [AI-assisted development](docs/ai-assisted-development.md)

## Layout

```text
crates/              Runnable terminal toys
libs/                Shared helper crates
web/                 Browser-based visual demos
```

Each app should start small and stay understandable. If two or three apps need the same drawing, timing, input, or cleanup logic, move that logic into `libs/terminal-toy-kit`.
