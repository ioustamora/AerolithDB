# AerolithDB Local Network Launcher (Fixed for PowerShell)
# A comprehensive script to launch a local distributed AerolithDB network
# Includes bootstrap node, regular nodes, and test scenarios

param(
    [int]$NodeCount = 4,      # Total nodes (including bootstrap)
    [int]$StartPort = 8080,   # Starting port for nodes
    [int]$HealthCheckTimeout = 30,  # Health check timeout in seconds
    [string]$LogLevel = "info",     # Log level for nodes
    [switch]$NoCleanup,       # Skip cleanup at the end
    [switch]$QuietMode        # Reduce output verbosity
)

# Global variables
$Global:NodeProcesses = @()
$Global:NodeUrls = @()
$Global:NodeDataDirs = @()

# Color output functions
function Write-ColoredOutput {
    param([string]$Color, [string]$Message)
    if (-not $QuietMode) {
        Write-Host $Message -ForegroundColor $Color
    }
}

function Write-Header {
    param([string]$Title)
    if (-not $QuietMode) {
        Write-Host ""
        Write-Host "=" * 60 -ForegroundColor Cyan
        Write-Host "  $Title" -ForegroundColor White
        Write-Host "=" * 60 -ForegroundColor Cyan
        Write-Host ""
    }
}

function Write-SubHeader {
    param([string]$Title)
    if (-not $QuietMode) {
        Write-Host ""
        Write-Host "-" * 40 -ForegroundColor Yellow
        Write-Host "  $Title" -ForegroundColor Yellow
        Write-Host "-" * 40 -ForegroundColor Yellow
    }
}

# Health check function
function Wait-ForNodeHealth {
    param(
        [string]$NodeUrl,
        [string]$NodeName,
        [int]$TimeoutSec = 30
    )
    
    Write-ColoredOutput "Yellow" "Waiting for $NodeName to become healthy..."
    $startTime = Get-Date
    
    do {
        try {
            $response = Invoke-RestMethod -Uri "$NodeUrl/health" -Method Get -TimeoutSec 5 -ErrorAction SilentlyContinue
            if ($response -and $response.status -eq "healthy") {
                Write-ColoredOutput "Green" "$NodeName is healthy"
                return $true
            }
        }
        catch {
            # Health check failed, continue waiting
        }
        
        Start-Sleep -Seconds 2
        $elapsed = (Get-Date) - $startTime
    } while ($elapsed.TotalSeconds -lt $TimeoutSec)
    
    Write-ColoredOutput "Red" "$NodeName failed to become healthy within $TimeoutSec seconds"
    return $false
}

# Cleanup function
function Stop-AllNodes {
    Write-Header "SHUTTING DOWN NETWORK NODES"
    
    foreach ($process in $Global:NodeProcesses) {
        if (-not $process.HasExited) {
            try {
                Write-ColoredOutput "Yellow" "Stopping node (PID: $($process.Id))..."
                $process.Kill()
                $process.WaitForExit(5000)
            }
            catch {
                Write-ColoredOutput "Red" "Failed to stop process $($process.Id): $($_.Exception.Message)"
            }
        }
    }
    
    # Clean up data directories if not preserving
    if (-not $NoCleanup) {
        Write-ColoredOutput "Yellow" "Cleaning up node data directories..."
        foreach ($dir in $Global:NodeDataDirs) {
            if (Test-Path $dir) {
                Remove-Item -Path $dir -Recurse -Force -ErrorAction SilentlyContinue
            }
        }
    }
    
    Write-ColoredOutput "Green" "All nodes stopped"
}

# Error handling
trap {
    Write-ColoredOutput "Red" "Script error: $($_.Exception.Message)"
    Stop-AllNodes
    exit 1
}

# Register cleanup on exit
Register-EngineEvent PowerShell.Exiting -Action { Stop-AllNodes }

# Main execution
try {
    Write-Header "AEROLITHDB LOCAL NETWORK LAUNCHER"
    Write-ColoredOutput "Cyan" "Starting $NodeCount nodes on ports $StartPort-$(($StartPort + $NodeCount - 1))"
    Write-Host ""
    
    # Cleanup previous test data
    if (-not $NoCleanup) {
        Write-ColoredOutput "Yellow" "Cleaning up previous test data..."
        Remove-Item -Path "test_data" -Recurse -Force -ErrorAction SilentlyContinue
        Remove-Item -Path "node_*" -Recurse -Force -ErrorAction SilentlyContinue
    }
    
    Write-ColoredOutput "Blue" "Creating node data directories..."
    for ($i = 0; $i -lt $NodeCount; $i++) {
        $dataDir = "node_$i"
        New-Item -ItemType Directory -Path $dataDir -Force | Out-Null
        $Global:NodeDataDirs += $dataDir
    }
    
    # Build the project
    Write-Header "BUILDING AEROLITHDB"
    Write-ColoredOutput "Blue" "Building project with optimizations..."
    $buildResult = cargo build --release 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-ColoredOutput "Red" "Build failed:"
        Write-Host $buildResult
        exit 1
    } else {
        Write-ColoredOutput "Green" "Build completed successfully"
    }
}
catch {
    Write-ColoredOutput "Red" "Build failed: $($_.Exception.Message)"
    exit 1
}

# Launch bootstrap node
try {
    Write-Header "LAUNCHING BOOTSTRAP NODE"
    
    $bootstrapPort = $StartPort
    $bootstrapUrl = "http://localhost:$bootstrapPort"
    $Global:NodeUrls += $bootstrapUrl
    
    Write-ColoredOutput "Blue" "Starting bootstrap node on port $StartPort..."
      $bootstrapProcess = Start-Process -FilePath "cargo" -ArgumentList @(
        "run", "--release", "--bin", "aerolithdb", "--", 
        "--port", $bootstrapPort,
        "--data-dir", "node_0",
        "--log-level", $LogLevel,
        "--node-type", "bootstrap"
    ) -NoNewWindow -PassThru
    
    $Global:NodeProcesses += $bootstrapProcess
    Write-ColoredOutput "Green" "Bootstrap node started (PID: $($bootstrapProcess.Id))"
    
    # Wait for bootstrap node to be healthy
    if (-not (Wait-ForNodeHealth -NodeUrl $bootstrapUrl -NodeName "Bootstrap node" -TimeoutSec $HealthCheckTimeout)) {
        Write-ColoredOutput "Red" "Bootstrap node failed to start properly"
        Stop-AllNodes
        exit 1
    }
    
    Write-Header "LAUNCHING REGULAR NODES"
    
    # Launch regular nodes
    for ($i = 1; $i -lt $NodeCount; $i++) {
        $nodePort = $StartPort + $i
        $nodeUrl = "http://localhost:$nodePort"
        $Global:NodeUrls += $nodeUrl
        
        Write-ColoredOutput "Blue" "Starting regular node $i on port $nodePort..."
          $nodeProcess = Start-Process -FilePath "cargo" -ArgumentList @(
            "run", "--release", "--bin", "aerolithdb", "--",
            "--port", $nodePort,
            "--data-dir", "node_$i",
            "--log-level", $LogLevel,
            "--bootstrap-nodes", $bootstrapUrl,
            "--node-type", "regular"
        ) -NoNewWindow -PassThru
        
        $Global:NodeProcesses += $nodeProcess
        Write-ColoredOutput "Green" "Regular node $i started (PID: $($nodeProcess.Id))"
        
        # Brief pause between node starts
        Start-Sleep -Seconds 2
    }
    
    Write-Header "WAITING FOR NETWORK FORMATION"
    
    # Wait for all nodes to become healthy
    $allHealthy = $true
    foreach ($url in $Global:NodeUrls) {
        $nodeIndex = $Global:NodeUrls.IndexOf($url)
        $nodeName = if ($nodeIndex -eq 0) { "Bootstrap node" } else { "Regular node $nodeIndex" }
        
        if (-not (Wait-ForNodeHealth -NodeUrl $url -NodeName $nodeName -TimeoutSec $HealthCheckTimeout)) {
            $allHealthy = $false
        }
    }
    
    if (-not $allHealthy) {
        Write-ColoredOutput "Red" "Some nodes failed to become healthy"
        Stop-AllNodes
        exit 1
    }
    
    # Network status
    Write-Header "NETWORK STATUS"
    Write-ColoredOutput "Green" "All $NodeCount nodes are running and healthy!"
    Write-Host ""
    Write-ColoredOutput "Cyan" "Node URLs:"
    foreach ($url in $Global:NodeUrls) {
        $nodeIndex = $Global:NodeUrls.IndexOf($url)
        $nodeType = if ($nodeIndex -eq 0) { "Bootstrap" } else { "Regular" }
        Write-ColoredOutput "Yellow" "   • $nodeType Node: $url"
    }
    Write-Host ""
    Write-ColoredOutput "Cyan" "You can now interact with the network using:"
    Write-ColoredOutput "Cyan" "   cargo run --release --bin aerolithdb-cli -- --url $($Global:NodeUrls[0]) health"
    Write-ColoredOutput "Cyan" "   cargo run --release --bin aerolithdb-cli -- --url $($Global:NodeUrls[0]) stats"
    Write-ColoredOutput "Cyan" "   cargo run --release --bin aerolithdb-cli -- --url $($Global:NodeUrls[0]) get users user_001"
    Write-Host ""
    
    # Interactive mode
    Write-ColoredOutput "Magenta" "Press Enter to run demo operations, or Ctrl+C to stop the network..."
    Read-Host
    
    # Demo operations
    Write-Header "RUNNING DEMO OPERATIONS"
    
    $primaryNode = $Global:NodeUrls[0]
    
    Write-SubHeader "Testing Document Operations"
    
    # Create some test documents
    Write-ColoredOutput "Blue" "Creating test documents..."
    
    # Test CRUD operations via CLI
    $documents = @(
        @{ collection = "users"; key = "user_001"; data = '{"name": "Alice Johnson", "email": "alice@example.com", "role": "admin"}' },
        @{ collection = "users"; key = "user_002"; data = '{"name": "Bob Smith", "email": "bob@example.com", "role": "user"}' },
        @{ collection = "products"; key = "prod_001"; data = '{"name": "AerolithDB License", "price": 99.99, "category": "software"}' },
        @{ collection = "orders"; key = "order_001"; data = '{"user_id": "user_001", "product_id": "prod_001", "quantity": 1, "timestamp": "2024-01-15T10:30:00Z"}' }
    )
    
    foreach ($doc in $documents) {
        Write-ColoredOutput "Yellow" "Creating $($doc.collection)/$($doc.key)..."
        $result = cargo run --release --bin aerolithdb-cli -- --url $primaryNode set $($doc.collection) $($doc.key) $($doc.data) 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-ColoredOutput "Green" "  Created successfully"
        } else {
            Write-ColoredOutput "Red" "  Failed: $result"
        }
    }
    
    Write-SubHeader "Testing Cross-Node Replication"
    
    # Test reading from different nodes to verify replication
    Write-ColoredOutput "Blue" "Testing data consistency across nodes..."
    foreach ($nodeUrl in $Global:NodeUrls) {
        $nodeIndex = $Global:NodeUrls.IndexOf($nodeUrl)
        Write-ColoredOutput "Yellow" "Reading from node $nodeIndex ($nodeUrl):"
        
        $result = cargo run --release --bin aerolithdb-cli -- --url $nodeUrl get users user_001 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-ColoredOutput "Green" "  Data available on node $nodeIndex"
        } else {
            Write-ColoredOutput "Red" "  Data not available on node ${nodeIndex}: $result"
        }
    }
    
    Write-SubHeader "Testing Analytics and Stats"
    
    # Get network statistics
    Write-ColoredOutput "Blue" "Fetching network statistics..."
    foreach ($nodeUrl in $Global:NodeUrls) {
        $nodeIndex = $Global:NodeUrls.IndexOf($nodeUrl)
        Write-ColoredOutput "Yellow" "Stats from node ${nodeIndex}:"
        
        $result = cargo run --release --bin aerolithdb-cli -- --url $nodeUrl stats 2>&1
        Write-ColoredOutput "Cyan" "  $result"
    }
    
    Write-SubHeader "Testing Advanced Query Operations"
    
    # Test batch operations
    Write-ColoredOutput "Blue" "Testing batch operations..."
    $batchFile = "demo_batch.json"
    $batchContent = @"
{
    "operations": [
        {
            "type": "set",
            "collection": "metrics",
            "key": "cpu_usage",
            "data": {"timestamp": "2024-01-15T10:35:00Z", "value": 75.2, "host": "node_001"}
        },
        {
            "type": "set",
            "collection": "metrics",
            "key": "memory_usage",
            "data": {"timestamp": "2024-01-15T10:35:00Z", "value": 68.5, "host": "node_001"}
        },
        {
            "type": "get",
            "collection": "users",
            "key": "user_001"
        }
    ]
}
"@
    
    $batchContent | Out-File -FilePath $batchFile -Encoding UTF8
    
    Write-ColoredOutput "Yellow" "Executing batch operations..."
    $result = cargo run --release --bin aerolithdb-cli -- --url $primaryNode batch $batchFile 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-ColoredOutput "Green" "Batch operations completed successfully"
        Write-ColoredOutput "Cyan" "Result: $result"
    } else {
        Write-ColoredOutput "Red" "Batch operations failed: $result"
    }
    
    # Cleanup batch file
    Remove-Item -Path $batchFile -ErrorAction SilentlyContinue
    
    Write-Header "DEMONSTRATION COMPLETE"
    Write-ColoredOutput "Green" "Local AerolithDB network demonstration completed successfully!"
    Write-ColoredOutput "Cyan" "Network is still running. You can continue testing manually or press Ctrl+C to stop."
    Write-Host ""
    Write-ColoredOutput "Yellow" "Manual testing commands:"
    Write-ColoredOutput "Yellow" "  cargo run --release --bin aerolithdb-cli -- --url $primaryNode health"
    Write-ColoredOutput "Yellow" "  cargo run --release --bin aerolithdb-cli -- --url $primaryNode get users user_001"
    Write-ColoredOutput "Yellow" "  cargo run --release --bin aerolithdb-cli -- --url $primaryNode set test key_001 '{\"test\": \"value\"}'"
    Write-Host ""
    
    # Keep network running
    Write-ColoredOutput "Magenta" "Press Ctrl+C to stop the network..."
    while ($true) {
        Start-Sleep -Seconds 5
        
        # Quick health check
        $healthyCount = 0
        foreach ($url in $Global:NodeUrls) {
            try {
                $response = Invoke-RestMethod -Uri "$url/health" -Method Get -TimeoutSec 2 -ErrorAction SilentlyContinue
                if ($response -and $response.status -eq "healthy") {
                    $healthyCount++
                }
            }
            catch {
                # Ignore health check failures in monitoring loop
            }
        }
        
        if ($healthyCount -lt $Global:NodeUrls.Count) {
            Write-ColoredOutput "Yellow" "Warning: Only $healthyCount/$($Global:NodeUrls.Count) nodes are healthy"
        }
    }
}
catch {
    Write-ColoredOutput "Red" "Error during node startup: $($_.Exception.Message)"
    Stop-AllNodes
    exit 1
}
finally {
    if (-not $NoCleanup) {
        Stop-AllNodes
    }
}
