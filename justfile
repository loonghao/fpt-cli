set windows-shell := ["pwsh.exe", "-NoLogo", "-NoProfile", "-Command"]


default:
    @vx just --list

fmt:
    vx cargo fmt --all

fmt-check:
    vx cargo fmt --all -- --check

check:
    vx cargo check --workspace

lint:
    vx cargo clippy --workspace --all-targets -- -D warnings

test:
    vx cargo test --workspace

pre-commit-install:
    vx python -m pip install pre-commit
    vx python -m pre_commit install --install-hooks --hook-type pre-commit --hook-type pre-push

pre-commit-run:
    vx python -m pre_commit run --all-files

ci:
    vx just fmt-check
    vx just check
    vx just lint
    vx just test
    vx just package-skills

package-skills:
    vx python scripts/package_openclaw_skill.py skills dist/skills --all

package-openclaw-skill:
    vx python scripts/package_openclaw_skill.py skills/fpt-cli-openclaw dist/skills

clawhub-sync-dry-run:
    vx npx clawhub@0.7.0 sync --root skills --all --dry-run --no-input

clawhub-sync:
    vx npx clawhub@0.7.0 sync --root skills --all --no-input


build-release-locked *args:
    vx cargo build --release --locked -p fpt-cli {{args}}

run *args:
    vx cargo run -p fpt-cli -- {{args}}


capabilities:
    vx cargo run -p fpt-cli -- capabilities --output json

