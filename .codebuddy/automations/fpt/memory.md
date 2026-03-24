# fpt-cli Code Cleanup Automation Memory

## Last Run: 2026-03-24

### Findings

1. **PR #76** (`chore/code-cleanup-2026-03-24`) — already merged from a prior run in this cycle
   - Extracted `DRY_RUN_NOTE` constant to eliminate 4 duplicate string literals in `transport.rs`
   - Used `reqwest::header::{ACCEPT, CONTENT_TYPE}` constants for consistent HTTP header casing
   - Deduplicated `print_stdout`/`print_stderr` via generic `format_value` helper in `output.rs`
   - Renamed `parse_batch_delete_input` → `parse_batch_id_list_input` (also used by revive)
   - Added `count_by_action` helper to replace 4 repeated `.iter().filter()` patterns in batch upsert
   - Renamed `build_query_params_public` → `build_common_query_params` for clarity

2. **PR #77** (`style/fix-cargo-fmt-2026-03-24`) — created and merged in this run
   - Applied `cargo fmt` to files changed by PR #76 that had formatting inconsistencies
   - Fixed multiline `#[command(...)]` attributes in `commands.rs`
   - Normalized match arm formatting in `output.rs`
   - Fixed error builder chain indentation in `batch.rs`
   - Removed extra blank line in `app_command_tests.rs`

### Codebase Health Summary

- `cargo fmt --check` ✅ (all files formatted)
- `cargo clippy --workspace -D warnings` ✅ (zero warnings)
- `cargo test --workspace` ✅ (all tests pass)
- No TODO/FIXME/HACK comments in source code
- No commented-out code blocks
- No `#[allow(dead_code)]` or `#[allow(unused)]` annotations
- No unused imports detected
- Consistent error handling patterns throughout
- Clean dependency graph in all Cargo.toml files

### Notes for Next Run

- The codebase is in excellent shape after these two cleanup PRs
- The `scripts/local_count_projects.ps1` (466 lines) uses Chinese comments, inconsistent with the English-only development convention, but it's a local debugging script — low priority
- Two `#![allow(clippy::result_large_err)]` at crate level are acceptable for the AppError pattern
- One `#[allow(clippy::too_many_arguments)]` on `entity_batch_upsert` (8 params) is acceptable
