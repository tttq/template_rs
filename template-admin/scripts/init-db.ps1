<#
.SYNOPSIS
    Template Admin 数据库一键初始化脚本

.DESCRIPTION
    自动完成以下操作：
    1. 启动 Docker Compose（PostgreSQL + Redis + MinIO），可选跳过
    2. 等待 PostgreSQL 就绪
    3. 创建数据库（如不存在）
    4. 执行 init-db.sql 完成建表、初始数据、菜单权限、密码 hash
    5. 验证初始化结果

.PARAMETER SkipDocker
    跳过 Docker Compose 启动步骤（适用于已运行 PostgreSQL 的环境）

.PARAMETER DatabaseUrl
    数据库连接 URL，默认读取 .env 或使用 postgres://postgres:root@localhost:5432/template

.PARAMETER Force
    跳过交互确认，直接执行（用于 CI/CD）

.EXAMPLE
    # 默认执行：启动 Docker + 初始化数据库
    .\scripts\init-db.ps1

.EXAMPLE
    # 跳过 Docker，使用自定义数据库 URL
    .\scripts\init-db.ps1 -SkipDocker -DatabaseUrl "postgres://postgres:123456@localhost:5432/template"

.EXAMPLE
    # 强制执行（无交互确认）
    .\scripts\init-db.ps1 -Force
#>

param(
    [switch]$SkipDocker,
    [string]$DatabaseUrl,
    [switch]$Force
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot

# ============================================================================
# 辅助函数
# ============================================================================
function Write-Step { param([string]$Message) Write-Host "`n[*] $Message" -ForegroundColor Cyan }
function Write-Ok { param([string]$Message) Write-Host "    [OK] $Message" -ForegroundColor Green }
function Write-Warn { param([string]$Message) Write-Host "    [!] $Message" -ForegroundColor Yellow }
function Write-Err { param([string]$Message) Write-Host "    [X] $Message" -ForegroundColor Red }

function Parse-DbUrl {
    param([string]$Url)
    # 解析 postgres://user:pass@host:port/dbname
    if ($Url -match '^postgres://([^:]+):([^@]+)@([^:]+):(\d+)/(.+)$') {
        return @{
            User = $matches[1]
            Password = $matches[2]
            Host = $matches[3]
            Port = [int]$matches[4]
            DbName = $matches[5]
        }
    }
    throw "无法解析数据库 URL: $Url"
}

function Test-PostgresReady {
    param(
        [string]$Host,
        [int]$Port,
        [string]$User,
        [string]$Password
    )
    $env:PGPASSWORD = $Password
    try {
        $result = & psql -h $Host -p $Port -U $User -d postgres -c "SELECT 1" -t -A 2>$null
        return ($LASTEXITCODE -eq 0 -and $result -match "1")
    } catch {
        return $false
    }
}

function Wait-PostgresReady {
    param(
        [string]$Host,
        [int]$Port,
        [string]$User,
        [string]$Password,
        [int]$TimeoutSeconds = 60
    )
    $elapsed = 0
    $interval = 2
    while ($elapsed -lt $TimeoutSeconds) {
        if (Test-PostgresReady -Host $Host -Port $Port -User $User -Password $Password) {
            return $true
        }
        Write-Host "    等待 PostgreSQL 就绪... ($elapsed / $TimeoutSeconds s)" -ForegroundColor Gray
        Start-Sleep -Seconds $interval
        $elapsed += $interval
    }
    return $false
}

# ============================================================================
# 1. 读取配置
# ============================================================================
Write-Step "读取配置"

# 优先使用参数指定的 URL，其次读取 .env，最后使用默认值
if (-not $DatabaseUrl) {
    $envFile = Join-Path $ProjectRoot ".env"
    if (Test-Path $envFile) {
        $envContent = Get-Content $envFile -Encoding UTF8
        $dbUrlLine = $envContent | Where-Object { $_ -match '^DATABASE_URL=' } | Select-Object -First 1
        if ($dbUrlLine) {
            $DatabaseUrl = $dbUrlLine -replace '^DATABASE_URL=', '' -replace '"', ''
        }
    }
    if (-not $DatabaseUrl) {
        $DatabaseUrl = "postgres://postgres:root@localhost:5432/template"
        Write-Warn "未找到 DATABASE_URL 配置，使用默认值: $DatabaseUrl"
    }
}

$dbConfig = Parse-DbUrl -Url $DatabaseUrl
Write-Ok "数据库: $($dbConfig.Host):$($dbConfig.Port)/$($dbConfig.DbName) (用户: $($dbConfig.User))"

# ============================================================================
# 2. 交互确认
# ============================================================================
if (-not $Force) {
    $warningMsg = @"
即将执行以下操作：
  1. $(if ($SkipDocker) { '跳过' } else { '启动' }) Docker Compose
  2. 等待 PostgreSQL 就绪
  3. 创建数据库 $($dbConfig.DbName)（如不存在）
  4. 执行 init-db.sql（DROP 所有表并重建，会清空已有数据）

确认继续？(y/N)
"@
    Write-Host $warningMsg -ForegroundColor Yellow
    $confirm = Read-Host
    if ($confirm -ne 'y' -and $confirm -ne 'Y') {
        Write-Host "已取消" -ForegroundColor Gray
        exit 0
    }
}

# ============================================================================
# 3. 启动 Docker Compose
# ============================================================================
if (-not $SkipDocker) {
    Write-Step "启动 Docker Compose (PostgreSQL + Redis + MinIO)"
    $composeFile = Join-Path $ProjectRoot "docker-compose.yml"
    if (-not (Test-Path $composeFile)) {
        Write-Err "未找到 docker-compose.yml: $composeFile"
        exit 1
    }

    # 检查 docker 是否可用
    $dockerCmd = Get-Command docker -ErrorAction SilentlyContinue
    if (-not $dockerCmd) {
        Write-Err "未找到 docker 命令，请先安装 Docker 或使用 -SkipDocker 参数跳过"
        exit 1
    }

    Push-Location $ProjectRoot
    try {
        & docker compose up -d
        if ($LASTEXITCODE -ne 0) {
            Write-Err "Docker Compose 启动失败"
            exit 1
        }
        Write-Ok "Docker Compose 已启动"
    } finally {
        Pop-Location
    }
} else {
    Write-Step "跳过 Docker Compose 启动"
}

# ============================================================================
# 4. 等待 PostgreSQL 就绪
# ============================================================================
Write-Step "等待 PostgreSQL 就绪"
$ready = Wait-PostgresReady -Host $dbConfig.Host -Port $dbConfig.Port -User $dbConfig.User -Password $dbConfig.Password -TimeoutSeconds 60
if (-not $ready) {
    Write-Err "PostgreSQL 在 60 秒内未就绪，请检查 Docker 容器状态或数据库连接配置"
    exit 1
}
Write-Ok "PostgreSQL 已就绪"

# ============================================================================
# 5. 创建数据库（如不存在）
# ============================================================================
Write-Step "检查/创建数据库 $($dbConfig.DbName)"
$env:PGPASSWORD = $dbConfig.Password

# 检查数据库是否存在
$existsResult = & psql -h $dbConfig.Host -p $dbConfig.Port -U $dbConfig.User -d postgres -t -A -c "SELECT 1 FROM pg_database WHERE datname = '$($dbConfig.DbName)'" 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Err "查询数据库列表失败"
    exit 1
}

if ($existsResult -match "1") {
    Write-Ok "数据库 $($dbConfig.DbName) 已存在"
} else {
    & psql -h $dbConfig.Host -p $dbConfig.Port -U $dbConfig.User -d postgres -c "CREATE DATABASE $($dbConfig.DbName)"
    if ($LASTEXITCODE -ne 0) {
        Write-Err "创建数据库 $($dbConfig.DbName) 失败"
        exit 1
    }
    Write-Ok "已创建数据库 $($dbConfig.DbName)"
}

# ============================================================================
# 6. 执行 init-db.sql
# ============================================================================
Write-Step "执行 init-db.sql 初始化脚本"
$sqlFile = Join-Path $PSScriptRoot "init-db.sql"
if (-not (Test-Path $sqlFile)) {
    Write-Err "未找到 SQL 文件: $sqlFile"
    exit 1
}

# 执行 SQL 文件，输出 NOTICE 和验证结果
$psqlOutput = & psql -h $dbConfig.Host -p $dbConfig.Port -U $dbConfig.User -d $dbConfig.DbName -f $sqlFile 2>&1
$psqlExitCode = $LASTEXITCODE

# 显示 psql 输出（包含 NOTICE 和 RAISE 消息）
$psqlOutput | ForEach-Object {
    if ($_ -match "NOTICE") {
        Write-Host "    $_" -ForegroundColor Gray
    } elseif ($_ -match "^=" -or $_ -match "数据库初始化完成" -or $_ -match "默认登录") {
        Write-Host "    $_" -ForegroundColor Green
    } elseif ($_ -match "^\s") {
        Write-Host "    $_" -ForegroundColor Gray
    } else {
        Write-Host "    $_" -ForegroundColor Gray
    }
}

if ($psqlExitCode -ne 0) {
    Write-Err "SQL 执行失败 (exit code: $psqlExitCode)"
    exit 1
}
Write-Ok "SQL 脚本执行完成"

# ============================================================================
# 7. 验证初始化结果
# ============================================================================
Write-Step "验证初始化结果"

$verifyQueries = @(
    @{ Name = "租户数"; Sql = "SELECT COUNT(*) FROM auth_sys_tenant" },
    @{ Name = "用户数"; Sql = "SELECT COUNT(*) FROM auth_sys_user" },
    @{ Name = "角色数"; Sql = "SELECT COUNT(*) FROM auth_sys_role" },
    @{ Name = "菜单数"; Sql = "SELECT COUNT(*) FROM auth_sys_menu" },
    @{ Name = "角色-菜单关联数"; Sql = "SELECT COUNT(*) FROM auth_sys_role_menu" },
    @{ Name = "字典类型数"; Sql = "SELECT COUNT(*) FROM auth_sys_dict_type" },
    @{ Name = "字典项数"; Sql = "SELECT COUNT(*) FROM auth_sys_dict_item" },
    @{ Name = "系统配置数"; Sql = "SELECT COUNT(*) FROM auth_sys_config" }
)

foreach ($q in $verifyQueries) {
    $count = & psql -h $dbConfig.Host -p $dbConfig.Port -U $dbConfig.User -d $dbConfig.DbName -t -A -c $q.Sql 2>$null
    Write-Host "    $($q.Name): $count" -ForegroundColor White
}

# 验证 admin 密码 hash 是否正确
$pwHash = & psql -h $dbConfig.Host -p $dbConfig.Port -U $dbConfig.User -d $dbConfig.DbName -t -A -c "SELECT pass_word FROM auth_sys_user WHERE user_name = 'admin'" 2>$null
if ($pwHash -match '^\$2[aby]\$') {
    Write-Ok "admin 密码已正确 hash (bcrypt 格式)"
} else {
    Write-Warn "admin 密码 hash 异常: $pwHash"
}

# ============================================================================
# 8. 输出完成信息
# ============================================================================
Write-Host ""
Write-Host "================================================" -ForegroundColor Green
Write-Host " 数据库初始化完成" -ForegroundColor Green
Write-Host "================================================" -ForegroundColor Green
Write-Host " 默认登录账号: admin" -ForegroundColor Yellow
Write-Host " 默认登录密码: admin123" -ForegroundColor Yellow
Write-Host "================================================" -ForegroundColor Green
Write-Host ""
Write-Host "后续步骤：" -ForegroundColor Cyan
Write-Host "  1. 启动后端: cd template-admin && cargo run" -ForegroundColor White
Write-Host "  2. 启动前端: cd template-web && pnpm dev" -ForegroundColor White
Write-Host ""
