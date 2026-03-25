# fpt-cli Code Cleanup Automation Memory

## Last Run: 2026-03-24 (cycle 3)

### Findings

1. **PR #76** (`chore/code-cleanup-2026-03-24`) — merged in cycle 1
   - Extracted `DRY_RUN_NOTE` constant to eliminate 4 duplicate string literals in `transport.rs`
   - Used `reqwest::header::{ACCEPT, CONTENT_TYPE}` constants for consistent HTTP header casing
   - Deduplicated `print_stdout`/`print_stderr` via generic `format_value` helper in `output.rs`
   - Renamed `parse_batch_delete_input` → `parse_batch_id_list_input` (also used by revive)
   - Added `count_by_action` helper to replace 4 repeated `.iter().filter()` patterns in batch upsert
   - Renamed `build_query_params_public` → `build_common_query_params` for clarity

2. **PR #77** (`style/fix-cargo-fmt-2026-03-24`) — merged in cycle 2
   - Applied `cargo fmt` to files changed by PR #76 that had formatting inconsistencies
   - Fixed multiline `#[command(...)]` attributes in `commands.rs`
   - Normalized match arm formatting in `output.rs`
   - Fixed error builder chain indentation in `batch.rs`
   - Removed extra blank line in `app_command_tests.rs`

3. **PR #90** (`chore/type-unification-cleanup-2026-03-24-2`) — merged in cycle 3
   - Extracted `push_page_params` helper in `query_helpers.rs` to deduplicate identical page-parsing logic between `build_find_params` (find.rs:97-118) and `build_query_params` (query_helpers.rs:109-130)
   - Removed trailing whitespace in `config.rs` (line 49, after `.with_allowed_values(...)`)

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

- The codebase is in excellent shape after three cleanup PRs in this cycle
- The `scripts/local_count_projects.ps1` (466 lines) uses Chinese comments, inconsistent with the English-only development convention, but it's a local debugging script — low priority
- Two `#![allow(clippy::result_large_err)]` at crate level are acceptable for the AppError pattern
- One `#[allow(clippy::too_many_arguments)]` on `entity_batch_upsert` (9 params) is acceptable — could consider an `UpsertOptions` struct in a future refactor, but not urgent
- The `entity_collection_path` function in `transport.rs` has a hand-rolled CamelCase→snake_case conversion (50 lines) that could theoretically use the `heck` crate, but it has ShotGrid-specific pluralization logic that makes a crate swap non-trivial — leave as-is
- Batch operations in `batch.rs` have repetitive template patterns that could benefit from a generic `run_batch` higher-order function in a future refactor — medium complexity, not urgent
