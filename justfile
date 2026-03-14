set windows-shell := ["pwsh.exe", "-NoLogo", "-NoProfile", "-Command"]


default:
    @vx just --list

fmt:
    vx cargo fmt --all

check:
    vx cargo check --workspace

lint:
    vx cargo clippy --workspace --all-targets -- -D warnings

test:
    vx cargo test --workspace

run *args:
    vx cargo run -p fpt-cli -- {{args}}

capabilities:
    vx cargo run -p fpt-cli -- capabilities --output json
