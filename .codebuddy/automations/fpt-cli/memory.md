# fpt-cli Automation Memory

## Last Run: 2026-03-26 (ninth pass — add preferences update + note reply update/delete)

### Project State
- **Total CommandSpecs**: 64 (all registered in capability.rs)
- **Total Tests**: 250+ across test files in 3 crates (all passed)
- **ShotGrid API Coverage**: Expanded — added 3 more REST API endpoints
- **Code Quality**: Zero clippy warnings, zero fmt diffs, all tests passing on all platforms

### What Was Done This Run
- Worked on branch `feat/new-api-endpoints-2026-03-26` from `origin/main`
- **Identified 3 ShotGrid REST API endpoints not yet integrated**:
  1. `preferences_update` — PUT /preferences
  2. `note_reply_update` — PUT /entity/notes/{note_id}/thread_contents/{reply_id}
  3. `note_reply_delete` — DELETE /entity/notes/{note_id}/thread_contents/{reply_id}
- **Added 3 new transport trait methods** with REST implementations
- **Added 3 new app layer methods** with input validation (note reply update reuses existing validation)
- **Added 3 new CLI commands** (PreferencesCommands::Update, NoteCommands::ReplyUpdate, NoteCommands::ReplyDelete)
- **Added 3 new CommandSpecs**: `preferences.update`, `note.reply-update`, `note.reply-delete`
- **Wired all commands in runner.rs**
- **Added 12 new tests**: 9 app command tests (delegation, validation, capabilities) + 3 REST transport tests
- Updated README.md and README_zh.md with new commands and test coverage
- Net change: +551/-46 across 15 files
- All CI checks passed (9/9: fmt, clippy, hakari, test on all 3 platforms, cross-platform builds, code coverage)
- PR #98 squash-merged to main

### Previous Runs
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
  - Entity relationship write (POST/PUT relationships) — only GET exists
  - Thumbnail/image upload (PUT /entity/{type}/{id}/image) — only GET url exists
  - Actual file upload via S3 presigned URL flow — only upload_url is implemented

### Architecture Notes
- Three-crate workspace: `fpt-cli` (binary), `fpt-core` (shared types), `fpt-domain` (business logic)
- `ShotgridTransport` trait with 45 async methods — all fully implemented in `RestTransport`
- 64 CommandSpecs registered in capability.rs — complete CLI surface
- `RecordingTransport` mock used in all domain tests
- Batch operations use `futures::stream::buffer_unordered` with configurable concurrency (default 8, max 32)
