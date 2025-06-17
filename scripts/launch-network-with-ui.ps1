#!/usr/bin/env pwsh
<#
.SYNOPSIS
Launch AerolithDB Full-Stack Development Environment with Network and Web UI

.DESCRIPTION
This script launches a complete AerolithDB development environment including:
- Distributed backend network (1 bootstrap + 4 regular nodes)
- Modern React web client with real-time monitoring
- Full-stack integration for testing and development
- Automatic health monitoring and status reporting

.PARAMETER NodesCount
Number of regular nodes to launch (default: 4)

.PARAMETER StartPort
Starting port for node allocation (default: 8080)

.PARAMETER WebPort
Port for the web client (default: 3000)

.PARAMETER SkipBuild
Skip building the Rust binaries (default: false)

.PARAMETER OpenBrowser
Automatically open browser to web client (default: true)

.PARAMETER Verbose
Enable verbose logging output

.EXAMPLE
.\launch-network-with-ui.ps1
Launch default environment (4 nodes + web UI)

.EXAMPLE
.\launch-network-with-ui.ps1 -NodesCount 6 -WebPort 3001 -Verbose
Launch 6-node network with web UI on port 3001

.NOTES
Requires:
- Rust toolchain (cargo)
- Node.js 18+ and npm 9+
- PowerShell 5.1+ or PowerShell Core 6+
#>

param(
    [int]$NodesCount = 4,
    [int]$StartPort = 8080,
    [int]$WebPort = 3000,
    [switch]$SkipBuild,
    [switch]$OpenBrowser,
    [switch]$Verbose
)

# Global state tracking
$script:NodeProcesses = @()
$script:WebProcess = $null
$script:NodeUrls = @()
$Global:DataDir = "fullstack-demo-data"

# Enhanced color output functions
function Write-ColoredOutput {
    param([string]$Color, [string]$Message)
    switch ($Color) {
        "Red" { Write-Host $Message -ForegroundColor Red }
        "Green" { Write-Host $Message -ForegroundColor Green }
        "Yellow" { Write-Host $Message -ForegroundColor Yellow }
        "Blue" { Write-Host $Message -ForegroundColor Blue }
        "Cyan" { Write-Host $Message -ForegroundColor Cyan }
        "Magenta" { Write-Host $Message -ForegroundColor Magenta }
        "White" { Write-Host $Message -ForegroundColor White }
        "Bold" { Write-Host $Message -ForegroundColor White }
        default { Write-Host $Message }
    }
}

function Write-Header {
    param([string]$Title)
    Write-Host ""
    Write-ColoredOutput "Bold" "=" * 80
    Write-ColoredOutput "Cyan" "  $Title"
    Write-ColoredOutput "Bold" "=" * 80
    Write-Host ""
}

function Write-SubHeader {
    param([string]$Title)
    Write-Host ""
    Write-ColoredOutput "Yellow" ">>> $Title"
    Write-Host ""
}

function Wait-ForHealthy {
    param([string]$Url, [string]$NodeName, [int]$TimeoutSec = 30)
    
    Write-ColoredOutput "Yellow" "⏳ Waiting for $NodeName to become healthy..."
    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    
    do {
        try {
            $response = Invoke-RestMethod -Uri "$Url/health" -Method Get -TimeoutSec 3 -ErrorAction SilentlyContinue
            if ($response -and $response.status -eq "healthy") {
                Write-ColoredOutput "Green" "✅ $NodeName is healthy"
                return $true
            }
        }
        catch {
            # Continue waiting
        }
        
        Start-Sleep -Seconds 2
    } while ($stopwatch.Elapsed.TotalSeconds -lt $TimeoutSec)
    
    Write-ColoredOutput "Red" "❌ $NodeName failed to become healthy within $TimeoutSec seconds"
    return $false
}

function Wait-ForWebUI {
    param([string]$Url, [int]$TimeoutSec = 45)
    
    Write-ColoredOutput "Yellow" "⏳ Waiting for Web UI to become available..."
    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    
    do {
        try {
            $response = Invoke-WebRequest -Uri $Url -Method Head -TimeoutSec 3 -ErrorAction SilentlyContinue
            if ($response.StatusCode -eq 200) {
                Write-ColoredOutput "Green" "✅ Web UI is available"
                return $true
            }
        }
        catch {
            # Continue waiting
        }
        
        Start-Sleep -Seconds 2
    } while ($stopwatch.Elapsed.TotalSeconds -lt $TimeoutSec)
    
    Write-ColoredOutput "Red" "❌ Web UI failed to become available within $TimeoutSec seconds"
    return $false
}

function Stop-FullStackEnvironment {
    Write-ColoredOutput "Yellow" "🛑 Stopping full-stack environment..."
    
    # Stop web UI
    if ($script:WebProcess -and !$script:WebProcess.HasExited) {
        Write-ColoredOutput "Yellow" "Stopping Web UI..."
        try {
            $script:WebProcess.Kill()
            $script:WebProcess.WaitForExit(5000)
        }
        catch {
            Write-ColoredOutput "Red" "Failed to stop Web UI gracefully"
        }
    }
    
    # Stop all node processes
    Write-ColoredOutput "Yellow" "Stopping all database nodes..."
    foreach ($process in $script:NodeProcesses) {
        if ($process -and !$process.HasExited) {
            try {
                $process.Kill()
                $process.WaitForExit(3000)
            }
            catch {
                Write-ColoredOutput "Red" "Failed to stop node process $($process.Id)"
            }
        }
    }
    
    # Cleanup data directories
    if (Test-Path $Global:DataDir) {
        Write-ColoredOutput "Yellow" "Cleaning up demo data..."
        try {
            Remove-Item -Path $Global:DataDir -Recurse -Force -ErrorAction SilentlyContinue
        }
        catch {
            Write-ColoredOutput "Yellow" "Could not clean up all demo data (files may be in use)"
        }
    }
    
    Write-ColoredOutput "Green" "✅ Full-stack environment stopped"
}

# Setup cleanup on exit
trap {
    Stop-FullStackEnvironment
    exit 1
}

# Register cleanup on Ctrl+C
[Console]::TreatControlCAsInput = $false
[Console]::CancelKeyPress = {
    param($s, $e)
    $e.Cancel = $true
    Stop-FullStackEnvironment
    exit 0
}

# Setup default for OpenBrowser
if (-not $PSBoundParameters.ContainsKey('OpenBrowser')) {
    $OpenBrowser = $true
}
Write-Header "🚀 AEROLITHDB FULL-STACK ENVIRONMENT LAUNCHER"
Write-ColoredOutput "Blue" "Launching complete development environment:"
Write-ColoredOutput "Blue" "  • Database Network: 1 bootstrap + $NodesCount regular nodes"
Write-ColoredOutput "Blue" "  • Web Client: React application on port $WebPort"
Write-ColoredOutput "Blue" "  • API Endpoints: http://localhost:$StartPort through http://localhost:$($StartPort + $NodesCount)"
Write-Host ""

# Validate prerequisites
Write-SubHeader "Validating Prerequisites"

# Check Rust/Cargo
try {
    $cargoVersion = cargo version 2>$null
    if ($LASTEXITCODE -eq 0) {
        Write-ColoredOutput "Green" "✅ Rust/Cargo: $cargoVersion"
    } else {
        throw "Cargo not found"
    }
}
catch {
    Write-ColoredOutput "Red" "❌ Rust/Cargo is required but not found"
    Write-ColoredOutput "Red" "   Please install from: https://rustup.rs/"
    exit 1
}

# Check Node.js
try {
    $nodeVersion = node --version 2>$null
    if ($LASTEXITCODE -eq 0) {
        Write-ColoredOutput "Green" "✅ Node.js: $nodeVersion"
    } else {
        throw "Node.js not found"
    }
}
catch {
    Write-ColoredOutput "Red" "❌ Node.js 18+ is required but not found"
    Write-ColoredOutput "Red" "   Please install from: https://nodejs.org/"
    exit 1
}

# Check npm
try {
    $npmVersion = npm --version 2>$null
    if ($LASTEXITCODE -eq 0) {
        Write-ColoredOutput "Green" "✅ npm: v$npmVersion"
    } else {
        throw "npm not found"
    }
}
catch {
    Write-ColoredOutput "Red" "❌ npm is required but not found"
    exit 1
}

# Clean previous data
if (Test-Path $Global:DataDir) {
    Write-ColoredOutput "Yellow" "🧹 Cleaning up previous demo data..."
    Remove-Item -Path $Global:DataDir -Recurse -Force -ErrorAction SilentlyContinue
}

# Create data directories
Write-ColoredOutput "Blue" "📁 Creating node data directories..."
$BootstrapDataDir = Join-Path $Global:DataDir "bootstrap-node"
New-Item -ItemType Directory -Path $BootstrapDataDir -Force | Out-Null

for ($i = 1; $i -le $NodesCount; $i++) {
    $nodeDataDir = Join-Path $Global:DataDir "node-$i"
    New-Item -ItemType Directory -Path $nodeDataDir -Force | Out-Null
}

# Build components
if (!$SkipBuild) {
    Write-Header "🔨 BUILDING COMPONENTS"
    
    # Build Rust backend
    Write-ColoredOutput "Blue" "Building AerolithDB backend..."
    try {
        cargo build --release | Out-Null
        if ($LASTEXITCODE -ne 0) {
            throw "Backend build failed"
        }
        Write-ColoredOutput "Green" "✅ Backend build completed"
    }
    catch {
        Write-ColoredOutput "Red" "❌ Backend build failed"
        exit 1
    }
    
    # Build web client
    Write-ColoredOutput "Blue" "Building Web UI client..."
    try {
        Push-Location "web-client"
        
        # Install dependencies if needed
        if (!(Test-Path "node_modules")) {
            Write-ColoredOutput "Yellow" "Installing web client dependencies..."
            npm install | Out-Null
            if ($LASTEXITCODE -ne 0) {
                throw "npm install failed"
            }
        }
        
        Write-ColoredOutput "Green" "✅ Web client ready"
    }
    catch {
        Write-ColoredOutput "Red" "❌ Web client setup failed"
        exit 1
    }
    finally {
        Pop-Location
    }
}

# Launch backend network
Write-Header "🏗️ LAUNCHING BACKEND NETWORK"

# Start bootstrap node
Write-ColoredOutput "Blue" "🎯 Starting bootstrap node on port $StartPort..."

$env:AEROLITHSDB_NODE_ID = "bootstrap-node-001"
$env:AEROLITHSDB_STORAGE_DATA_DIR = $BootstrapDataDir
$env:AEROLITHSDB_API_REST_PORT = $StartPort.ToString()
$env:AEROLITHSDB_NETWORK_IS_BOOTSTRAP = "true"
$env:RUST_LOG = if ($Verbose) { "debug,aerolithsdb=trace" } else { "info,aerolithsdb=info" }

$bootstrapProcess = Start-Process -FilePath "cargo" -ArgumentList @("run", "--release", "--") -NoNewWindow -PassThru
$script:NodeProcesses += $bootstrapProcess
$bootstrapUrl = "http://localhost:$StartPort"
$script:NodeUrls += $bootstrapUrl

Write-ColoredOutput "Green" "✅ Bootstrap node started (PID: $($bootstrapProcess.Id))"

# Wait for bootstrap node
if (!(Wait-ForHealthy -Url $bootstrapUrl -NodeName "Bootstrap Node")) {
    Write-ColoredOutput "Red" "❌ Bootstrap node failed to start"
    Stop-FullStackEnvironment
    exit 1
}

# Start regular nodes
Write-ColoredOutput "Blue" "🌐 Starting $NodesCount regular nodes..."
for ($i = 1; $i -le $NodesCount; $i++) {
    $nodePort = $StartPort + $i
    $nodeId = "regular-node-$('{0:D3}' -f $i)"
    $nodeDataDir = Join-Path $Global:DataDir "node-$i"
    $nodeUrl = "http://localhost:$nodePort"
    
    Write-ColoredOutput "Cyan" "  Starting node $i on port $nodePort..."
    
    $env:AEROLITHSDB_NODE_ID = $nodeId
    $env:AEROLITHSDB_STORAGE_DATA_DIR = $nodeDataDir
    $env:AEROLITHSDB_API_REST_PORT = $nodePort.ToString()
    $env:AEROLITHSDB_NETWORK_IS_BOOTSTRAP = "false"
    $env:AEROLITHSDB_NETWORK_BOOTSTRAP_NODES = $bootstrapUrl
    
    $nodeProcess = Start-Process -FilePath "cargo" -ArgumentList @("run", "--release", "--") -NoNewWindow -PassThru
    $script:NodeProcesses += $nodeProcess
    $script:NodeUrls += $nodeUrl
    
    Write-ColoredOutput "Green" "    ✅ Node $i started (PID: $($nodeProcess.Id))"
    Start-Sleep -Seconds 2
}

# Wait for all nodes to be healthy
Write-ColoredOutput "Yellow" "⏳ Waiting for network formation..."
$allHealthy = $true
foreach ($url in $script:NodeUrls) {
    $nodeIndex = $script:NodeUrls.IndexOf($url)
    $nodeName = if ($nodeIndex -eq 0) { "Bootstrap Node" } else { "Regular Node $nodeIndex" }
    
    if (!(Wait-ForHealthy -Url $url -NodeName $nodeName -TimeoutSec 30)) {
        $allHealthy = $false
    }
}

if (!$allHealthy) {
    Write-ColoredOutput "Red" "❌ Not all nodes became healthy"
    Stop-FullStackEnvironment
    exit 1
}

# Additional stabilization time
Write-ColoredOutput "Yellow" "⏳ Allowing time for P2P mesh formation..."
Start-Sleep -Seconds 5

Write-ColoredOutput "Green" "✅ Backend network is ready!"

# Launch web client
Write-Header "🌐 LAUNCHING WEB CLIENT"

try {
    Push-Location "web-client"
    
    Write-ColoredOutput "Blue" "🎯 Starting React development server on port $WebPort..."
    
    # Set environment variables for web client
    $env:VITE_API_BASE_URL = "http://localhost:$StartPort"
    $env:VITE_WS_BASE_URL = "ws://localhost:$StartPort"
    $env:PORT = $WebPort.ToString()
    
    # Start the development server
    $script:WebProcess = Start-Process -FilePath "npm" -ArgumentList @("run", "dev", "--", "--port", $WebPort, "--host") -NoNewWindow -PassThru
    
    Write-ColoredOutput "Green" "✅ Web client started (PID: $($script:WebProcess.Id))"
    
}
catch {
    Write-ColoredOutput "Red" "❌ Failed to start web client: $($_.Exception.Message)"
    Stop-FullStackEnvironment
    exit 1
}
finally {
    Pop-Location
}

# Wait for web client to be ready
$webUrl = "http://localhost:$WebPort"
if (Wait-ForWebUI -Url $webUrl -TimeoutSec 45) {
    Write-ColoredOutput "Green" "✅ Web client is ready!"
    
    if ($OpenBrowser) {
        Write-ColoredOutput "Blue" "🌐 Opening browser..."
        Start-Process $webUrl
    }
} else {
    Write-ColoredOutput "Yellow" "⚠️  Web client may still be starting up"
}

# Create some demo data
Write-Header "📝 CREATING DEMO DATA"
Write-ColoredOutput "Blue" "Adding sample documents for testing..."

$DemoDocuments = @(
    @{
        collection = "users"
        id = "demo_user_001"
        data = @{
            name = "Alice Developer"
            email = "alice@aerolithdb.com"
            department = "Engineering"
            role = "Senior Developer"
            location = "San Francisco"
            joined_date = "2024-01-15"
        }
    },
    @{
        collection = "users"
        id = "demo_user_002"
        data = @{
            name = "Bob Manager"
            email = "bob@aerolithdb.com"
            department = "Product"
            role = "Product Manager"
            location = "New York"
            joined_date = "2024-02-01"
        }
    },
    @{
        collection = "projects"
        id = "demo_project_001"
        data = @{
            name = "Web UI Integration"
            description = "Integrating React web client with distributed backend"
            status = "active"
            team_members = @("demo_user_001", "demo_user_002")
            start_date = "2024-03-01"
            priority = "high"
        }
    }
)

foreach ($doc in $DemoDocuments) {
    $targetUrl = $script:NodeUrls[0]  # Use bootstrap node
    $jsonData = $doc.data | ConvertTo-Json -Depth 10
    
    try {
        cargo run --release --bin aerolithsdb-cli -- --url $targetUrl put $doc.collection $doc.id --data $jsonData | Out-Null
        Write-ColoredOutput "Green" "  ✅ Created '$($doc.id)' in '$($doc.collection)'"
    }
    catch {
        Write-ColoredOutput "Yellow" "  ⚠️  Failed to create demo document: $($_.Exception.Message)"
    }
}

# Environment summary
Write-Header "🎯 ENVIRONMENT READY"

Write-ColoredOutput "Green" "✅ AerolithDB Full-Stack Environment is running!"
Write-Host ""
Write-ColoredOutput "Blue" "📊 Environment Summary:"
Write-ColoredOutput "Blue" "   • Backend Network: $(($script:NodeUrls.Count)) nodes"
Write-ColoredOutput "Blue" "   • Web Client: $webUrl"
Write-ColoredOutput "Blue" "   • Demo Data: $(($DemoDocuments.Count)) sample documents"
Write-Host ""

Write-ColoredOutput "Yellow" "🔗 Available Endpoints:"
foreach ($url in $script:NodeUrls) {
    $nodeIndex = $script:NodeUrls.IndexOf($url)
    $nodeType = if ($nodeIndex -eq 0) { "Bootstrap API" } else { "Node $nodeIndex API" }
    Write-ColoredOutput "Yellow" "   • $nodeType : $url"
}
Write-ColoredOutput "Yellow" "   • Web Dashboard: $webUrl"
Write-Host ""

Write-ColoredOutput "Cyan" "💡 Try these features in the Web UI:"
Write-ColoredOutput "Cyan" "   • Dashboard: Real-time cluster overview"
Write-ColoredOutput "Cyan" "   • Network Explorer: Live node monitoring"
Write-ColoredOutput "Cyan" "   • Data Browser: Document management"
Write-ColoredOutput "Cyan" "   • Query Interface: Advanced querying"
Write-ColoredOutput "Cyan" "   • Real-time Monitor: Live data streams"
Write-Host ""

Write-ColoredOutput "Cyan" "🛠️  Manual CLI Testing:"
Write-ColoredOutput "Cyan" "   cargo run --release --bin aerolithsdb-cli -- --url $($script:NodeUrls[0]) health"
Write-ColoredOutput "Cyan" "   cargo run --release --bin aerolithsdb-cli -- --url $($script:NodeUrls[0]) stats"
Write-ColoredOutput "Cyan" "   cargo run --release --bin aerolithsdb-cli -- --url $($script:NodeUrls[0]) get users demo_user_001"
Write-Host ""

Write-ColoredOutput "Bold" "Press Ctrl+C to stop the full-stack environment"

# Keep environment running with monitoring
Write-ColoredOutput "Blue" "🔄 Monitoring environment health... (Press Ctrl+C to exit)"

while ($true) {
    Start-Sleep -Seconds 30
    
    # Quick health check
    $healthyNodes = 0
    $healthyWeb = $false
    
    foreach ($url in $script:NodeUrls) {
        try {
            $response = Invoke-RestMethod -Uri "$url/health" -Method Get -TimeoutSec 3 -ErrorAction SilentlyContinue
            if ($response -and $response.status -eq "healthy") {
                $healthyNodes++
            }
        }
        catch {
            # Node not responding
        }
    }
    
    try {
        $response = Invoke-WebRequest -Uri $webUrl -Method Head -TimeoutSec 3 -ErrorAction SilentlyContinue
        $healthyWeb = ($response.StatusCode -eq 200)
    }
    catch {
        # Web UI not responding
    }
    
    $totalNodes = $script:NodeUrls.Count
    $timestamp = Get-Date -Format "HH:mm:ss"
    
    if ($healthyNodes -eq $totalNodes -and $healthyWeb) {
        Write-ColoredOutput "Green" "[$timestamp] ✅ Environment healthy: $healthyNodes/$totalNodes nodes, Web UI: OK"
    } else {
        Write-ColoredOutput "Yellow" "[$timestamp] ⚠️  Environment status: $healthyNodes/$totalNodes nodes healthy, Web UI: $(if($healthyWeb){'OK'}else{'ERROR'})"
    }
}
