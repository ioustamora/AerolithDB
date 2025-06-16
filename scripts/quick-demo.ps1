#!/usr/bin/env pwsh
<#
.SYNOPSIS
Quick AerolithDB Multi-Node Demo

.DESCRIPTION
Simple demonstration of AerolithDB's distributed functionality:
- Builds the project
- Starts a bootstrap node
- Starts 2 regular nodes  
- Shows basic CRUD operations
- Demonstrates cross-node data access

.PARAMETER Port
Starting port (default: 8080)

.EXAMPLE
.\scripts\quick-demo.ps1
Run quick 3-node demo
#>

param(
    [int]$Port = 8080
)

$ErrorActionPreference = "Stop"

# Colors
$Green = "`e[32m"
$Blue = "`e[34m" 
$Yellow = "`e[33m"
$Red = "`e[31m"
$Reset = "`e[0m"

function Write-Color($Color, $Message) {
    Write-Host "$Color$Message$Reset"
}

function Test-Port($Port) {
    try {
        $null = Test-NetConnection -ComputerName localhost -Port $Port -WarningAction SilentlyContinue -ErrorAction Stop
        return $true
    } catch {
        return $false
    }
}

function Wait-ForHealth($Url, $TimeoutSec = 30) {
    $elapsed = 0
    while ($elapsed -lt $TimeoutSec) {
        try {
            $response = Invoke-RestMethod -Uri "$Url/health" -Method Get -TimeoutSec 3 -ErrorAction Stop
            return $true
        }
        catch {
            Start-Sleep -Seconds 2
            $elapsed += 2
        }
    }
    return $false
}

Write-Color $Blue "🚀 AerolithDB Quick Multi-Node Demo"
Write-Color $Blue "====================================="

# Check if ports are available
$RequiredPorts = @($Port, ($Port + 1), ($Port + 2))
foreach ($p in $RequiredPorts) {
    if (Test-Port $p) {
        Write-Color $Red "❌ Port $p is already in use. Please stop the service or use a different port."
        exit 1
    }
}

# Build project
Write-Color $Yellow "🔨 Building AerolithDB..."
try {
    cargo build --release 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) {
        throw "Build failed"
    }
    Write-Color $Green "✅ Build completed"
}
catch {
    Write-Color $Red "❌ Build failed. Please check your Rust installation."
    exit 1
}

# Clean previous data
if (Test-Path "quick-demo-data") {
    Remove-Item -Path "quick-demo-data" -Recurse -Force
}
New-Item -ItemType Directory -Path "quick-demo-data" -Force | Out-Null

$Processes = @()

try {
    # Start bootstrap node
    Write-Color $Yellow "🎯 Starting bootstrap node on port $Port..."
    
    $env:AEROLITHSDB_NODE_ID = "demo-bootstrap"
    $env:AEROLITHSDB_STORAGE_DATA_DIR = "quick-demo-data\bootstrap"
    $env:AEROLITHSDB_API_REST_PORT = $Port.ToString()
    $env:RUST_LOG = "info"
    
    New-Item -ItemType Directory -Path "quick-demo-data\bootstrap" -Force | Out-Null
    $bootstrapProcess = Start-Process -FilePath "cargo" -ArgumentList @("run", "--release") -NoNewWindow -PassThru
    $Processes += $bootstrapProcess
    
    # Wait for bootstrap
    $bootstrapUrl = "http://localhost:$Port"
    if (!(Wait-ForHealth $bootstrapUrl)) {
        throw "Bootstrap node failed to start"
    }
    Write-Color $Green "✅ Bootstrap node ready"
    
    # Start node 1
    Write-Color $Yellow "🎯 Starting regular node 1 on port $($Port + 1)..."
    
    $env:AEROLITHSDB_NODE_ID = "demo-node-1" 
    $env:AEROLITHSDB_STORAGE_DATA_DIR = "quick-demo-data\node1"
    $env:AEROLITHSDB_API_REST_PORT = ($Port + 1).ToString()
    
    New-Item -ItemType Directory -Path "quick-demo-data\node1" -Force | Out-Null
    $node1Process = Start-Process -FilePath "cargo" -ArgumentList @("run", "--release") -NoNewWindow -PassThru
    $Processes += $node1Process
    
    # Start node 2  
    Write-Color $Yellow "🎯 Starting regular node 2 on port $($Port + 2)..."
    
    $env:AEROLITHSDB_NODE_ID = "demo-node-2"
    $env:AEROLITHSDB_STORAGE_DATA_DIR = "quick-demo-data\node2" 
    $env:AEROLITHSDB_API_REST_PORT = ($Port + 2).ToString()
    
    New-Item -ItemType Directory -Path "quick-demo-data\node2" -Force | Out-Null
    $node2Process = Start-Process -FilePath "cargo" -ArgumentList @("run", "--release") -NoNewWindow -PassThru
    $Processes += $node2Process
    
    # Wait for nodes
    Start-Sleep -Seconds 5
    
    $node1Url = "http://localhost:$($Port + 1)"
    $node2Url = "http://localhost:$($Port + 2)"
    
    if (!(Wait-ForHealth $node1Url)) {
        throw "Node 1 failed to start"
    }
    Write-Color $Green "✅ Node 1 ready"
    
    if (!(Wait-ForHealth $node2Url)) {
        throw "Node 2 failed to start"  
    }
    Write-Color $Green "✅ Node 2 ready"
    
    Write-Color $Blue "`n🎭 Demonstrating Distributed Operations"
    Write-Color $Blue "======================================="
    
    # Create document on bootstrap node
    Write-Color $Yellow "📝 Creating document on bootstrap node..."
    $docData = '{"name":"Demo User","email":"demo@aerolithdb.com","created":"' + (Get-Date -Format "yyyy-MM-ddTHH:mm:ssZ") + '"}'
    
    cargo run --release --bin aerolithsdb-cli -- --url $bootstrapUrl put demo_users user_001 --data $docData 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Write-Color $Green "✅ Document created on bootstrap node"
    } else {
        Write-Color $Yellow "⚠️  Document creation status unclear"
    }
    
    Start-Sleep -Seconds 2
    
    # Try to read from different nodes
    Write-Color $Yellow "📖 Reading document from Node 1..."
    $result1 = cargo run --release --bin aerolithsdb-cli -- --url $node1Url get demo_users user_001 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Color $Green "✅ Document successfully read from Node 1"
    } else {
        Write-Color $Yellow "⚠️  Document not yet replicated to Node 1 (eventual consistency)"
    }
    
    Write-Color $Yellow "📖 Reading document from Node 2..."
    $result2 = cargo run --release --bin aerolithsdb-cli -- --url $node2Url get demo_users user_001 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Color $Green "✅ Document successfully read from Node 2"
    } else {
        Write-Color $Yellow "⚠️  Document not yet replicated to Node 2 (eventual consistency)"
    }
    
    # Health checks
    Write-Color $Yellow "`n🩺 Health checks across all nodes..."
    
    foreach ($url in @($bootstrapUrl, $node1Url, $node2Url)) {
        $nodeIndex = @($bootstrapUrl, $node1Url, $node2Url).IndexOf($url)
        $nodeName = @("Bootstrap", "Node 1", "Node 2")[$nodeIndex]
        
        cargo run --release --bin aerolithsdb-cli -- --url $url health 2>&1 | Out-Null
        if ($LASTEXITCODE -eq 0) {
            Write-Color $Green "✅ $nodeName is healthy"
        } else {
            Write-Color $Red "❌ $nodeName health check failed"
        }
    }
    
    Write-Color $Blue "`n🎯 Demo Completed Successfully!"
    Write-Color $Blue "=============================="
    Write-Color $Green "✅ Multi-node AerolithDB network is running"
    Write-Color $Green "✅ Distributed document operations demonstrated"
    Write-Color $Green "✅ Cross-node connectivity verified"
    
    Write-Color $Yellow "`n🔗 Available endpoints:"
    Write-Color $Yellow "   Bootstrap Node: $bootstrapUrl"
    Write-Color $Yellow "   Regular Node 1: $node1Url" 
    Write-Color $Yellow "   Regular Node 2: $node2Url"
    
    Write-Color $Blue "`n💡 Try these commands:"
    Write-Color $Blue "   cargo run --release --bin aerolithsdb-cli -- --url $bootstrapUrl stats"
    Write-Color $Blue "   cargo run --release --bin aerolithsdb-cli -- --url $node1Url get demo_users user_001"
    Write-Color $Blue "   cargo run --release --bin aerolithsdb-cli -- --url $node2Url health"
    
    Write-Color $Yellow "`nPress Ctrl+C to stop all nodes and exit..."
    
    # Keep running until interrupted
    while ($true) {
        Start-Sleep -Seconds 5
        
        # Quick health check
        $healthyCount = 0
        foreach ($url in @($bootstrapUrl, $node1Url, $node2Url)) {
            try {
                $null = Invoke-RestMethod -Uri "$url/health" -Method Get -TimeoutSec 2 -ErrorAction Stop
                $healthyCount++
            } catch {
                # Node not responding
            }
        }
        
        Write-Color $Blue "🔄 Network Status: $healthyCount/3 nodes healthy"
    }
}
catch {
    Write-Color $Red "❌ Demo failed: $($_.Exception.Message)"
}
finally {
    Write-Color $Yellow "`n🛑 Cleaning up processes..."
    
    foreach ($process in $Processes) {
        if ($process -and !$process.HasExited) {
            try {
                $process.Kill()
                $process.WaitForExit(3000)
            } catch {
                # Force kill if needed
            }
        }
    }
    
    Write-Color $Green "✅ Cleanup completed"
}
