#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Builds the site with kivro-supplied config and uploads dist/ to Cloudflare Pages.

.DESCRIPTION
    Cloudflare's build image has no trunk/wasm-bindgen/wasm-opt and a cold Leptos
    release build exceeds its time budget, so we build here and Direct Upload the
    artifacts. Build config never leaves this machine.

.PARAMETER Env
    kivro environment to pull config from. Defaults to the project default (dev).

.PARAMETER Branch
    Cloudflare Pages branch for a preview deployment, served at
    <branch>.infinity-website-ac6.pages.dev. Defaults to "main".

.PARAMETER Production
    Deploy to the project's production branch, which serves the live domains.
    Cannot be combined with -Branch.

.PARAMETER SkipBuild
    Upload the existing dist/ without rebuilding.

.EXAMPLE
    ./scripts/deploy.ps1 -Env prod -Production
    ./scripts/deploy.ps1 -Branch preview-auth
#>
[CmdletBinding()]
param(
    [string]$Env,
    [string]$Branch = "main",
    [switch]$Production,
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"

$ProjectName = "infinity-website"
$ProductionBranch = "master"

if ($Production)
{
    if ($PSBoundParameters.ContainsKey("Branch"))
    {
        throw "-Production deploys to '$ProductionBranch'; drop -Branch."
    }
    $Branch = $ProductionBranch
}
$Root = Split-Path -Parent $PSScriptRoot
$Dist = Join-Path $Root "dist"

$MaxFileBytes = 25 * 1024 * 1024

function Write-Step($msg)
{ Write-Host "`n==> $msg" -ForegroundColor Cyan
}
function Write-Ok($msg)
{ Write-Host "    $msg" -ForegroundColor Green
}

Push-Location $Root
try
{
    $Npx = if ($env:OS -eq "Windows_NT")
    { "npx.cmd"
    } else
    { "npx"
    }

    foreach ($tool in @("kivro", "trunk", $Npx))
    {
        if (-not (Get-Command $tool -ErrorAction SilentlyContinue))
        {
            throw "$tool not found on PATH."
        }
    }

    $kivroArgs = if ($Env)
    { @("-e", $Env)
    } else
    { @()
    }

    Write-Step "Checking config ($(if ($Env) { $Env } else { 'default env' }))"
    & kivro @kivroArgs status
    if ($LASTEXITCODE -ne 0)
    {
        throw "kivro reports missing config. Run 'kivro set <NAME>' for each one listed above."
    }

    if (-not $SkipBuild)
    {
        Write-Step "Building release bundle"
        if (Test-Path $Dist)
        { Remove-Item $Dist -Recurse -Force
        }
        $prevRustflags = $env:RUSTFLAGS
        $env:RUSTFLAGS = " "
        try
        {
            & kivro @kivroArgs run -- trunk build --release
        } finally
        {
            $env:RUSTFLAGS = $prevRustflags
        }
        if ($LASTEXITCODE -ne 0)
        { throw "trunk build failed."
        }
    } elseif (-not (Test-Path $Dist))
    {
        throw "-SkipBuild was passed but dist/ does not exist."
    }

    Write-Step "Verifying artifacts"

    $oversized = Get-ChildItem $Dist -Recurse -File |
        Where-Object { $_.Length -gt $MaxFileBytes }
    if ($oversized)
    {
        $oversized | ForEach-Object {
            Write-Host "    $($_.Name) is $([math]::Round($_.Length / 1MB, 1)) MB" -ForegroundColor Red
        }
        throw "File(s) exceed Cloudflare's 25 MiB limit. Did a debug build land in dist/?"
    }

    foreach ($required in @("index.html", "_redirects", "_headers"))
    {
        if (-not (Test-Path (Join-Path $Dist $required)))
        {
            throw "dist/$required is missing. Without _redirects every deep link 404s."
        }
    }

    $wasm = Get-ChildItem $Dist -Filter "*.wasm" | Select-Object -First 1
    if ($wasm)
    { Write-Ok "wasm: $([math]::Round($wasm.Length / 1MB, 2)) MB"
    }
    Write-Ok "$((Get-ChildItem $Dist -Recurse -File).Count) files ready"

    $target = if ($Branch -eq $ProductionBranch)
    { "PRODUCTION"
    } else
    { "preview"
    }
    Write-Step "Deploying to Cloudflare Pages ($ProjectName, branch: $Branch, $target)"
    & $Npx --yes wrangler@latest pages deploy $Dist `
        --project-name=$ProjectName `
        --branch=$Branch
    if ($LASTEXITCODE -ne 0)
    { throw "wrangler deploy failed."
    }

    Write-Ok "Deployed."
} finally
{
    Pop-Location
}
