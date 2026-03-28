# fpt-cli Automation Memory

## Last Run: 2026-03-28 (sixteenth pass — code cleanup cycle #8)

### Project State
- **Total CommandSpecs**: 68 (all registered in capability.rs)
- **Total Tests**: 118 across test files in 3 crates (all passed)
- **Code Quality**: Zero clippy warnings, zero fmt diffs, all tests passing

### What Was Done This Run
- Created branch `chore/code-cleanup-2026-03-28-v4` from `origin/main`
- **Extracted `capability/preferences.rs` module** — moved `PREFERENCES_GET_SPEC`, `PREFERENCES_UPDATE_SPEC`, `PREFERENCES_CUSTOM_ENTITY_SPEC` from `capability/activity.rs` into a dedicated `capability/preferences.rs` module (these specs were misplaced in the activity module)
- **Eliminated unnecessary `.as_object().cloned()` clones** — replaced with `let Value::Object(mut object) = input else { ... }` pattern matching on owned `Value` in `summarize.rs` (1 site) and `work_schedule.rs` (2 sites)
- **Optimized `extract_rpc_results` in `transport.rs`** — uses `map.remove("results")` instead of `.get("results").cloned()` to avoid deep-cloning the RPC results Value
- **Reused `update_available` variable in `self_update.rs`** — eliminated redundant `release_version > current_version` recomputation
- **Replaced magic string `"rest"` with `TRANSPORT_REST` constant** in `self_update.rs` (2 sites)
- Net change: +107/-96 across 7 files
- All local checks passed: `cargo fmt`, `cargo clippy` (zero warnings), `cargo test` (all 118 passed)
- PR #111 squash-merged to main

### Previous Runs
- **Fifteenth pass**: Replaced magic strings in `note.rs`, extracted `dry_run_response()` helper in `entity.rs`, consistent batch result helpers in `batch.rs`, extracted `entity_instance_path()` in `transport.rs`. PR #109 merged.
- **Fourteenth pass**: Extracted transport constants, added doc comments, simplified `string_list_to_csv`, fixed `OnConflict::Error` id display. PR #108 merged.
- **Thirteenth pass**: OnceLock caching for env vars, `RequestPlan.notes` to `Vec<&'static str>`, `TOKEN_CACHE_POISONED` consistency, `.with_operation()` on 13 `AppError` sites in `self_update.rs`. PR #107 merged.
- **Twelfth pass**: TOKEN_CACHE_POISONED constant, visibility narrowing, httpmock 0.8 API migration, workspace-hack fix. PR #103 merged.
- **Eleventh pass**: entity relationship delete. PR #100 merged.
- **Tenth pass**: entity relationship write + entity share. PR #99 merged.
- **Ninth pass**: preferences update + note reply update/delete. PR #98 merged.
- **Eighth pass**: Exposed remaining unregistered CLI commands (user.current, note.reply-read, filmstrip.url). PR #94 merged.
- **Seventh pass**: self_update map_err consolidation + edge-case tests. PR #92 merged.
- **Sixth pass**: Batch helpers, retry dedup, visibility tightening. PR #89 merged.
- **Fifth pass**: Filter deduplication and module cleanup. PR #88 merged.
- **Fourth pass**: Code cleanup round 4 — deduplicated errors and helpers. PR #86 merged.
- **Third pass**: Comprehensive test coverage — 27 REST transport tests + 10 CLI contract tests. PR #87 merged.
- **Second pass**: Code cleanup — fixed formatting diffs. PR #83 merged.
- **First pass**: Added 10 new tests for `inspect.command` and `entity.batch.summarize` error paths. PR #82 merged.

### Known Remaining Gaps (Low Priority)
- `self.update` core logic (download/verify/replace) has no unit tests — intentionally skipped as it requires real binary replacement
- **Batch executor pattern**: 8+ batch methods in `batch.rs` share stream/collect/sort boilerplate — could extract a generic executor, but the risk/reward is marginal
- **`entity_batch_upsert`** (278 lines) — large function but complex branching makes splitting risky without adding bugs
- **`build_find_params`** (183 lines in `find.rs`) — could split into sub-functions for readability
- **`ConnectionSettings::resolve`** (120 lines in `config.rs`) — could extract env var reading helpers
- **Retry loop duplication** — `authorized_json_request` and `authorized_search_request` share ~50 lines of retry logic that could be extracted into a generic retry helper
- **`access_token_response`** (104 lines) — multiple responsibilities (cache check, form building, HTTP call, parsing) could be decomposed
- **`self_update::run`** (107 lines) — could be decomposed into smaller functions
- **`include_archived_projects`** logic in `summarize.rs` — counter-intuitive conditional (only inserts when `false`)
- **`has_any_set_arg`** in `config.rs` — 9 `.is_some()` checks maintenance risk
- **`translate_note_threads_error`** in `note.rs` — 5-level deep `and_then`/`is_some_and` chain
- **`serialized_body.clone()` in retry loop** (`transport.rs:747`) — clones `Vec<u8>` each iteration; could use `bytes::Bytes` for zero-copy clone, but marginal benefit
- **Remaining ShotGrid REST API endpoints not yet integrated**:
  - Server-side `_batch` API (POST /api/v1/entity/_batch) — current batch is client-side orchestration
  - Thumbnail/image upload (PUT /entity/{type}/{id}/image) — only GET url exists
  - Actual file upload via S3 presigned URL flow — only upload_url is implemented

### Architecture Notes
- Three-crate workspace: `fpt-cli` (binary), `fpt-core` (shared types), `fpt-domain` (business logic)
- `ShotgridTransport` trait with 49 async methods — all fully implemented in `RestTransport`
- 68 CommandSpecs registered in capability.rs — complete CLI surface
- `RecordingTransport` mock used in all domain tests
- Batch operations use `futures::stream::buffer_unordered` with configurable concurrency (default 8, max 32)
- Env var reads for `FPT_DEBUG`, `FPT_MAX_RETRIES`, `FPT_BATCH_CONCURRENCY` are now cached via `OnceLock`
- Transport magic strings replaced with named constants: `TRANSPORT_REST`, `TRANSPORT_RPC`, `SHOTGRID_SEARCH_CONTENT_TYPE`
- Capability specs properly organized: `activity.rs` (activity stream + event log), `preferences.rs` (preferences get/update/custom-entity)
