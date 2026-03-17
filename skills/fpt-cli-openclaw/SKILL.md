---
name: fpt-cli-openclaw
description: This skill should be used when OpenClaw needs to install, configure, inspect, or operate `fpt-cli` for Autodesk Flow Production Tracking / ShotGrid workflows, especially for auth setup, schema/entity reads, structured searches, and safe write previews.
---

## Purpose
Provide a stable workflow for using `fpt-cli` from OpenClaw.

Keep the agent behavior aligned with the repository contract:
- prefer explicit CLI commands over ad-hoc API calls
- prefer JSON output for machine consumption
- prefer capability discovery before composing new command invocations
- prefer safe write previews before real mutations

## When to use
Use this skill when any of the following is needed:
- install or update `fpt-cli`
- configure ShotGrid / FPT authentication for OpenClaw
- inspect which commands the CLI already exposes
- query schema or entities through the CLI
- run complex searches with `filter_dsl`, structured `search`, or `additional_filter_presets`
- perform write operations with `--dry-run` first

## Workflow

### 1. Choose the execution mode
Determine whether the task should use a released binary or a source checkout.

- For released binary installation or update, read `references/install-and-auth.md` and prefer release archives plus checksum verification over pipe-to-shell installers.
- For repository-local development, prefer `vx cargo run -p fpt-cli -- ...` and `vx just ...`.


### 2. Prefer environment-based authentication
Load credentials through environment variables instead of putting secrets directly on the command line.

Preferred variables:

| Variable | Required | Auth modes | Description |
|---|---|---|---|
| `FPT_SITE` | **required** | all | Full URL of the ShotGrid / FPT site, e.g. `https://example.shotgrid.autodesk.com` |
| `FPT_AUTH_MODE` | **required** | all | Auth strategy: `script`, `user_password`, or `session_token` |
| `FPT_SCRIPT_NAME` | **required** | `script` | Name of the API script credential registered in ShotGrid |
| `FPT_SCRIPT_KEY` | **required** | `script` | Secret key for the script credential; quote the value when it contains special characters |
| `FPT_USERNAME` | **required** | `user_password` | ShotGrid user login (usually an email address) |
| `FPT_PASSWORD` | **required** | `user_password` | Password for the ShotGrid user account |
| `FPT_AUTH_TOKEN` | optional | `user_password` | One-time 2FA token; only needed when the site enforces two-factor authentication |
| `FPT_SESSION_TOKEN` | **required** | `session_token` | A pre-obtained ShotGrid session token; use when script or password credentials are unavailable |
| `FPT_API_VERSION` | optional | all | Override the ShotGrid REST API version, e.g. `v1.1`; defaults to the CLI built-in value when omitted |

Allow `SG_*` variables only as compatibility fallback when `FPT_*` is not available.

### 3. Discover the contract before composing commands
Inspect the CLI surface before building new automation.

Use:
- `fpt capabilities --output json`
- `fpt inspect command <command-name> --output json`

Prefer dotted command names in inspection calls, for example:
- `entity.find`
- `entity.find-one`
- `entity.update`

### 4. Choose the narrowest useful command
Prefer the smallest command that satisfies the task.

- Use `entity.get` when the entity id is known.
- Use `entity.find-one` when only one match is needed.
- Use `entity.find` when multiple matches or collection metadata are needed.
- Use `entity.batch.*` when repeating the same operation over many inputs.
- Use `schema.entities` and `schema.fields` before guessing entity or field names.

### 5. Prefer structured JSON output
Default to `--output json` unless a human explicitly needs a different view.

This keeps OpenClaw orchestration stable and lowers prompt/token overhead.

### 6. Prefer native search features for complex queries
For non-trivial filters:
- prefer structured `search` JSON when building native `_search` payloads
- use `additional_filter_presets` for “latest”-style workflows
- use `--filter-dsl` for concise human-authored boolean logic

Read `references/query-patterns.md` for examples.

### 7. Apply write safety rules
For writes:
- run `--dry-run` first when supported
- treat dry-run output as the request-plan contract
- require explicit confirmation before real deletes (`--yes`)

### 8. Debug in a contract-first order
When something fails:
1. validate auth with `auth test`
2. inspect the command contract
3. confirm entity and field names via schema commands
4. reduce the command to the smallest JSON-shaped reproduction
5. only then expand to batch or write workflows

## References
- `references/install-and-auth.md`
- `references/query-patterns.md`
