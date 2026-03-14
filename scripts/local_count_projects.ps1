param(
    [int]$PageSize = 100,
    [switch]$VerbosePage,
    [ValidateSet("auto", "script", "user_password", "session_token")]
    [string]$AuthMode = "auto"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Resolve-DotEnvPath {
    $scriptRoot = Split-Path -Parent $PSCommandPath
    $candidates = @(
        (Join-Path (Get-Location) ".env"),
        (Join-Path $scriptRoot ".env"),
        (Join-Path (Split-Path -Parent $scriptRoot) ".env")
    )

    foreach ($candidate in $candidates) {
        if (Test-Path -LiteralPath $candidate) {
            return (Resolve-Path -LiteralPath $candidate).Path
        }
    }

    return $null
}

function Import-DotEnv {
    param([string]$Path)


    if ([string]::IsNullOrWhiteSpace($Path) -or -not (Test-Path -LiteralPath $Path)) {
        return
    }

    $loaded = 0
    $skipped = 0

    foreach ($rawLine in Get-Content -LiteralPath $Path -Encoding UTF8) {
        $line = $rawLine.Trim()
        if ([string]::IsNullOrWhiteSpace($line) -or $line.StartsWith("#")) {
            continue
        }

        if ($line.StartsWith("export ")) {
            $line = $line.Substring(7).Trim()
        }

        $idx = $line.IndexOf("=")
        if ($idx -le 0) {
            continue
        }

        $name = $line.Substring(0, $idx).Trim()
        $value = $line.Substring($idx + 1).Trim()
        if ([string]::IsNullOrWhiteSpace($name)) {
            continue
        }

        if (($value.StartsWith('"') -and $value.EndsWith('"')) -or ($value.StartsWith("'") -and $value.EndsWith("'"))) {
            $value = $value.Substring(1, $value.Length - 2)
        }

        $existing = [Environment]::GetEnvironmentVariable($name)
        if ([string]::IsNullOrWhiteSpace($existing)) {
            [Environment]::SetEnvironmentVariable($name, $value, "Process")
            $loaded += 1
        } else {
            $skipped += 1
        }
    }

    Write-Host ("[提示] 已从 .env 加载 {0} 个变量（跳过已存在 {1} 个）: {2}" -f $loaded, $skipped, $Path) -ForegroundColor DarkCyan
}

$dotEnvPath = Resolve-DotEnvPath
Import-DotEnv -Path $dotEnvPath


function Get-EnvCompat {

    param(
        [string]$Primary,
        [string]$Fallback
    )

    $value = [Environment]::GetEnvironmentVariable($Primary)
    if (-not [string]::IsNullOrWhiteSpace($value)) {
        return $value.Trim()
    }

    $fallbackValue = [Environment]::GetEnvironmentVariable($Fallback)
    if (-not [string]::IsNullOrWhiteSpace($fallbackValue)) {
        return $fallbackValue.Trim()
    }

    return $null
}

function Get-SecretFingerprint {
    param([string]$Value)

    if ([string]::IsNullOrEmpty($Value)) {
        return "<empty>"
    }

    $prefixLength = [Math]::Min(3, $Value.Length)
    $suffixLength = [Math]::Min(3, $Value.Length)
    $prefix = $Value.Substring(0, $prefixLength)
    $suffix = $Value.Substring($Value.Length - $suffixLength, $suffixLength)
    $hasWhitespace = $Value -match "\s"

    return ("len={0}, prefix={1}, suffix={2}, has_whitespace={3}" -f $Value.Length, $prefix, $suffix, $hasWhitespace)
}


function Assert-EnvCompat {
    param(
        [string]$Primary,
        [string]$Fallback
    )

    if ([string]::IsNullOrWhiteSpace((Get-EnvCompat -Primary $Primary -Fallback $Fallback))) {
        throw "缺少环境变量: $Primary 或 $Fallback"
    }
}

function Resolve-AuthMode {
    $mode = Get-EnvCompat -Primary "FPT_AUTH_MODE" -Fallback "SG_AUTH_MODE"
    if (-not [string]::IsNullOrWhiteSpace($mode)) {
        return $mode.Trim().ToLower().Replace("-", "_")
    }

    # 与 CLI 保持一致的推断优先级：user_password > session_token > script
    $username = Get-EnvCompat -Primary "FPT_USERNAME" -Fallback "SG_USERNAME"
    $password = Get-EnvCompat -Primary "FPT_PASSWORD" -Fallback "SG_PASSWORD"
    $authToken = Get-EnvCompat -Primary "FPT_AUTH_TOKEN" -Fallback "SG_AUTH_TOKEN"
    if (-not [string]::IsNullOrWhiteSpace($username) -or -not [string]::IsNullOrWhiteSpace($password) -or -not [string]::IsNullOrWhiteSpace($authToken)) {
        return "user_password"
    }

    $sessionToken = Get-EnvCompat -Primary "FPT_SESSION_TOKEN" -Fallback "SG_SESSION_TOKEN"
    if (-not [string]::IsNullOrWhiteSpace($sessionToken)) {
        return "session_token"
    }

    return "script"
}

function Assert-AuthConfig {
    param([string]$Mode)

    switch ($Mode) {
        "script" {
            Assert-EnvCompat -Primary "FPT_SCRIPT_NAME" -Fallback "SG_SCRIPT_NAME"
            Assert-EnvCompat -Primary "FPT_SCRIPT_KEY" -Fallback "SG_SCRIPT_KEY"
        }
        "user_password" {
            Assert-EnvCompat -Primary "FPT_USERNAME" -Fallback "SG_USERNAME"
            Assert-EnvCompat -Primary "FPT_PASSWORD" -Fallback "SG_PASSWORD"
        }
        "session_token" {
            Assert-EnvCompat -Primary "FPT_SESSION_TOKEN" -Fallback "SG_SESSION_TOKEN"
        }
        default {
            throw "不支持的 FPT_AUTH_MODE: $Mode（可选: script / user_password / session_token）"
        }
    }
}

function Build-AuthArgs {
    param([string]$Mode)

    $site = Get-EnvCompat -Primary "FPT_SITE" -Fallback "SG_SITE"
    # 策略：只通过命令行传 --site 和 --auth-mode，
    # 凭据（script_key、password 等）全部通过环境变量传递。
    # 原因：Windows 命令行参数中 ^、&、!、% 等特殊字符会被 shell 转义/吞掉，
    # 而环境变量是进程级别继承的，不经过命令行解析，特殊字符安全无损。
    $cliArgs = @("--site", $site, "--auth-mode")

    switch ($Mode) {
        "script" {
            $cliArgs += "script"
            # script_name / script_key 由 CLI 从 FPT_SCRIPT_NAME/FPT_SCRIPT_KEY 环境变量读取
        }
        "user_password" {
            $cliArgs += "user-password"
            # username / password / auth_token 由 CLI 从 FPT_USERNAME/FPT_PASSWORD/FPT_AUTH_TOKEN 环境变量读取
        }
        "session_token" {
            $cliArgs += "session-token"
            # session_token 由 CLI 从 FPT_SESSION_TOKEN 环境变量读取
        }
        default {
            throw "不支持的认证模式: $Mode"
        }
    }

    return $cliArgs
}

# 最低要求：站点
Assert-EnvCompat -Primary "FPT_SITE" -Fallback "SG_SITE"

$authMode = $AuthMode
if ($authMode -eq "auto") {
    $authMode = Resolve-AuthMode
    if ([string]::IsNullOrWhiteSpace((Get-EnvCompat -Primary "FPT_AUTH_MODE" -Fallback "SG_AUTH_MODE"))) {
        Write-Host "[提示] 未设置 FPT_AUTH_MODE/SG_AUTH_MODE，已按环境自动推断为: $authMode" -ForegroundColor Yellow
    }
} else {
    Write-Host "[提示] 已使用命令行参数强制认证模式: $authMode" -ForegroundColor Yellow
}

$currentSite = Get-EnvCompat -Primary "FPT_SITE" -Fallback "SG_SITE"
Write-Host ("[信息] 当前认证模式: {0}" -f $authMode) -ForegroundColor Cyan
Write-Host ("[信息] 当前站点: {0}" -f $currentSite) -ForegroundColor Cyan

if ($authMode -eq "script") {
    $scriptNameValue = Get-EnvCompat -Primary "FPT_SCRIPT_NAME" -Fallback "SG_SCRIPT_NAME"
    $scriptKeyValue = Get-EnvCompat -Primary "FPT_SCRIPT_KEY" -Fallback "SG_SCRIPT_KEY"
    Write-Host ("[诊断] script_name='{0}', script_key_fingerprint={1}" -f $scriptNameValue, (Get-SecretFingerprint -Value $scriptKeyValue)) -ForegroundColor DarkCyan
    # 检查是否含有 Windows 命令行敏感的特殊字符
    $dangerousChars = [regex]::Matches($scriptKeyValue, '[^\w.~-]')
    if ($dangerousChars.Count -gt 0) {
        $charList = ($dangerousChars | ForEach-Object { "'" + $_.Value + "'" }) -join ", "
        Write-Host ("[诊断] script_key 含特殊字符: {0} — 已通过环境变量传递（避免命令行转义问题）" -f $charList) -ForegroundColor DarkYellow
    }
    Write-Host "[诊断] 凭据传递方式: 环境变量（非命令行参数，特殊字符安全）" -ForegroundColor DarkCyan
}

if ($authMode -eq "user_password") {

    $authTokenValue = Get-EnvCompat -Primary "FPT_AUTH_TOKEN" -Fallback "SG_AUTH_TOKEN"
    if (-not [string]::IsNullOrWhiteSpace($authTokenValue) -and ($authTokenValue -notmatch '^\d{6}$')) {
        Write-Host "[警告] 在 user_password 模式下，FPT_AUTH_TOKEN/SG_AUTH_TOKEN 通常应为 2FA 一次性 6 位验证码，而不是长 token。" -ForegroundColor Yellow
    }
}

try {
    Assert-AuthConfig $authMode
} catch {
    Write-Host "[错误] 认证配置不完整：$($_.Exception.Message)" -ForegroundColor Red
    Write-Host ""
    Write-Host "可参考以下任一方式设置后重试：" -ForegroundColor Yellow
    Write-Host "# script 模式" -ForegroundColor DarkYellow
    Write-Host '$env:FPT_AUTH_MODE = "script"'
    Write-Host '$env:FPT_SCRIPT_NAME = "your-script-name"'
    Write-Host '$env:FPT_SCRIPT_KEY = "your-script-key"'
    Write-Host ""
    Write-Host "# user_password 模式" -ForegroundColor DarkYellow
    Write-Host '$env:FPT_AUTH_MODE = "user_password"'
    Write-Host '$env:FPT_USERNAME = "you@example.com"'
    Write-Host '$env:FPT_PASSWORD = "your-password"'
    Write-Host '# 若站点开启 2FA，还需要：$env:FPT_AUTH_TOKEN = "123456"'
    Write-Host ""
    Write-Host "# session_token 模式" -ForegroundColor DarkYellow
    Write-Host '$env:FPT_AUTH_MODE = "session_token"'
    Write-Host '$env:FPT_SESSION_TOKEN = "your-session-token"'
    throw
}

$authArgs = Build-AuthArgs $authMode

Write-Host "[步骤] 先执行 auth test 做认证预检..." -ForegroundColor Cyan

# 先用原生 PowerShell HTTP 请求独立验证凭据（绕过 CLI，排查是凭据问题还是 CLI 问题）
function New-FormUrlEncodedBody {
    param([hashtable]$Fields)

    $pairs = [System.Collections.Generic.List[System.Collections.Generic.KeyValuePair[string,string]]]::new()
    foreach ($entry in $Fields.GetEnumerator()) {
        $pairs.Add([System.Collections.Generic.KeyValuePair[string,string]]::new([string]$entry.Key, [string]$entry.Value))
    }

    $content = [System.Net.Http.FormUrlEncodedContent]::new($pairs)
    try {
        return $content.ReadAsStringAsync().GetAwaiter().GetResult()
    } finally {
        $content.Dispose()
    }
}

function Test-ShotGridAuthDirect {
    param([string]$Mode)

    $site = Get-EnvCompat -Primary "FPT_SITE" -Fallback "SG_SITE"
    $apiVer = Get-EnvCompat -Primary "FPT_API_VERSION" -Fallback "SG_API_VERSION"
    if ([string]::IsNullOrWhiteSpace($apiVer)) { $apiVer = "v1.1" }
    $tokenUrl = "$site/api/$apiVer/auth/access_token"

    $formFields = @{}
    $curlData = $null
    switch ($Mode) {
        "script" {
            $sn = Get-EnvCompat -Primary "FPT_SCRIPT_NAME" -Fallback "SG_SCRIPT_NAME"
            $sk = Get-EnvCompat -Primary "FPT_SCRIPT_KEY" -Fallback "SG_SCRIPT_KEY"
            $formFields = @{
                grant_type    = "client_credentials"
                client_id     = $sn
                client_secret = $sk
            }
            $skMasked = "***REDACTED(len=$($sk.Length))***"
            $curlData = "grant_type=client_credentials&client_id=$sn&client_secret=$skMasked"
        }
        "user_password" {
            $un = Get-EnvCompat -Primary "FPT_USERNAME" -Fallback "SG_USERNAME"
            $pw = Get-EnvCompat -Primary "FPT_PASSWORD" -Fallback "SG_PASSWORD"
            $formFields = @{
                grant_type = "password"
                username   = $un
                password   = $pw
            }
            $curlData = "grant_type=password&username=$un&password=***REDACTED***"
            $at = Get-EnvCompat -Primary "FPT_AUTH_TOKEN" -Fallback "SG_AUTH_TOKEN"
            if (-not [string]::IsNullOrWhiteSpace($at)) {
                $formFields.auth_token = $at
                $curlData += "&auth_token=$at"
            }
        }
        "session_token" {
            $st = Get-EnvCompat -Primary "FPT_SESSION_TOKEN" -Fallback "SG_SESSION_TOKEN"
            $formFields = @{
                grant_type    = "session_token"
                session_token = $st
            }
            $curlData = "grant_type=session_token&session_token=***REDACTED***"
        }
    }

    $formBody = New-FormUrlEncodedBody -Fields $formFields

    Write-Host ("[诊断] 等效 curl 命令：") -ForegroundColor DarkCyan
    Write-Host ("  curl -X POST `"$tokenUrl`" -H `"Accept: application/json`" -H `"Content-Type: application/x-www-form-urlencoded`" -d `"$curlData`"") -ForegroundColor DarkGray

    Write-Host "[诊断] 直接用 PowerShell HTTP 请求验证凭据..." -ForegroundColor DarkCyan
    try {
        $headers = @{ "Accept" = "application/json" }
        $resp = Invoke-WebRequest -Uri $tokenUrl -Method POST -Body $formBody -ContentType "application/x-www-form-urlencoded" -Headers $headers -SkipHttpErrorCheck -ErrorAction Stop
        $statusCode = [int]$resp.StatusCode
        $bodyText = if ($null -ne $resp.Content) { [string]$resp.Content } else { "" }

        if ($statusCode -lt 200 -or $statusCode -ge 300) {
            Write-Host ("[诊断] ✗ 直接 HTTP 验证失败 (HTTP {0}):" -f $statusCode) -ForegroundColor Red
            if (-not [string]::IsNullOrWhiteSpace($bodyText)) {
                Write-Host ("  {0}" -f $bodyText) -ForegroundColor DarkRed
            } else {
                Write-Host "  <empty response body>" -ForegroundColor DarkRed
            }
            return $false
        }

        $json = $null
        if (-not [string]::IsNullOrWhiteSpace($bodyText)) {
            try {
                $json = $bodyText | ConvertFrom-Json -ErrorAction Stop
            } catch {
                Write-Host "[诊断] ✗ 直接 HTTP 验证返回 2xx，但响应不是合法 JSON" -ForegroundColor Red
                Write-Host ("  {0}" -f $bodyText) -ForegroundColor DarkRed
                return $false
            }
        }

        if ($null -ne $json -and $null -ne $json.access_token -and -not [string]::IsNullOrWhiteSpace([string]$json.access_token)) {
            Write-Host "[诊断] ✓ 直接 HTTP 验证成功！拿到 access_token" -ForegroundColor Green
            return $true
        }

        Write-Host "[诊断] ✗ 直接 HTTP 验证返回 2xx，但响应中缺少 access_token" -ForegroundColor Red
        if (-not [string]::IsNullOrWhiteSpace($bodyText)) {
            Write-Host ("  {0}" -f $bodyText) -ForegroundColor DarkRed
        } else {
            Write-Host "  <empty response body>" -ForegroundColor DarkRed
        }
        return $false
    } catch {
        Write-Host "[诊断] ✗ 直接 HTTP 请求过程异常:" -ForegroundColor Red
        Write-Host ("  {0}" -f $_.Exception.Message) -ForegroundColor DarkRed
        return $false
    }
}

$directAuthOk = Test-ShotGridAuthDirect -Mode $authMode

if (-not $directAuthOk) {
    Write-Host ""
    Write-Host "[错误] 凭据本身无法通过 ShotGrid 认证（与 CLI 无关）。" -ForegroundColor Red
    Write-Host "排查建议：" -ForegroundColor Yellow
    Write-Host "1) 确认 FPT_SITE/SG_SITE 是否为正确站点地址"
    Write-Host "2) script 模式下确认 script_name 和 script_key 是否匹配"
    Write-Host "3) 在 ShotGrid Admin > Scripts 页面确认脚本存在且为 active 状态"
    Write-Host "4) 确认 script_key 是否已过期或被重新生成"
    Write-Host "5) user_password 模式下确认用户名/密码是否正确"
    Write-Host "6) 若站点开启 2FA，FPT_AUTH_TOKEN/SG_AUTH_TOKEN 应为当前 6 位一次性验证码"
    throw "凭据验证失败（直接 HTTP 调用 ShotGrid 也失败）"
}

# 凭据已确认有效，继续用 CLI 跑 auth test
$null = & vx cargo run -q -p fpt-cli -- auth test @authArgs --output json
if ($LASTEXITCODE -ne 0) {
    Write-Host "[错误] CLI auth test 失败，但直接 HTTP 验证成功——问题可能出在 CLI 代码中。" -ForegroundColor Red
    Write-Host "请设置 FPT_DEBUG=1 环境变量后重试，查看详细请求日志。" -ForegroundColor Yellow
    throw "auth test 失败（exit code=$LASTEXITCODE）"
}

$page = 1
$total = 0
$serverTotal = $null

while ($true) {
    $query = @{
        fields = @("id")
        page   = @{
            number = $page
            size   = $PageSize
        }
    } | ConvertTo-Json -Depth 10 -Compress

    $raw = & vx cargo run -q -p fpt-cli -- entity find Project --input $query @authArgs --output json
    if ($LASTEXITCODE -ne 0) {
        throw "fpt entity find 执行失败（exit code=$LASTEXITCODE）"
    }

    $resp = $raw | ConvertFrom-Json

    $items = @()
    if ($null -ne $resp.data) {
        $items = @($resp.data)
    }

    $countThisPage = $items.Count
    $total += $countThisPage

    if ($VerbosePage) {
        Write-Host ("page={0}, count={1}, accumulated={2}" -f $page, $countThisPage, $total)
    }

    try {
        if ($null -eq $serverTotal -and $null -ne $resp.meta -and $null -ne $resp.meta.pagination -and $null -ne $resp.meta.pagination.total) {
            $serverTotal = [int]$resp.meta.pagination.total
        }
    } catch {
        # 忽略 meta 结构不一致
    }

    if ($countThisPage -lt $PageSize) {
        break
    }

    $page += 1
}

Write-Host ""
Write-Host "=== Project 统计结果 ===" -ForegroundColor Cyan
Write-Host ("site: {0}" -f $currentSite)
Write-Host ("count_by_paging: {0}" -f $total)
if ($null -ne $serverTotal) {
    Write-Host ("count_from_meta: {0}" -f $serverTotal)
}

@{
    site = $currentSite
    count_by_paging = $total
    count_from_meta = $serverTotal
} | ConvertTo-Json -Depth 5
