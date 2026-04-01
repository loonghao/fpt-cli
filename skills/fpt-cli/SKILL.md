---
name: fpt-cli
description: This skill should be used when OpenClaw needs to install, configure, inspect, or operate `fpt-cli` for Autodesk Flow Production Tracking / ShotGrid workflows, especially for auth setup, schema/entity reads, structured searches, and safe write previews.
---

## Purpose
Provide a stable, agent-optimized workflow for using `fpt-cli` from OpenClaw.

This skill follows the Agent DX principles from "You Need to Rewrite Your CLI for AI Agents":
- optimize for **predictability** over discoverability
- keep **context window usage low** (fetch only what you need)
- enforce **deep defense** against hallucinated inputs
- use **schema introspection at runtime** instead of relying on pre-trained knowledge

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
Never trigger browser-based OAuth flows in headless environments.

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

### 3. Schema introspection — do not guess
Do not rely on pre-trained knowledge to determine command signatures or entity/field names.
Pre-trained knowledge goes stale. Guessing causes syntax errors and hallucinates parameters.

**Pattern: fetch command list first, then fetch only the contracts you need.**

```bash
# Step 1: get all command names (cheap — no full payload)
fpt inspect list --output json

# Step 2: fetch contract for specific commands you will actually use
fpt inspect command entity.find --output json
fpt inspect command entity.batch.count --output json

# Step 3: verify entity and field names before composing queries
fpt schema entities --output json
fpt schema fields Shot --output json
```

Using `inspect list` before `inspect command` keeps context window usage low.
Using `capabilities` returns the full payload for all commands — use it only when you need an overview.

### 4. Choose the narrowest useful command
Prefer the smallest command that satisfies the task.

- Use `entity.get` when the entity id is known.
- Use `entity.find-one` when only one match is needed.
- Use `entity.find` when multiple matches or collection metadata are needed.
- Use `entity.batch.*` when repeating the same operation over many inputs.
- Use `entity.batch.count` when counting records across multiple entity types — do not call `entity.count` once per type.
- Use `schema.entities` and `schema.fields` before guessing entity or field names.

### 5. Context window discipline
Large API responses can consume significant context window and degrade reasoning.

**Always limit the fields you request:**

```bash
# Good: request only the fields you need
fpt entity get Shot 123 --fields code,sg_status_list --output json

# Good: use fields param in find input
fpt entity find Shot --input '{"fields": ["code", "sg_status_list"]}' --output json

# Good: batch count across multiple types in one call
fpt entity batch count --input '["Shot","Asset","Task"]' --output json
```

Do not request all fields when you only need a few.
Use `entity.batch.*` commands instead of looping single-entity calls.

### 6. Prefer structured JSON output
Default to `--output json` unless a human explicitly needs a different view.

This keeps OpenClaw orchestration stable and lowers prompt/token overhead.

### 7. Apply input hardening invariants
The CLI validates inputs against hallucination patterns. These checks cannot be bypassed:

- **Entity type names must not contain `?` or `#`** — do not embed query parameters in entity names.
  Wrong: `entity get "Shot?fields=code" 123`
  Right: `entity get Shot 123 --fields code`
- **Entity type names must not contain control characters** (below ASCII 0x20).
- **Do not pre-encode URLs** — the CLI handles percent-encoding automatically. Sending `%2e%2e` instead of `..` will cause double-encoding failures.
- **Resource IDs are numeric** — never embed filter expressions or query parameters in an ID argument.

The CLI operates in a zero-trust model: all inputs are validated as if they came from an untrusted source.

### 8. Prefer native search features for complex queries
For non-trivial filters:
- prefer structured `search` JSON when building native `_search` payloads
- use `additional_filter_presets` for "latest"-style workflows
- use `--filter-dsl` for concise human-authored boolean logic

Read `references/query-patterns.md` for examples.

### 9. Apply write safety rules
For writes:
- run `--dry-run` first when supported — treat dry-run output as the request-plan contract
- review the dry-run plan to confirm there are no hallucinated parameters before executing
- require explicit confirmation before real deletes (`--yes`)

```bash
# Always preview before mutating
fpt entity create Version --input @payload.json --dry-run --output json
# Then execute only after reviewing the plan
fpt entity create Version --input @payload.json --output json
```

### 10. Debug in a contract-first order
When something fails:
1. validate auth with `auth test`
2. inspect the command contract with `inspect command <name>`
3. confirm entity and field names via schema commands
4. reduce the command to the smallest JSON-shaped reproduction
5. only then expand to batch or write workflows

## References
- `references/install-and-auth.md`
- `references/query-patterns.md`
