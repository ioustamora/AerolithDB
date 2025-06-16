#!/usr/bin/env powershell
# Aerolith Renaming Verification Script

Write-Host "=== Aerolith Renaming Verification ===" -ForegroundColor Green
Write-Host ""

# Check for remaining old references
$oldPatterns = @(
    "aerolithsdb",
    "aerolithsDB", 
    "aerolithDB",
    "aerolithsClient",
    "aerolithsConfig",
    "aerolithSDB_"
)

$foundIssues = $false

Write-Host "Checking for remaining old references..." -ForegroundColor Cyan
Write-Host ""

foreach ($pattern in $oldPatterns) {
    Write-Host "Searching for '$pattern'..." -ForegroundColor Yellow
      $searchResults = Select-String -Path "*.rs", "*.toml", "*.md", "*.ps1" -Pattern $pattern -Exclude "rename-*.ps1", "verify-renaming.ps1" -ErrorAction SilentlyContinue
    
    if ($searchResults) {
        $foundIssues = $true
        Write-Host "  Found $($searchResults.Count) matches:" -ForegroundColor Red
        foreach ($match in $searchResults) {
            Write-Host "    $($match.Filename):$($match.LineNumber): $($match.Line.Trim())" -ForegroundColor Red
        }
    } else {
        Write-Host "  ✓ No matches found" -ForegroundColor Green
    }
    Write-Host ""
}

# Check directory structure
Write-Host "Checking directory structure..." -ForegroundColor Cyan
$expectedDirs = @(
    "aerolith-core",
    "aerolith-consensus", 
    "aerolith-storage",
    "aerolith-network",
    "aerolith-cache",
    "aerolith-security",
    "aerolith-query",
    "aerolith-api",
    "aerolith-plugins",
    "aerolith-cli"
)

foreach ($dir in $expectedDirs) {
    if (Test-Path $dir) {
        Write-Host "  ✓ $dir" -ForegroundColor Green
    } else {
        Write-Host "  ✗ $dir (missing)" -ForegroundColor Red
        $foundIssues = $true
    }
}

Write-Host ""

if ($foundIssues) {
    Write-Host "❌ Issues found! Please review and fix the above problems." -ForegroundColor Red
} else {
    Write-Host "✅ All checks passed! Renaming appears successful." -ForegroundColor Green
    Write-Host ""
    Write-Host "Recommended next steps:" -ForegroundColor Yellow
    Write-Host "1. cargo check --workspace" -ForegroundColor White
    Write-Host "2. cargo test --workspace" -ForegroundColor White
    Write-Host "3. cargo build --workspace" -ForegroundColor White
}
