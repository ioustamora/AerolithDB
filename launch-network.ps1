#!/usr/bin/env pwsh
# AerolithDB Multinode Test Network Launcher
# Cross-platform PowerShell script to easily launch multinode test networks

param(
    [Parameter(HelpMessage="Type of test to run: quick, full, or advanced")]
    [ValidateSet("quick", "full", "advanced")]
    [string]$TestType = "full",
    
    [Parameter(HelpMessage="Number of nodes to launch (default: 4 for full, 6 for advanced)")]
    [int]$NodesCount = 0,
    
    [Parameter(HelpMessage="Enable verbose logging")]
    [switch]$Verbose,
    
    [Parameter(HelpMessage="Test duration for advanced tests (seconds)")]
    [int]$TestDuration = 300,
    
    [Parameter(HelpMessage="Show available options and exit")]
    [switch]$Help
)

# Color output functions
function Write-ColorOutput {
    param([string]$Message, [string]$Color = "White")
    
    switch ($Color) {
        "Red" { Write-Host $Message -ForegroundColor Red }
        "Green" { Write-Host $Message -ForegroundColor Green }
        "Yellow" { Write-Host $Message -ForegroundColor Yellow }
        "Blue" { Write-Host $Message -ForegroundColor Blue }
        "Cyan" { Write-Host $Message -ForegroundColor Cyan }
        "Magenta" { Write-Host $Message -ForegroundColor Magenta }
        default { Write-Host $Message }
    }
}

function Show-Help {
    Write-ColorOutput "`n🚀 AerolithDB Multinode Test Network Launcher" "Cyan"
    Write-ColorOutput "=============================================" "Cyan"
    Write-Host ""
    Write-ColorOutput "USAGE:" "Yellow"
    Write-Host "  .\launch-network.ps1 [OPTIONS]"
    Write-Host ""
    Write-ColorOutput "TEST TYPES:" "Yellow"
    Write-Host "  quick     - 3 nodes, basic functionality demo (2-3 minutes)"
    Write-Host "  full      - 4+ nodes, comprehensive testing (5-10 minutes)"  
    Write-Host "  advanced  - 6+ nodes, all features + stress testing (10+ minutes)"
    Write-Host ""
    Write-ColorOutput "OPTIONS:" "Yellow"
    Write-Host "  -TestType     Test type to run [quick|full|advanced] (default: full)"
    Write-Host "  -NodesCount   Number of nodes (auto-selected if not specified)"
    Write-Host "  -Verbose      Enable detailed logging"
    Write-Host "  -TestDuration Duration for advanced tests in seconds (default: 300)"
    Write-Host "  -Help         Show this help message"
    Write-Host ""
    Write-ColorOutput "EXAMPLES:" "Yellow"
    Write-Host "  .\launch-network.ps1                           # Full test with 4 nodes"
    Write-Host "  .\launch-network.ps1 -TestType quick           # Quick 3-node demo"
    Write-Host "  .\launch-network.ps1 -TestType advanced -Verbose # Advanced with logging"
    Write-Host "  .\launch-network.ps1 -NodesCount 8 -Verbose    # Custom 8-node network"
    Write-Host ""
    Write-ColorOutput "AFTER LAUNCH:" "Yellow"
    Write-Host "  • Web UI: http://localhost:8080"
    Write-Host "  • Network Explorer: http://localhost:8080/explorer"
    Write-Host "  • CLI: cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8080 health"
    Write-Host ""
}

function Test-Prerequisites {
    Write-ColorOutput "🔍 Checking prerequisites..." "Blue"
    
    # Check if Cargo is available
    try {
        $cargoVersion = cargo --version 2>$null
        if ($LASTEXITCODE -eq 0) {
            Write-ColorOutput "✅ Cargo: $cargoVersion" "Green"
        } else {
            throw "Cargo not found"
        }
    } catch {
        Write-ColorOutput "❌ Cargo (Rust) is required but not found" "Red"
        Write-ColorOutput "   Please install Rust: https://rustup.rs/" "Yellow"
        return $false
    }
    
    # Check if in AerolithDB directory
    if (-not (Test-Path "Cargo.toml")) {
        Write-ColorOutput "❌ Not in AerolithDB root directory" "Red"
        Write-ColorOutput "   Please run this script from the AerolithDB project root" "Yellow"
        return $false
    }
    
    # Check if scripts exist
    $requiredScripts = @(
        "scripts\quick-demo.ps1",
        "scripts\launch-local-network.ps1",
        "scripts\demo-advanced-test.ps1"
    )
    
    foreach ($script in $requiredScripts) {
        if (-not (Test-Path $script)) {
            Write-ColorOutput "❌ Required script not found: $script" "Red"
            return $false
        }
    }
    
    Write-ColorOutput "✅ All prerequisites met" "Green"
    return $true
}

function Get-DefaultNodeCount {
    param([string]$TestType)
    
    switch ($TestType) {
        "quick" { return 3 }
        "full" { return 4 }
        "advanced" { return 6 }
        default { return 4 }
    }
}

function Show-TestInfo {
    param([string]$TestType, [int]$NodeCount, [int]$Duration)
    
    Write-ColorOutput "`n🎯 Test Configuration" "Cyan"
    Write-ColorOutput "===================" "Cyan"
    Write-Host "Test Type: $TestType"
    Write-Host "Node Count: $NodeCount"
    
    switch ($TestType) {
        "quick" {
            Write-Host "Duration: ~2-3 minutes"
            Write-ColorOutput "`nFeatures Tested:" "Yellow"
            Write-Host "  • Basic network formation"
            Write-Host "  • Simple CRUD operations"
            Write-Host "  • Cross-node replication"
            Write-Host "  • Health monitoring"
        }
        "full" {
            Write-Host "Duration: ~5-10 minutes"  
            Write-ColorOutput "`nFeatures Tested:" "Yellow"
            Write-Host "  • Complete network formation (bootstrap + $($NodeCount-1) nodes)"
            Write-Host "  • Full CRUD operations across nodes"
            Write-Host "  • Distributed queries and analytics"
            Write-Host "  • User simulation and admin operations"
            Write-Host "  • Cross-node data consistency"
            Write-Host "  • Health monitoring and statistics"
        }
        "advanced" {
            Write-Host "Duration: ~$Duration seconds + setup"
            Write-ColorOutput "`nFeatures Tested:" "Yellow"
            Write-Host "  • All full test features PLUS:"
            Write-Host "  • Byzantine fault tolerance"
            Write-Host "  • Network partition recovery"
            Write-Host "  • Cross-datacenter replication"
            Write-Host "  • Load testing and performance"
            Write-Host "  • Security and encryption"
            Write-Host "  • Compliance and governance"
        }
    }
    
    Write-ColorOutput "`nNetwork Endpoints (after launch):" "Yellow"
    for ($i = 0; $i -lt $NodeCount; $i++) {
        $port = 8080 + $i
        $nodeType = if ($i -eq 0) { "Bootstrap" } else { "Node $i" }
        Write-Host "  • $nodeType : http://localhost:$port"
    }
    Write-Host ""
}

function Launch-Test {
    param([string]$TestType, [int]$NodeCount, [bool]$VerboseMode, [int]$Duration)
    
    $scriptArgs = @()
    
    switch ($TestType) {
        "quick" {
            Write-ColorOutput "🚀 Launching Quick Demo..." "Green"
            $scriptPath = "scripts\quick-demo.ps1"
        }
        "full" {
            Write-ColorOutput "🚀 Launching Full Network Demo..." "Green"
            $scriptPath = "scripts\launch-local-network.ps1"
            $scriptArgs += "-NodesCount", $NodeCount
            if ($VerboseMode) { $scriptArgs += "-Verbose" }
        }
        "advanced" {
            Write-ColorOutput "🚀 Launching Advanced Network Test..." "Green"
            $scriptPath = "scripts\demo-advanced-test.ps1"
            $scriptArgs += "-NodesCount", $NodeCount
            $scriptArgs += "-TestDuration", $Duration
            if ($VerboseMode) { $scriptArgs += "-Verbose" }
        }
    }
    
    Write-ColorOutput "Executing: $scriptPath $($scriptArgs -join ' ')" "Blue"
    Write-Host ""
    
    try {
        if ($scriptArgs.Count -gt 0) {
            & $scriptPath @scriptArgs
        } else {
            & $scriptPath
        }
    } catch {
        Write-ColorOutput "❌ Error launching test: $($_.Exception.Message)" "Red"
        return $false
    }
    
    return $true
}

# Main execution
if ($Help) {
    Show-Help
    exit 0
}

Write-ColorOutput "🏗️  AerolithDB Multinode Test Network Launcher" "Magenta"
Write-ColorOutput "===============================================" "Magenta"

# Check prerequisites
if (-not (Test-Prerequisites)) {
    exit 1
}

# Set default node count if not specified
if ($NodesCount -eq 0) {
    $NodesCount = Get-DefaultNodeCount -TestType $TestType
}

# Show test information
Show-TestInfo -TestType $TestType -NodeCount $NodesCount -Duration $TestDuration

# Confirm launch
Write-ColorOutput "`nReady to launch? (Press Enter to continue, Ctrl+C to cancel)" "Yellow"
Read-Host

# Launch the test
if (Launch-Test -TestType $TestType -NodeCount $NodesCount -VerboseMode $Verbose -Duration $TestDuration) {
    Write-ColorOutput "`n🎉 Test network launched successfully!" "Green"
    Write-ColorOutput "Network will remain running for manual testing." "Green"
    Write-ColorOutput "Press Ctrl+C in the script window to shutdown gracefully." "Yellow"
} else {
    Write-ColorOutput "`n❌ Failed to launch test network" "Red"
    exit 1
}
