# Changelog

## [0.2.12](https://github.com/loonghao/fpt-cli/compare/v0.2.11...v0.2.12) (2026-03-23)


### Miscellaneous Chores

* **deps:** update rust crate zip to v8.3.1 ([#62](https://github.com/loonghao/fpt-cli/issues/62)) ([f4294c1](https://github.com/loonghao/fpt-cli/commit/f4294c1b569706f068934f195f547075fd5836be))

## [0.2.11](https://github.com/loonghao/fpt-cli/compare/v0.2.10...v0.2.11) (2026-03-21)


### Miscellaneous Chores

* **deps:** bump rustls-webpki from 0.103.9 to 0.103.10 ([07880cb](https://github.com/loonghao/fpt-cli/commit/07880cb54b49fa0d89f7fdb09107ff75d269fc28))
* **deps:** bump tar from 0.4.44 to 0.4.45 ([e76208b](https://github.com/loonghao/fpt-cli/commit/e76208b020ccfe65484f9591d86d78149991f958))

## [0.2.10](https://github.com/loonghao/fpt-cli/compare/v0.2.9...v0.2.10) (2026-03-20)


### Miscellaneous Chores

* **deps:** update rust crate zip to v8.3.0 ([23d87a6](https://github.com/loonghao/fpt-cli/commit/23d87a623356926273e848822b8290b345a5ce7c))

## [0.2.9](https://github.com/loonghao/fpt-cli/compare/v0.2.8...v0.2.9) (2026-03-19)


### Features

* add resumable bulk upsert with checkpoint/resume support ([#43](https://github.com/loonghao/fpt-cli/issues/43)) ([1773651](https://github.com/loonghao/fpt-cli/commit/17736517a3ac8c6f6f1a83cab0ab36becf573923))
* improve entity-link filter ergonomics in entity find ([78f645c](https://github.com/loonghao/fpt-cli/commit/78f645cad687bf010e44209e335115381f8c7f36)), closes [#42](https://github.com/loonghao/fpt-cli/issues/42)


### Bug Fixes

* **#41-45:** exit codes, search normalization, entity-link DSL, batch upsert, retry/stats ([0f260d5](https://github.com/loonghao/fpt-cli/commit/0f260d50d55087b6ef5d1f7e910556a1bc53ff5e))
* isolate auth_config_tests env and apply cargo fmt ([1d4de57](https://github.com/loonghao/fpt-cli/commit/1d4de57760f942b8711f74e5610159045f316792))
* normalize structured search payloads consistently across query commands ([16f02b8](https://github.com/loonghao/fpt-cli/commit/16f02b8bd79b838cfa776ecc5d87ce36430ea700)), closes [#44](https://github.com/loonghao/fpt-cli/issues/44)
* resolve config clear panic, capabilities version mismatch, and harden exit codes ([#48](https://github.com/loonghao/fpt-cli/issues/48), [#49](https://github.com/loonghao/fpt-cli/issues/49), [#45](https://github.com/loonghao/fpt-cli/issues/45)) ([26da0f7](https://github.com/loonghao/fpt-cli/commit/26da0f78486f6b4655d2544b04206f0749aa7e0a))
* **skills:** complete credential declarations in SKILL.md and install-and-auth.md ([6b4dbe5](https://github.com/loonghao/fpt-cli/commit/6b4dbe55c68bf8606bce038d200725e4bd7917a6))


### Miscellaneous Chores

* replace all Chinese strings/comments with English ([e938351](https://github.com/loonghao/fpt-cli/commit/e938351401d7ef635f125a4fb5685137d4954b5a))

## [0.2.8](https://github.com/loonghao/fpt-cli/compare/v0.2.7...v0.2.8) (2026-03-17)


### Features

* enrich all AppError callsites with structured details fields ([9065ff5](https://github.com/loonghao/fpt-cli/commit/9065ff5faae47f4d1d54cc2fd3e46d11a40dd8bd))


### Bug Fixes

* **test:** isolate infers_user_password_auth test from persisted config and env vars ([f04be7b](https://github.com/loonghao/fpt-cli/commit/f04be7b9478d3e6e4faf9126c86c7e7210acf848))
* use candidates.contains() instead of iter().any() per clippy ([e543059](https://github.com/loonghao/fpt-cli/commit/e5430599dd75579b5e6b325212e86e811719a8de))

## [0.2.7](https://github.com/loonghao/fpt-cli/compare/v0.2.6...v0.2.7) (2026-03-17)


### Features

* add missing ShotGrid API endpoints, public API facade, tests & CI coverage ([bb4cde5](https://github.com/loonghao/fpt-cli/commit/bb4cde50879f333d5f8d586b7f8bce8387bf7bcf))


### Miscellaneous Chores

* enable chore type to bump version in release-please config ([dcf27b2](https://github.com/loonghao/fpt-cli/commit/dcf27b2b58f4d08c58aec2038366856733a948b8))
* remove --locked flags from cargo commands ([a6b5419](https://github.com/loonghao/fpt-cli/commit/a6b5419b78086f560ff349a4761e0694b9314e6f))
* update dependencies (cargo update) ([12909a2](https://github.com/loonghao/fpt-cli/commit/12909a2072078a1f1d5bb5bcb711fef5ea3cfd57))

## [0.2.6](https://github.com/loonghao/fpt-cli/compare/v0.2.5...v0.2.6) (2026-03-16)


### Bug Fixes

* **ci:** sync lockfile with workspace version ([a425815](https://github.com/loonghao/fpt-cli/commit/a4258156ba8bd7b981fe33e590c4f5e15a9808e4))

## [0.2.5](https://github.com/loonghao/fpt-cli/compare/v0.2.4...v0.2.5) (2026-03-16)


### Bug Fixes

* **ci:** catch stale Cargo.lock before release ([3e96719](https://github.com/loonghao/fpt-cli/commit/3e967198655ee0d74677a67751d177adfe08558a))

## [0.2.4](https://github.com/loonghao/fpt-cli/compare/v0.2.3...v0.2.4) (2026-03-16)


### Bug Fixes

* add --locked to CI commands and update Cargo.lock ([024b4bc](https://github.com/loonghao/fpt-cli/commit/024b4bcef687f2e5c5fb477e2575db4400c8a926))

## [0.2.3](https://github.com/loonghao/fpt-cli/compare/v0.2.2...v0.2.3) (2026-03-15)


### Bug Fixes

* **ci:** add build test job and update Cargo.lock ([9125559](https://github.com/loonghao/fpt-cli/commit/9125559426a6704a1af05b147fc52d1affadf652))

## [0.2.2](https://github.com/loonghao/fpt-cli/compare/v0.2.1...v0.2.2) (2026-03-15)


### Bug Fixes

* **ci:** enable cargo-workspace plugin for release-please ([4e850a9](https://github.com/loonghao/fpt-cli/commit/4e850a9c940a87bb623e935dff67be4a813951c0))

## [0.2.1](https://github.com/loonghao/fpt-cli/compare/v0.2.0...v0.2.1) (2026-03-15)


### Bug Fixes

* **ci:** publish release and skills from release-please ([9ae31aa](https://github.com/loonghao/fpt-cli/commit/9ae31aaf857621c2ecd90b83ddbe543bf35681ff))
* **ci:** publish release and skills from release-please ([c39377f](https://github.com/loonghao/fpt-cli/commit/c39377f078e3a6b9dfda10190f2b7d36d8ab3c25))
* unify release artifact publishing ([18b0476](https://github.com/loonghao/fpt-cli/commit/18b0476020ff0ba1662012bc0fc8ba773631e179))


### Documentation

* align skill install paths with installers ([7034e7d](https://github.com/loonghao/fpt-cli/commit/7034e7d8b941f1fb30722203d5974a04bcc245ce))
* harden OpenClaw skill install guidance ([d35bdd9](https://github.com/loonghao/fpt-cli/commit/d35bdd9077f50fa1f48fdc3fab27e0d6afce8bea))

## [0.2.0](https://github.com/loonghao/fpt-cli/compare/v0.1.0...v0.2.0) (2026-03-15)


### Features

* expand shotgrid cli capabilities and release workflows ([f4d50d7](https://github.com/loonghao/fpt-cli/commit/f4d50d79d87d69e2a3504a6c067936b051862c3c))
* improve shotgrid cli docs and coverage ([3ed1f26](https://github.com/loonghao/fpt-cli/commit/3ed1f261afcd4d0fa8b7c03f5b4a6ff54c123f9c))


### Bug Fixes

* stabilize ci checks for pull request ([0e01a1c](https://github.com/loonghao/fpt-cli/commit/0e01a1cd4b9528a349839584eb85e24541517f6d))
