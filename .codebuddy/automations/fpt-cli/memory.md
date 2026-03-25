# fpt-cli Automation Memory

## Last Run: 2026-03-24 (seventh pass — self_update map_err consolidation + edge-case tests)

### Project State
- **Total CommandSpecs**: 55 (all registered in capability.rs)
- **Total Tests**: 220+ across 11 test files in 3 crates (all passed)
- **ShotGrid API Coverage**: Complete — all known REST API endpoints integrated
- **Code Quality**: Zero clippy warnings, zero fmt diffs, all tests passing on all platforms

### What Was Done This Run
- Worked on branch `chore/type-unification-cleanup-2026-03-24-2` from `origin/main`
- **Consolidated self_update.rs map_err boilerplate** (~14 repeated closures eliminated):
  - Extracted `map_io_error(context, path)` helper replacing ~8 identical `map_err(|error| AppError::internal(format!(...)))` closures
  - Extracted `map_network_error(message)` helper replacing ~6 identical `map_err(|error| AppError::network(format!(...)))` closures
  - Applied across `write_bytes`, `extract_tar_gz_binary`, `extract_zip_binary`, `ensure_executable`, `download_bytes`, `download_text`, `fetch_release`
  - No behavioral changes — same error messages, same error types
- **Added 7 new REST transport edge-case tests** (34→41 tests in rest_transport_tests.rs):
  - `upload_url_with_content_type_and_multipart_upload` — content_type + multipart_upload query params
  - `entity_unfollow_rejects_user_without_id` — INVALID_INPUT error for missing user id
  - `activity_stream_passes_query_parameters` — entity_fields param forwarding
  - `event_log_entries_passes_query_parameters` — fields+sort param forwarding
  - `user_following_passes_query_parameters` — fields+page param forwarding
  - `entity_relationships_passes_query_parameters` — fields+page param forwarding
  - `note_threads_passes_query_parameters` — fields+page param forwarding
- **Expanded entity path tests** (4→16 parametrized rstest cases):
  - Single-word entities, already-plural, multi-uppercase, trailing digits, whitespace trimming, hyphenated, ending with 's'
- Net change: 347 insertions, 125 deletions across 7 files
- All CI checks passed (fmt, clippy, test on all 3 platforms, cross-platform builds)
- PR #92 squash-merged to main
- **Mirror type deliberate skip**: `OutputFormatArg`/`OutputFormat`, `AuthModeArg`/`AuthMode` not unified — adding `clap` to `fpt-core`/`fpt-domain` would leak CLI concerns into domain layer

### Previous Runs
- **Sixth pass**: Batch helpers, retry dedup, visibility tightening. PR #89 merged.
- **Fifth pass**: Filter deduplication and module cleanup. PR #88 merged.
- **Fourth pass**: Code cleanup round 4 — deduplicated errors and helpers. PR #86 merged.
- **Third pass**: Comprehensive test coverage — 27 REST transport tests + 10 CLI contract tests. PR #87 merged.
- **Second pass**: Code cleanup — fixed formatting diffs. PR #83 merged.
- **First pass**: Added 10 new tests for `inspect.command` and `entity.batch.summarize` error paths. PR #82 merged.

### Recent Features (Already in main)
- `entity.count` — with 4 tests
- `entity.batch.upsert` — with 12 checkpoint/resume tests
- `entity.batch.summarize` — with 1 happy path + 7 error path tests
- `schema.entity-create` — with transport delegation test
- `schema.entity-revive` — with transport delegation test

### Identified Cleanup Opportunities (Future — Higher Risk)
- **Batch executor pattern**: 8+ batch methods in `batch.rs` share stream/collect/sort boilerplate — could extract a generic executor
- ~~**Mirror types remaining**: `OutputFormatArg`/`OutputFormat`, `AuthModeArg`/`AuthMode` — deliberately kept as-is; adding `clap` to core/domain crates would leak CLI concerns~~
- ~~**self_update.rs map_err repetition**: consolidated in seventh pass via `map_io_error`/`map_network_error` helpers~~

### Known Remaining Gaps (Low Priority)
- `self.update` core logic (download/verify/replace) has no unit tests — intentionally skipped as it requires real binary replacement
- All ShotGrid REST API endpoints are covered with both domain-layer and HTTP-level mock tests
- All CLI commands have contract tests

### Architecture Notes
- Three-crate workspace: `fpt-cli` (binary), `fpt-core` (shared types), `fpt-domain` (business logic)
- `ShotgridTransport` trait with 39 async methods
- `RestTransport` implementation with token caching, exponential backoff retry, rate-limit handling
- `RecordingTransport` mock used in all domain tests
- Batch operations use `futures::stream::buffer_unordered` with configurable concurrency (default 8, max 32)
- Filter DSL parser converts human-readable expressions to ShotGrid JSON format
- `query_helpers.rs` is the canonical home for shared query/filter utilities (scalar_to_string, string_list_to_csv, string_list, normalize_filters, normalize_filter_operator, build_query_params)
- `batch.rs` uses `batch_result_ok`/`batch_result_err` helpers for all batch result JSON construction
- `transport.rs` uses `should_retry_rate_limit()` for unified 429 handling
