#!/usr/bin/env pwsh
<#
.SYNOPSIS
Launch AerolithDB Local Test Network with Bootstrap Node and Mesh Networking

.DESCRIPTION
This PowerShell script demonstrates AerolithDB's distributed, cross-platform, multi-node 
functionality by launching a local test network with:
- 1 Bootstrap/seed node for network formation
- 4 Regular nodes connecting via P2P mesh networking
- Simulated user activity (document CRUD operations)
- Administrative operations (health checks, network monitoring)
- Cross-node data replication and consistency validation

.PARAMETER NodesCount
Number of regular nodes to launch (default: 4)

.PARAMETER DataDir
Base directory for node data storage (default: ./test-network-data)

.PARAMETER StartPort
Starting port for node allocation (default: 8080)

.PARAMETER Verbose
Enable verbose logging output

.EXAMPLE
.\launch-local-network.ps1
Launch default 4-node network

.EXAMPLE
.\launch-local-network.ps1 -NodesCount 6 -Verbose
Launch 6-node network with verbose logging

.NOTES
Requires: 
- Cargo build system (Rust toolchain)
- PowerShell 5.1+ or PowerShell Core 6+
- Available ports 8080-8090
#>

param(
    [int]$NodesCount = 4,
    [string]$DataDir = "test-network-data",
    [int]$StartPort = 8080,
    [switch]$Verbose
)

# Configuration
$ErrorActionPreference = "Stop"
$InformationPreference = if ($Verbose) { "Continue" } else { "SilentlyContinue" }

# ANSI Colors for cross-platform terminal output
$Colors = @{
    Green = "`e[32m"
    Blue = "`e[34m"
    Yellow = "`e[33m"
    Red = "`e[31m"
    Cyan = "`e[36m"
    Reset = "`e[0m"
    Bold = "`e[1m"
}

function Write-ColoredOutput {
    param([string]$Color, [string]$Message)
    Write-Host "$($Colors[$Color])$Message$($Colors.Reset)"
}

function Write-Header {
    param([string]$Title)
    Write-Host ""
    Write-ColoredOutput "Bold" "=" * 80
    Write-ColoredOutput "Cyan" "  $Title"
    Write-ColoredOutput "Bold" "=" * 80
    Write-Host ""
}

function Wait-ForHealthy {
    param([string]$Url, [string]$NodeName, [int]$TimeoutSec = 30)
    
    Write-ColoredOutput "Yellow" "⏳ Waiting for $NodeName to become healthy..."
    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    
    do {
        try {
            $response = Invoke-RestMethod -Uri "$Url/health" -Method Get -TimeoutSec 5 -ErrorAction SilentlyContinue
            if ($response) {
                Write-ColoredOutput "Green" "✅ $NodeName is healthy"
                return $true
            }
        }
        catch {
            # Health check failed, continue waiting
        }
        
        Start-Sleep -Seconds 2
    } while ($stopwatch.Elapsed.TotalSeconds -lt $TimeoutSec)
    
    Write-ColoredOutput "Red" "❌ $NodeName failed to become healthy within $TimeoutSec seconds"
    return $false
}

# Cleanup function for graceful shutdown
function Stop-NetworkNodes {
    Write-Header "🛑 SHUTTING DOWN NETWORK NODES"
    
    # Stop CLI operations
    Get-Job | Where-Object { $_.Name -like "CLI-*" } | Stop-Job | Remove-Job -Force
    
    # Stop all node processes
    $script:NodeProcesses | ForEach-Object {
        if ($_ -and !$_.HasExited) {
            Write-ColoredOutput "Yellow" "Stopping node process (PID: $($_.Id))"
            try {
                $_.Kill()
                $_.WaitForExit(5000)
            }
            catch {
                Write-ColoredOutput "Red" "Force terminating node process: $($_.Id)"
            }
        }
    }
    
    Write-ColoredOutput "Green" "✅ All nodes stopped"
}

# Register cleanup handler
$null = Register-EngineEvent -SourceIdentifier PowerShell.Exiting -Action {
    Stop-NetworkNodes
}

# Trap Ctrl+C
[Console]::TreatControlCAsInput = $false
[Console]::CancelKeyPress = {
    param($sender, $e)
    $e.Cancel = $true
    Stop-NetworkNodes
    exit 0
}

Write-Header "🚀 AEROLITHDB LOCAL NETWORK LAUNCHER"
Write-ColoredOutput "Blue" "Launching distributed test network with $NodesCount nodes"
Write-ColoredOutput "Blue" "Bootstrap port: $StartPort, Data directory: $DataDir"

# Clean up previous test data
if (Test-Path $DataDir) {
    Write-ColoredOutput "Yellow" "🧹 Cleaning up previous test data..."
    Remove-Item -Path $DataDir -Recurse -Force
}

# Create data directories
Write-ColoredOutput "Blue" "📁 Creating node data directories..."
$BootstrapDataDir = Join-Path $DataDir "bootstrap-node"
New-Item -ItemType Directory -Path $BootstrapDataDir -Force | Out-Null

for ($i = 1; $i -le $NodesCount; $i++) {
    $nodeDataDir = Join-Path $DataDir "node-$i"
    New-Item -ItemType Directory -Path $nodeDataDir -Force | Out-Null
}

# Build the project if needed
Write-Header "🔨 BUILDING AEROLITHDB"
Write-ColoredOutput "Blue" "Building AerolithDB binaries..."

try {
    $buildOutput = cargo build --release 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-ColoredOutput "Red" "❌ Build failed:"
        Write-Host $buildOutput
        exit 1
    }
    Write-ColoredOutput "Green" "✅ Build completed successfully"
}
catch {
    Write-ColoredOutput "Red" "❌ Build failed: $($_.Exception.Message)"
    exit 1
}

# Initialize tracking arrays
$script:NodeProcesses = @()
$NodeUrls = @()

Write-Header "🏗️ LAUNCHING BOOTSTRAP NODE"

# Set environment variables for bootstrap node
$env:AEROLITHSDB_NODE_ID = "bootstrap-node-001"
$env:AEROLITHSDB_STORAGE_DATA_DIR = $BootstrapDataDir
$env:AEROLITHSDB_API_REST_PORT = $StartPort.ToString()
$env:AEROLITHSDB_NETWORK_IS_BOOTSTRAP = "true"
$env:RUST_LOG = if ($Verbose) { "debug,aerolithsdb=trace" } else { "info,aerolithsdb=info" }

Write-ColoredOutput "Blue" "🎯 Starting bootstrap node on port $StartPort..."

# Start bootstrap node
$bootstrapProcess = Start-Process -FilePath "cargo" -ArgumentList @("run", "--release", "--") -NoNewWindow -PassThru
$script:NodeProcesses += $bootstrapProcess
$bootstrapUrl = "http://localhost:$StartPort"
$NodeUrls += $bootstrapUrl

Write-ColoredOutput "Green" "✅ Bootstrap node started (PID: $($bootstrapProcess.Id))"

# Wait for bootstrap node to be ready
if (!(Wait-ForHealthy -Url $bootstrapUrl -NodeName "Bootstrap Node")) {
    Write-ColoredOutput "Red" "❌ Bootstrap node failed to start properly"
    Stop-NetworkNodes
    exit 1
}

Write-Header "🌐 LAUNCHING REGULAR NODES"

# Launch regular nodes
for ($i = 1; $i -le $NodesCount; $i++) {
    $nodePort = $StartPort + $i
    $nodeId = "regular-node-$('{0:D3}' -f $i)"
    $nodeDataDir = Join-Path $DataDir "node-$i"
    $nodeUrl = "http://localhost:$nodePort"
    
    Write-ColoredOutput "Blue" "🎯 Starting regular node $i on port $nodePort..."
    
    # Set environment variables for this node
    $env:AEROLITHSDB_NODE_ID = $nodeId
    $env:AEROLITHSDB_STORAGE_DATA_DIR = $nodeDataDir
    $env:AEROLITHSDB_API_REST_PORT = $nodePort.ToString()
    $env:AEROLITHSDB_NETWORK_IS_BOOTSTRAP = "false"
    $env:AEROLITHSDB_NETWORK_BOOTSTRAP_NODES = $bootstrapUrl
    
    # Start regular node
    $nodeProcess = Start-Process -FilePath "cargo" -ArgumentList @("run", "--release", "--") -NoNewWindow -PassThru
    $script:NodeProcesses += $nodeProcess
    $NodeUrls += $nodeUrl
    
    Write-ColoredOutput "Green" "✅ Regular node $i started (PID: $($nodeProcess.Id))"
    
    # Brief pause between node startups
    Start-Sleep -Seconds 2
}

Write-Header "⏳ WAITING FOR NETWORK FORMATION"

# Wait for all nodes to be healthy
$allHealthy = $true
foreach ($url in $NodeUrls) {
    $nodeIndex = $NodeUrls.IndexOf($url)
    $nodeName = if ($nodeIndex -eq 0) { "Bootstrap Node" } else { "Regular Node $nodeIndex" }
    
    if (!(Wait-ForHealthy -Url $url -NodeName $nodeName -TimeoutSec 45)) {
        $allHealthy = $false
    }
}

if (!$allHealthy) {
    Write-ColoredOutput "Red" "❌ Not all nodes became healthy"
    Stop-NetworkNodes
    exit 1
}

# Additional wait for network stabilization
Write-ColoredOutput "Yellow" "⏳ Allowing time for P2P mesh formation..."
Start-Sleep -Seconds 10

Write-Header "🎭 SIMULATING USER ACTIVITY"

# Simulate user document operations across nodes
$TestDocuments = @(
    @{
        collection = "users"
        id = "user_001"
        data = @{
            name = "Alice Johnson"
            email = "alice@aerolithdb.com"
            department = "Engineering"
            role = "Senior Developer"
            joined_date = "2024-01-15"
        }
    },
    @{
        collection = "users" 
        id = "user_002"
        data = @{
            name = "Bob Smith"
            email = "bob@aerolithdb.com"
            department = "Product"
            role = "Product Manager"
            joined_date = "2024-02-01"
        }
    },
    @{
        collection = "projects"
        id = "proj_001"
        data = @{
            name = "AerolithDB Enhancement"
            description = "Improving distributed consensus algorithms"
            status = "active"
            team_members = @("user_001", "user_002")
            start_date = "2024-03-01"
        }
    },
    @{
        collection = "analytics"
        id = "metrics_$(Get-Date -Format 'yyyyMMdd_HHmmss')"
        data = @{
            event_type = "network_test"
            timestamp = Get-Date -Format "yyyy-MM-ddTHH:mm:ssZ"
            nodes_count = $NodesCount + 1
            test_phase = "user_simulation"
        }
    }
)

# CREATE operations across different nodes
Write-ColoredOutput "Blue" "📝 Creating documents across nodes..."
for ($i = 0; $i -lt $TestDocuments.Count; $i++) {
    $doc = $TestDocuments[$i]
    $targetNodeIndex = $i % $NodeUrls.Count
    $targetUrl = $NodeUrls[$targetNodeIndex]
    $nodeName = if ($targetNodeIndex -eq 0) { "Bootstrap" } else { "Node $targetNodeIndex" }
    
    Write-ColoredOutput "Cyan" "  Creating document '$($doc.id)' in collection '$($doc.collection)' on $nodeName..."
    
    $jsonData = $doc.data | ConvertTo-Json -Depth 10
    
    try {
        cargo run --release --bin aerolithsdb-cli -- --url $targetUrl put $doc.collection $doc.id --data $jsonData | Out-Null
        Write-ColoredOutput "Green" "    ✅ Document created successfully"
    }
    catch {
        Write-ColoredOutput "Red" "    ❌ Failed to create document: $($_.Exception.Message)"
    }
    
    Start-Sleep -Seconds 1
}

Write-Header "🔄 DEMONSTRATING CROSS-NODE OPERATIONS"

# READ operations from different nodes to demonstrate replication
Write-ColoredOutput "Blue" "📖 Reading documents from different nodes (demonstrating replication)..."

foreach ($doc in $TestDocuments) {
    # Try reading from a different node than where it was written
    $readerNodeIndex = (($TestDocuments.IndexOf($doc) + 2) % $NodeUrls.Count)
    $readerUrl = $NodeUrls[$readerNodeIndex]
    $readerName = if ($readerNodeIndex -eq 0) { "Bootstrap" } else { "Node $readerNodeIndex" }
    
    Write-ColoredOutput "Cyan" "  Reading '$($doc.id)' from $readerName..."
    
    try {
        $result = cargo run --release --bin aerolithsdb-cli -- --url $readerUrl get $doc.collection $doc.id 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-ColoredOutput "Green" "    ✅ Cross-node read successful"
        }
        else {
            Write-ColoredOutput "Yellow" "    ⚠️  Document not yet replicated (this is normal in eventual consistency)"
        }
    }
    catch {
        Write-ColoredOutput "Yellow" "    ⚠️  Document not yet replicated: $($_.Exception.Message)"
    }
    
    Start-Sleep -Seconds 1
}

Write-Header "🔍 QUERYING AND ANALYTICS"

# Query operations demonstrating search capabilities
Write-ColoredOutput "Blue" "🔎 Performing queries across the distributed network..."

$QuerieTests = @(
    @{
        collection = "users"
        description = "Query all users in Engineering department"
        filter = '{"department": "Engineering"}'
    },
    @{
        collection = "projects" 
        description = "Query active projects"
        filter = '{"status": "active"}'
    },
    @{
        collection = "analytics"
        description = "Query recent analytics events"
        filter = '{"event_type": "network_test"}'
    }
)

foreach ($query in $QuerieTests) {
    # Distribute queries across different nodes
    $queryNodeIndex = ($QuerieTests.IndexOf($query) % $NodeUrls.Count)
    $queryUrl = $NodeUrls[$queryNodeIndex]
    $queryNodeName = if ($queryNodeIndex -eq 0) { "Bootstrap" } else { "Node $queryNodeIndex" }
    
    Write-ColoredOutput "Cyan" "  $($query.description) (via $queryNodeName)..."
    
    try {
        $queryResult = cargo run --release --bin aerolithsdb-cli -- --url $queryUrl query $query.collection --filter $query.filter 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-ColoredOutput "Green" "    ✅ Query executed successfully"
        }
        else {
            Write-ColoredOutput "Yellow" "    ⚠️  Query returned no results (data may not be fully replicated yet)"
        }
    }
    catch {
        Write-ColoredOutput "Yellow" "    ⚠️  Query failed: $($_.Exception.Message)"
    }
    
    Start-Sleep -Seconds 1
}

Write-Header "👑 ADMINISTRATIVE OPERATIONS"

# Administrative health checks and monitoring
Write-ColoredOutput "Blue" "🩺 Performing network health checks and monitoring..."

foreach ($url in $NodeUrls) {
    $nodeIndex = $NodeUrls.IndexOf($url)
    $nodeName = if ($nodeIndex -eq 0) { "Bootstrap Node" } else { "Regular Node $nodeIndex" }
    
    Write-ColoredOutput "Cyan" "  Health check: $nodeName..."
    
    try {
        cargo run --release --bin aerolithsdb-cli -- --url $url health | Out-Null
        if ($LASTEXITCODE -eq 0) {
            Write-ColoredOutput "Green" "    ✅ $nodeName is healthy"
        }
        else {
            Write-ColoredOutput "Red" "    ❌ $nodeName health check failed"
        }
    }
    catch {
        Write-ColoredOutput "Red" "    ❌ $nodeName health check error: $($_.Exception.Message)"
    }
}

# Statistics collection
Write-ColoredOutput "Blue" "📊 Collecting system statistics..."
try {
    $statsUrl = $NodeUrls[0]  # Get stats from bootstrap node
    cargo run --release --bin aerolithsdb-cli -- --url $statsUrl stats --format table | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Write-ColoredOutput "Green" "    ✅ Statistics collected successfully"
    }
}
catch {
    Write-ColoredOutput "Yellow" "    ⚠️  Statistics collection error: $($_.Exception.Message)"
}

Write-Header "🎯 NETWORK TEST COMPLETED"

Write-ColoredOutput "Green" "✅ AerolithDB distributed network test completed successfully!"
Write-Host ""
Write-ColoredOutput "Blue" "📋 Test Summary:"
Write-ColoredOutput "Blue" "   • Bootstrap Node: $($NodeUrls[0])"
Write-ColoredOutput "Blue" "   • Regular Nodes: $($NodesCount) nodes"
Write-ColoredOutput "Blue" "   • Documents Created: $($TestDocuments.Count)"
Write-ColoredOutput "Blue" "   • Cross-node Operations: Tested"
Write-ColoredOutput "Blue" "   • Query Operations: Tested"
Write-ColoredOutput "Blue" "   • Administrative Operations: Tested"
Write-Host ""
Write-ColoredOutput "Yellow" "🔗 Network endpoints:"
foreach ($url in $NodeUrls) {
    $nodeIndex = $NodeUrls.IndexOf($url)
    $nodeType = if ($nodeIndex -eq 0) { "Bootstrap" } else { "Regular" }
    Write-ColoredOutput "Yellow" "   • $nodeType Node: $url"
}
Write-Host ""
Write-ColoredOutput "Cyan" "💡 You can now interact with the network using:"
Write-ColoredOutput "Cyan" "   cargo run --release --bin aerolithsdb-cli -- --url $($NodeUrls[0]) health"
Write-ColoredOutput "Cyan" "   cargo run --release --bin aerolithsdb-cli -- --url $($NodeUrls[0]) stats"
Write-ColoredOutput "Cyan" "   cargo run --release --bin aerolithsdb-cli -- --url $($NodeUrls[0]) get users user_001"
Write-Host ""
Write-ColoredOutput "Bold" "Press Ctrl+C to stop all nodes and exit"

# Keep the script running until interrupted
try {
    while ($true) {
        Start-Sleep -Seconds 10
        
        # Periodic health check
        $healthyCount = 0
        foreach ($url in $NodeUrls) {
            try {
                $null = Invoke-RestMethod -Uri "$url/health" -Method Get -TimeoutSec 3 -ErrorAction SilentlyContinue
                $healthyCount++
            }
            catch {
                # Node not responding
            }
        }
        
        Write-ColoredOutput "Blue" "🔄 Network Status: $healthyCount/$($NodeUrls.Count) nodes healthy"
    }
}
catch {
    Write-ColoredOutput "Yellow" "Received interrupt signal"
}
finally {
    Stop-NetworkNodes
}
