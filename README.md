## `fpt-cli`

[English](README.md) | [简体中文](README_zh.md)

A Rust CLI for Autodesk Flow Production Tracking (**ShotGrid / FPT**) designed for **OpenClaw, AI agents, and automation-first workflows**.

`fpt-cli` is intentionally **CLI-first**. It aims to provide a stable command contract for ShotGrid/FPT operations, make agent orchestration easier, and reduce unnecessary **MCP token consumption** by moving repeatable interactions into explicit CLI commands with structured JSON output.

This repository is also a practical validation and exploration of the ideas discussed in [MCP Is Dead, Long Live the CLI](https://ejholmes.github.io/2026/02/28/mcp-is-dead-long-live-the-cli.html).

### Current status

This repository is the first implementation stage and already includes:

- **Rust workspace** split into focused crates
- **CLI-first** command tree
- **Structured JSON output** for automation
- **Command capability / inspect** discovery APIs
- **REST transport MVP** for auth, schema, and entity CRUD
- **`entity batch`** client-side orchestration for batch CRUD operations
- **Controlled batch concurrency** with stable ordered results
- **Dry-run** planning for write operations
- **Three auth modes**: script, user password, and session token
- **In-process access token reuse** to reduce repeated auth overhead

### Development environment

All environments in this project should be managed through **`vx`**, and command collections should be exposed through the repository **`justfile`**.

```bash
vx setup
vx just test
vx just capabilities
```

### Authentication environment variables

The CLI prefers the `FPT_*` prefix and also supports `SG_*` as a fallback when `FPT_*` is not present.

- `FPT_SITE` / `SG_SITE`
- `FPT_AUTH_MODE` / `SG_AUTH_MODE`: `script` / `user_password` / `session_token`
- `FPT_SCRIPT_NAME` / `SG_SCRIPT_NAME`
- `FPT_SCRIPT_KEY` / `SG_SCRIPT_KEY`
- `FPT_USERNAME` / `SG_USERNAME`
- `FPT_PASSWORD` / `SG_PASSWORD`
- `FPT_AUTH_TOKEN` / `SG_AUTH_TOKEN`: optional when the site uses 2FA
- `FPT_SESSION_TOKEN` / `SG_SESSION_TOKEN`
- `FPT_API_VERSION` / `SG_API_VERSION` (optional, default `v1.1`)

### Authentication modes

- **script**: uses `script_name + script_key` with the `client_credentials` flow
- **user_password**: uses `username + password` with the `password` grant
- **session_token**: uses an existing `session_token` with the `session_token` grant

If `--auth-mode` is not provided explicitly, the CLI infers it from the available inputs:

- **Prefer user password**: if any of `username`, `password`, or `auth_token` is present
- **Then session token**: if `session_token` is present
- **Otherwise script**: fall back to `script_name + script_key`

### Implemented commands

```bash
fpt capabilities --output json
fpt inspect command entity.update --output json
fpt auth test --site https://example.shotgrid.autodesk.com --auth-mode script --script-name bot --script-key xxx
fpt auth test --site https://example.shotgrid.autodesk.com --auth-mode user-password --username user@example.com --password secret
fpt auth test --site https://example.shotgrid.autodesk.com --auth-mode session-token --session-token xxx
fpt schema entities --site ... --auth-mode script --script-name ... --script-key ...
fpt schema fields Shot --site ... --auth-mode user-password --username ... --password ...
fpt entity get Shot 123 --site ... --auth-mode session-token --session-token ...
fpt entity find Asset --input @query.json --site ... --auth-mode script --script-name ... --script-key ...
fpt entity find Asset --filter-dsl "sg_status_list == 'ip' and (code ~ 'bunny' or id > 100)" --site ... --auth-mode script --script-name ... --script-key ...
fpt entity create Version --input @payload.json --dry-run
fpt entity update Task 42 --input @patch.json --dry-run
fpt entity delete Playlist 99 --dry-run

fpt entity batch get Shot --input '{"ids":[101,102],"fields":["code","sg_status_list"]}' --output json
fpt entity batch find Asset --input @batch_queries.json --output json
fpt entity batch create Version --input @batch_payloads.json --dry-run --output json
fpt entity batch update Task --input @batch_updates.json --dry-run --output json
fpt entity batch delete Playlist --input '{"ids":[99,100]}' --dry-run --output json
```

### Batch CRUD

`entity batch` provides batch get / find / create / update / delete workflows.
The current implementation is **client-side orchestration over existing REST CRUD endpoints** and returns a unified `results` array where each item carries its own `ok` state plus `response` or `error`.

Input conventions:

- **`entity batch get`**: `[1,2,3]` or `{"ids":[1,2,3],"fields":["code"]}`
- **`entity batch find`**: `[{...query1...},{...query2...}]` or `{"requests":[...]}`
- **`entity batch create`**: `[{...body1...},{...body2...}]` or `{"items":[...]}`
- **`entity batch update`**: `[{"id":42,"body":{...}}, {"id":43,"body":{...}}]` or `{"items":[...]}`
- **`entity batch delete`**: `[42,43]` or `{"ids":[42,43]}`

Notes:

- **Batch create / update / delete support `--dry-run`**
- **Batch delete still requires explicit `--yes` for real execution**
- Batch sub-requests in the same CLI process **reuse the access token**
- Batch sub-requests run with **controlled concurrency**, defaulting to `8`
- Use **`FPT_BATCH_CONCURRENCY`** to tune concurrency; set it to `1` to force serial execution

### Complex filter DSL

`entity find` supports complex filtering through `--filter-dsl` (or `filter_dsl` inside the `--input` JSON).
When DSL is used, the CLI automatically switches to ShotGrid REST `_search`.

Supported DSL features:

- Field paths: `field` / `linked.field`
- Logical operators: `and` / `or` / `(...)`
- Comparison operators: `==`, `!=`, `>`, `>=`, `<`, `<=`, `~` (mapped to `contains`)
- Keyword operators: such as `in`, `not in`, `starts_with` (forwarded as-is to ShotGrid)
- Value types: string, number, boolean, `null`, array

Example:

```bash
fpt entity find Asset --filter-dsl "sg_status_list == 'ip' and (code ~ 'bunny' or id > 100)"
```

> `filters` and `filter_dsl` cannot be used together.

### Test coverage

Current coverage is split into two layers:

- **App orchestration tests**: `auth.test`, `schema.entities`, `schema.fields`, `entity.get/find/create/update/delete`, `entity.batch.*`
- **REST transport tests**: OAuth token acquisition, schema/entity route mapping, `_search` switching, write-method mapping, error classification, token reuse

Recommended command during development:

```bash
vx just test
```

### OpenClaw site debugging example

Prefer environment variables instead of putting secrets directly into shell history.

```powershell
$env:FPT_SITE = "https://openclaw.shotgrid.autodesk.com"
$env:FPT_AUTH_MODE = "user_password"
$env:FPT_USERNAME = "user@example.com"
$env:FPT_PASSWORD = "your-password"
vx cargo run -p fpt-cli -- auth test --output pretty-json
```

When debugging `scripts/local_count_projects.ps1`, you can also place a `.env` file in the repository root. The script loads it automatically and does not overwrite variables already set in the current shell.

```dotenv
FPT_SITE=https://openclaw.shotgrid.autodesk.com
FPT_AUTH_MODE=script
FPT_SCRIPT_NAME=openclaw
FPT_SCRIPT_KEY="your-script-key"
```

```powershell
pwsh -File .\scripts\local_count_projects.ps1 -AuthMode script -VerbosePage
```

> Pass secrets through environment variables or `.env`, not raw command-line arguments.
> On Windows, characters such as `^`, `&`, `!`, and `%` may be escaped or swallowed somewhere in the shell and process launch chain.
> The script now passes only `--auth-mode`, while credentials are loaded from the environment by the CLI.
> Its direct connectivity precheck uses `Invoke-WebRequest -SkipHttpErrorCheck`, so even a `400` response from ShotGrid can still surface the response body for diagnosis.

If the site uses two-factor authentication, you can also set:

```powershell
$env:FPT_AUTH_TOKEN = "123456"
vx cargo run -p fpt-cli -- auth test --output pretty-json
```

### Design principles

- **The CLI contract stays independent from transport implementation details**
- **JSON is the default output for agent-friendly integration**
- **Write operations support `--dry-run`**
- **Future transports beyond REST can be added without breaking the OpenClaw-facing contract**
