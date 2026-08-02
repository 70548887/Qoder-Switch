#Requires -Version 5.1
<#
.SYNOPSIS
    规范化编译与版本归档脚本 (Qoder Proxy)
.DESCRIPTION
    以 tauri.conf.json 的 version 作为单一版本来源，执行 pnpm tauri build 生成 NSIS 安装包，
    并按版本号精确匹配后归档到 releases/v<version>/ 目录。
.EXAMPLE
    ./build.ps1
#>

$ErrorActionPreference = 'Stop'
try { [Console]::OutputEncoding = [System.Text.Encoding]::UTF8 } catch {}

# 项目根目录 = 脚本所在目录
$Root = $PSScriptRoot
Set-Location $Root

function Write-Step($msg) { Write-Host "==> $msg" -ForegroundColor Cyan }

# --- 1. 读取版本号（单一来源：tauri.conf.json） ---
$confPath = Join-Path $Root 'src-tauri\tauri.conf.json'
$conf = Get-Content $confPath -Raw | ConvertFrom-Json
$version = $conf.version
$product = $conf.productName
if ([string]::IsNullOrWhiteSpace($version)) { throw '未能从 tauri.conf.json 读取到 version 字段' }
Write-Step "产品：$product    目标版本：v$version"

# --- 2. 版本一致性校验（不一致仅告警，不中断） ---
$pkg = Get-Content (Join-Path $Root 'package.json') -Raw | ConvertFrom-Json
if ($pkg.version -ne $version) {
    Write-Warning "package.json 版本 ($($pkg.version)) 与 tauri.conf.json ($version) 不一致"
}
$cargo = Get-Content (Join-Path $Root 'src-tauri\Cargo.toml') -Raw
if ($cargo -match '(?m)^\s*version\s*=\s*"([^"]+)"' -and $matches[1] -ne $version) {
    Write-Warning "Cargo.toml 版本 ($($matches[1])) 与 tauri.conf.json ($version) 不一致"
}

# --- 3. 执行编译 ---
Write-Step '开始编译：pnpm tauri build'
pnpm tauri build
if ($LASTEXITCODE -ne 0) { throw "编译失败（退出码 $LASTEXITCODE）" }

# --- 4. 归档 NSIS 安装包（按当前版本号精确匹配） ---
$nsisDir = Join-Path $Root 'src-tauri\target\release\bundle\nsis'
$pattern = "*_${version}_*-setup.exe"
$installer = Get-ChildItem -Path $nsisDir -Filter $pattern -ErrorAction SilentlyContinue |
    Sort-Object LastWriteTime -Descending | Select-Object -First 1
if ($null -eq $installer) { throw "未找到匹配的 NSIS 安装包：$nsisDir\$pattern" }

$destDir = Join-Path $Root "releases\v$version"
New-Item -ItemType Directory -Force -Path $destDir | Out-Null
Copy-Item $installer.FullName (Join-Path $destDir $installer.Name) -Force

$sizeMB = [math]::Round($installer.Length / 1MB, 2)
Write-Host ''
Write-Step '归档完成'
Write-Host "    文件：$($installer.Name)" -ForegroundColor Green
Write-Host "    大小：$sizeMB MB" -ForegroundColor Green
Write-Host "    位置：releases\v$version\" -ForegroundColor Green
