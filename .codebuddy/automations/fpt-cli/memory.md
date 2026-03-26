# fpt-cli Automation Memory

## Last Run: 2026-03-26 (eleventh pass — entity relationship delete)

### Project State
- **Total CommandSpecs**: 68 (all registered in capability.rs)
- **Total Tests**: 278+ across test files in 3 crates (all passed)
- **ShotGrid API Coverage**: Expanded — completed relationship CRUD surface
- **Code Quality**: Zero clippy warnings, zero fmt diffs, all tests passing on all platforms

### What Was Done This Run
- Created branch `feat/entity-relationship-delete` from `origin/main`
- **Identified `entity_relationship_delete` as the last feasible API endpoint to integrate**:
  - DELETE /entity/{type}/{id}/relationships/{field} — removes links from a multi-entity relationship field
- **Added 1 new transport trait method** with REST implementation (DELETE with JSON body)
- **Added 1 new app layer method** with input validation (requires `data` field, same as create/update)
- **Added 1 new CLI command** (EntityCommands::RelationshipDelete)
- **Added 1 new CommandSpec**: `entity.relationship-delete`
- **Wired command in runner.rs**
- **Added 4 new tests**: 3 app command tests (delegation + 2 validation) + 1 REST transport test
- Updated all 6 mock transport impls across 4 test files
- Updated README.md and README_zh.md with new commands and test coverage
- Net change: +248/-23 across 14 files
- All CI checks passed (9/9: fmt, clippy, hakari, test on all 3 platforms, cross-platform builds, code coverage)
- PR #100 squash-merged to main

### Previous Runs
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
- **Batch executor pattern**: 8+ batch methods in `batch.rs` share stream/collect/sort boilerplate — could extract a generic executor
- **Remaining ShotGrid REST API endpoints not yet integrated**:
  - Server-side `_batch` API (POST /api/v1/entity/_batch) — current batch is client-side orchestration
  - Thumbnail/image upload (PUT /entity/{type}/{id}/image) — only GET url exists
  - Actual file upload via S3 presigned URL flow — only upload_url is implemented
  - _(entity relationship delete is now implemented — relationship CRUD is complete)_

### Architecture Notes
- Three-crate workspace: `fpt-cli` (binary), `fpt-core` (shared types), `fpt-domain` (business logic)
- `ShotgridTransport` trait with 49 async methods — all fully implemented in `RestTransport`
- 68 CommandSpecs registered in capability.rs — complete CLI surface
- `RecordingTransport` mock used in all domain tests
- Batch operations use `futures::stream::buffer_unordered` with configurable concurrency (default 8, max 32)
