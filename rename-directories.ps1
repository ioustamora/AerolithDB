#!/usr/bin/env powershell
# Aerolith Renaming Script - Phase 1: Directory Structure

Write-Host "=== Aerolith Renaming Script - Phase 1: Directories ===" -ForegroundColor Green
Write-Host "This script will rename all directories from aerolithdb-* to aerolith-*" -ForegroundColor Yellow
Write-Host ""

# Stop on any error
$ErrorActionPreference = "Stop"

# Define directory mappings
$directoryMappings = @{
    "aerolithdb-core" = "aerolith-core"
    "aerolithdb-consensus" = "aerolith-consensus" 
    "aerolithdb-storage" = "aerolith-storage"
    "aerolithdb-network" = "aerolith-network"
    "aerolithdb-cache" = "aerolith-cache"
    "aerolithdb-security" = "aerolith-security"
    "aerolithdb-query" = "aerolith-query"
    "aerolithdb-api" = "aerolith-api"
    "aerolithdb-plugins" = "aerolith-plugins"
    "aerolithdb-cli" = "aerolith-cli"
}

Write-Host "Renaming directories..." -ForegroundColor Cyan

foreach ($mapping in $directoryMappings.GetEnumerator()) {
    $oldName = $mapping.Key
    $newName = $mapping.Value
    
    if (Test-Path $oldName) {
        Write-Host "  $oldName -> $newName" -ForegroundColor Green
        Rename-Item $oldName $newName -Force
    } else {
        Write-Host "  SKIP: $oldName (not found)" -ForegroundColor Yellow
    }
}

Write-Host ""
Write-Host "✅ Directory renaming completed!" -ForegroundColor Green
Write-Host "Next: Run the main renaming script to update all file contents" -ForegroundColor Yellow
