## Install and update `fpt-cli`

### Released binary installation
Prefer downloading a release archive, optionally verifying `fpt-checksums.txt`, and then extracting the binary locally.
Avoid remote-script piping and direct remote-expression execution patterns in automated environments.


Release asset names:
- Linux: `fpt-x86_64-unknown-linux-gnu.tar.gz`
- Windows: `fpt-x86_64-pc-windows-msvc.zip`
- macOS (Intel): `fpt-x86_64-apple-darwin.tar.gz`
- macOS (Apple Silicon): `fpt-aarch64-apple-darwin.tar.gz`

#### macOS / Linux example
```bash
export FPT_VERSION="v0.1.0"
curl -fLO "https://github.com/loonghao/fpt-cli/releases/download/${FPT_VERSION}/fpt-x86_64-unknown-linux-gnu.tar.gz"
curl -fLO "https://github.com/loonghao/fpt-cli/releases/download/${FPT_VERSION}/fpt-checksums.txt"
sha256sum -c --ignore-missing fpt-checksums.txt
tar -xzf fpt-x86_64-unknown-linux-gnu.tar.gz
./fpt capabilities --output json
```

#### Windows PowerShell example
```powershell
$FptVersion = "v0.1.0"
$Archive = "fpt-x86_64-pc-windows-msvc.zip"
Invoke-WebRequest -Uri "https://github.com/loonghao/fpt-cli/releases/download/$FptVersion/$Archive" -OutFile $Archive
Expand-Archive -Path $Archive -DestinationPath ".\fpt-bin" -Force
.\fpt-bin\fpt.exe capabilities --output json
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
- Prefer release archives over remote-script execution in agent workflows.
- Prefer environment variables over raw credential arguments.
- Treat `FPT_*` as the primary namespace.
- Use `SG_*` only as fallback compatibility inputs.
- Add `FPT_AUTH_TOKEN` when the ShotGrid site requires 2FA.
