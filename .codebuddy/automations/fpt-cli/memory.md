# fpt-cli Automation Memory

## Last Run: 2026-03-25 (eighth pass — expose remaining unregistered CLI commands)

### Project State
- **Total CommandSpecs**: 58 (all registered in capability.rs)
- **Total Tests**: 230+ across test files in 3 crates (all passed)
- **ShotGrid API Coverage**: Complete — all known REST API endpoints integrated AND exposed as CLI commands
- **Code Quality**: Zero clippy warnings, zero fmt diffs, all tests passing on all platforms

### What Was Done This Run
- Worked on branch `feat/new-cli-commands-2026-03-25` from `origin/main`
- **Identified 3 API endpoints with full Transport + App implementations but no CLI commands**:
  1. `current_user` — GET /entity/{collection}/current
  2. `note_reply_read` — GET /entity/notes/{note_id}/thread_contents/{reply_id}
  3. `filmstrip_thumbnail` — GET /entity/{type}/{id}/filmstrip_image
- **Added 3 new CommandSpecs**: `user.current`, `note.reply-read`, `filmstrip.url`
- **Added CLI commands**: `UserCommands`, `FilmstripCommands`, `NoteCommands::ReplyRead`
- **Wired commands in runner.rs** with proper input handling
- **Added 7 app command tests**: current_user delegation, api type, invalid type rejection; note_reply_read delegation; filmstrip delegation; capabilities spec inclusion
- **Added 6 REST transport tests**: current_user human/api paths + query params; note_reply_read path + query params; filmstrip_thumbnail path
- **Regenerated workspace-hack** for lalrpop-util version alignment
- Net change: +774/-14 across 19 files (feat commit) + workspace-hack fix
- All CI checks passed (fmt, clippy, hakari, test on all 3 platforms, cross-platform builds)
- PR #94 squash-merged to main

### Previous Runs
- **Seventh pass**: self_update map_err consolidation + edge-case tests. PR #92 merged.
- **Sixth pass**: Batch helpers, retry dedup, visibility tightening. PR #89 merged.
- **Fifth pass**: Filter deduplication and module cleanup. PR #88 merged.
- **Fourth pass**: Code cleanup round 4 — deduplicated errors and helpers. PR #86 merged.
- **Third pass**: Comprehensive test coverage — 27 REST transport tests + 10 CLI contract tests. PR #87 merged.
- **Second pass**: Code cleanup — fixed formatting diffs. PR #83 merged.
- **First pass**: Added 10 new tests for `inspect.command` and `entity.batch.summarize` error paths. PR #82 merged.

### Known Remaining Gaps (Low Priority)
- `self.update` core logic (download/verify/replace) has no unit tests — intentionally skipped as it requires real binary replacement
- All ShotGrid REST API endpoints are now both implemented AND exposed as CLI commands
- **Batch executor pattern**: 8+ batch methods in `batch.rs` share stream/collect/sort boilerplate — could extract a generic executor

### Architecture Notes
- Three-crate workspace: `fpt-cli` (binary), `fpt-core` (shared types), `fpt-domain` (business logic)
- `ShotgridTransport` trait with 42 async methods — all fully implemented in `RestTransport`
- 58 CommandSpecs registered in capability.rs — complete CLI surface
- `RecordingTransport` mock used in all domain tests
- Batch operations use `futures::stream::buffer_unordered` with configurable concurrency (default 8, max 32)
