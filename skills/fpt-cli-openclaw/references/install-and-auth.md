## Install and update `fpt-cli`

### Released binary installation
Install the latest release over HTTPS.

#### macOS / Linux
```bash
curl -fsSL https://raw.githubusercontent.com/loonghao/fpt-cli/main/scripts/install.sh | sh
```

#### Windows PowerShell
```powershell
powershell -ExecutionPolicy Bypass -c "irm https://raw.githubusercontent.com/loonghao/fpt-cli/main/scripts/install.ps1 | iex"
```

### In-place update
Use the released binary's self-update command when `fpt` is already installed.

```bash
fpt self-update --check --output pretty-json
fpt self-update
```

### Repository-local execution
When operating from the source checkout, prefer the repository-managed environment.

```bash
vx cargo run -p fpt-cli -- capabilities --output json
vx just test
```

## Authentication quick reference

### Preferred environment variables
- `FPT_SITE`
- `FPT_AUTH_MODE`
- `FPT_SCRIPT_NAME`
- `FPT_SCRIPT_KEY`
- `FPT_USERNAME`
- `FPT_PASSWORD`
- `FPT_AUTH_TOKEN`
- `FPT_SESSION_TOKEN`
- `FPT_API_VERSION`

### Auth modes
- `script`
- `user_password`
- `session_token`

### Auth validation
Validate credentials before running entity or schema commands.

```bash
fpt auth test --output json
```

### Example: script auth
```bash
export FPT_SITE="https://example.shotgrid.autodesk.com"
export FPT_AUTH_MODE="script"
export FPT_SCRIPT_NAME="openclaw"
export FPT_SCRIPT_KEY="your-script-key"
fpt auth test --output json
```

### Example: user-password auth
```bash
export FPT_SITE="https://example.shotgrid.autodesk.com"
export FPT_AUTH_MODE="user_password"
export FPT_USERNAME="user@example.com"
export FPT_PASSWORD="your-password"
fpt auth test --output json
```

### Windows PowerShell example
```powershell
$env:FPT_SITE = "https://openclaw.shotgrid.autodesk.com"
$env:FPT_AUTH_MODE = "user_password"
$env:FPT_USERNAME = "user@example.com"
$env:FPT_PASSWORD = "your-password"
fpt auth test --output pretty-json
```

## Safety notes
- Prefer environment variables over raw credential arguments.
- Treat `FPT_*` as the primary namespace.
- Use `SG_*` only as fallback compatibility inputs.
- Add `FPT_AUTH_TOKEN` when the ShotGrid site requires 2FA.
