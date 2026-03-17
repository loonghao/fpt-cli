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

coverage:
    vx cargo llvm-cov --workspace --lcov --output-path lcov.info

coverage-html:
    vx cargo llvm-cov --workspace --html

hakari-generate:
    vx cargo hakari generate
    vx cargo hakari manage-deps -y

hakari-check:
    vx cargo hakari generate --diff
    vx cargo hakari manage-deps --dry-run
    vx cargo hakari verify

pre-commit-install:
    vx python -m pip install pre-commit
    vx python -m pre_commit install --install-hooks --hook-type pre-commit --hook-type pre-push

pre-commit-run:
    vx python -m pre_commit run --all-files

ci:
    vx just fmt-check
    vx just hakari-check
    vx just check
    vx just lint
    vx just test
    vx just package-skills


package-skills:
    vx uv run python scripts/package_openclaw_skill.py skills dist/skills --all

package-openclaw-skill:
    vx uv run python scripts/package_openclaw_skill.py skills/fpt-cli-openclaw dist/skills


clawhub-sync-dry-run:
    vx npx clawhub@0.7.0 sync --root skills --all --dry-run --no-input

clawhub-sync:
    vx npx clawhub@0.7.0 sync --root skills --all --no-input


build-release *args:
    vx cargo build --release -p fpt-cli {{args}}

build-release-target target:
    vx cargo build --release -p fpt-cli --target {{target}}

release-version:
    vx uv run python scripts/release_metadata.py version

release-matrix:
    vx uv run python scripts/release_metadata.py matrix

verify-release-tag tag:
    vx uv run python scripts/release_metadata.py verify-tag {{tag}}

run *args:
    vx cargo run -p fpt-cli -- {{args}}


capabilities:
    vx cargo run -p fpt-cli -- capabilities --output json
