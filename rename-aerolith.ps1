#!/usr/bin/env powershell
# Aerolith Comprehensive Renaming Script
# This script will rename all references from aerolithsdb/aerolithsDB/aerolithDB to aerolith

Write-Host "=== Aerolith Comprehensive Renaming Script ===" -ForegroundColor Green
Write-Host "This script will update all file contents to use 'aerolith' instead of 'aerolithsdb'" -ForegroundColor Yellow
Write-Host ""

# Stop on any error
$ErrorActionPreference = "Stop"

# Define the renaming patterns
$renamingPatterns = @{
    # Package and crate names
    "aerolithsdb-core" = "aerolith-core"
    "aerolithsdb-consensus" = "aerolith-consensus"
    "aerolithsdb-storage" = "aerolith-storage" 
    "aerolithsdb-network" = "aerolith-network"
    "aerolithsdb-cache" = "aerolith-cache"
    "aerolithsdb-security" = "aerolith-security"
    "aerolithsdb-query" = "aerolith-query"
    "aerolithsdb-api" = "aerolith-api"
    "aerolithsdb-plugins" = "aerolith-plugins"
    "aerolithsdb-cli" = "aerolith-cli"
    "aerolithsdb" = "aerolith"
    
    # Class and struct names (most specific first)
    "aerolithsDB" = "Aerolith"
    "aerolithsClient" = "AerolithClient"
    "aerolithsConfig" = "AerolithConfig"
    "aerolithsDBBattleTest" = "AerolithBattleTest"
    
    # Environment variables and constants
    "aerolithSDB_" = "AEROLITH_"
    "aerolithsdb=" = "aerolith="
}

# File types to process
$fileTypes = @("*.rs", "*.toml", "*.md", "*.ps1", "*.yaml", "*.yml", "*.json", "*.txt")

Write-Host "Finding files to process..." -ForegroundColor Cyan

# Get all files to process
$filesToProcess = @()
foreach ($fileType in $fileTypes) {
    $files = Get-ChildItem -Path . -Recurse -Include $fileType | Where-Object { 
        # Exclude target directory and other build artifacts
        $_.FullName -notlike "*\target\*" -and 
        $_.FullName -notlike "*\.git\*" -and
        $_.Name -ne "rename-directories.ps1" -and
        $_.Name -ne "rename-aerolith.ps1"
    }
    $filesToProcess += $files
}

Write-Host "Found $($filesToProcess.Count) files to process" -ForegroundColor Green
Write-Host ""

Write-Host "Processing files..." -ForegroundColor Cyan

$processedCount = 0
$changedCount = 0

foreach ($file in $filesToProcess) {
    $processedCount++
    Write-Progress -Activity "Processing files" -Status "Processing $($file.Name)" -PercentComplete (($processedCount / $filesToProcess.Count) * 100)
    
    try {
        $content = Get-Content $file.FullName -Raw -Encoding UTF8
        $originalContent = $content
        
        # Apply all renaming patterns
        foreach ($pattern in $renamingPatterns.GetEnumerator()) {
            $content = $content -replace [regex]::Escape($pattern.Key), $pattern.Value
        }
        
        # Only write if content changed
        if ($content -ne $originalContent) {
            Set-Content -Path $file.FullName -Value $content -Encoding UTF8 -NoNewline
            $changedCount++
            Write-Host "  ✓ $($file.Name)" -ForegroundColor Green
        }
    }
    catch {
        Write-Host "  ✗ Error processing $($file.Name): $($_.Exception.Message)" -ForegroundColor Red
    }
}

Write-Progress -Activity "Processing files" -Completed

Write-Host ""
Write-Host "=== Renaming Summary ===" -ForegroundColor Green
Write-Host "Files processed: $processedCount" -ForegroundColor Cyan
Write-Host "Files changed: $changedCount" -ForegroundColor Cyan
Write-Host ""

if ($changedCount -gt 0) {
    Write-Host "✅ Renaming completed successfully!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Next steps:" -ForegroundColor Yellow
    Write-Host "1. Run 'cargo check' to verify everything compiles" -ForegroundColor White
    Write-Host "2. Run 'cargo test' to verify tests still pass" -ForegroundColor White
    Write-Host "3. Update any remaining manual references" -ForegroundColor White
} else {
    Write-Host "ℹ️  No files needed changes" -ForegroundColor Yellow
}
