# Simple AerolithDB Test Script
# Test the basic functionality without complex networking

Write-Host "=== AerolithDB Simple Test ===" -ForegroundColor Cyan
Write-Host ""

# Step 1: Build the project
Write-Host "Building AerolithDB..." -ForegroundColor Yellow
$buildResult = cargo build --release 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed:" -ForegroundColor Red
    Write-Host $buildResult
    exit 1
}
Write-Host "Build completed successfully" -ForegroundColor Green
Write-Host ""

# Step 2: Start a single server instance in background
Write-Host "Starting AerolithDB server..." -ForegroundColor Yellow
$serverProcess = Start-Process -FilePath "cargo" -ArgumentList @(
    "run", "--release", "--bin", "aerolithdb"
) -NoNewWindow -PassThru

Write-Host "Server started (PID: $($serverProcess.Id))" -ForegroundColor Green
Write-Host "Waiting for server to initialize..." -ForegroundColor Yellow

# Wait a bit for the server to start
Start-Sleep -Seconds 5

# Step 3: Test server health
Write-Host "Testing server health..." -ForegroundColor Yellow
try {
    $healthResponse = Invoke-RestMethod -Uri "http://localhost:8080/health" -Method Get -TimeoutSec 10
    Write-Host "Health check successful: $($healthResponse | ConvertTo-Json)" -ForegroundColor Green
}
catch {
    Write-Host "Health check failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Step 4: Test CLI commands
Write-Host ""
Write-Host "Testing CLI commands..." -ForegroundColor Yellow

# Test CLI help
Write-Host "CLI Help:" -ForegroundColor Cyan
cargo run --release -p aerolithdb-cli -- --help 2>&1

Write-Host ""
Write-Host "Testing CLI health command..." -ForegroundColor Yellow
cargo run --release -p aerolithdb-cli -- health 2>&1

Write-Host ""
Write-Host "Testing CLI stats command..." -ForegroundColor Yellow  
cargo run --release -p aerolithdb-cli -- stats 2>&1

# Step 5: Test document operations
Write-Host ""
Write-Host "Testing document operations..." -ForegroundColor Yellow

Write-Host "Creating test document..." -ForegroundColor Cyan
cargo run --release -p aerolithdb-cli -- put users user_001 '{"name": "Test User", "email": "test@example.com"}' 2>&1

Write-Host ""
Write-Host "Retrieving test document..." -ForegroundColor Cyan
cargo run --release -p aerolithdb-cli -- get users user_001 2>&1

Write-Host ""
Write-Host "Listing documents..." -ForegroundColor Cyan
cargo run --release -p aerolithdb-cli -- list users 2>&1

# Step 6: Keep server running for manual testing
Write-Host ""
Write-Host "=== Test Complete ===" -ForegroundColor Green
Write-Host "Server is still running for manual testing" -ForegroundColor Yellow
Write-Host "Server endpoints:" -ForegroundColor Cyan
Write-Host "  - REST API: http://localhost:8080/api/v1/" -ForegroundColor White
Write-Host "  - Health: http://localhost:8080/health" -ForegroundColor White
Write-Host "  - GraphQL: http://localhost:8081/graphql" -ForegroundColor White
Write-Host ""
Write-Host "Manual CLI testing examples:" -ForegroundColor Cyan
Write-Host "  cargo run --release -p aerolithdb-cli -- health" -ForegroundColor White
Write-Host "  cargo run --release -p aerolithdb-cli -- get users user_001" -ForegroundColor White
Write-Host "  cargo run --release -p aerolithdb-cli -- put test key_001 '{\"test\": \"value\"}'" -ForegroundColor White
Write-Host ""
Write-Host "Press Ctrl+C to stop the server" -ForegroundColor Magenta

# Keep running until user stops
try {
    while ($true) {
        Start-Sleep -Seconds 2
        # Quick health check
        try {
            $response = Invoke-RestMethod -Uri "http://localhost:8080/health" -Method Get -TimeoutSec 2 -ErrorAction SilentlyContinue
            # Server is healthy, continue
        }
        catch {
            Write-Host "Warning: Server appears to be down" -ForegroundColor Yellow
            break
        }
    }
}
catch {
    Write-Host "Shutting down..." -ForegroundColor Yellow
}
finally {
    # Clean shutdown
    Write-Host "Stopping server..." -ForegroundColor Yellow
    try {
        if (-not $serverProcess.HasExited) {
            $serverProcess.Kill()
            $serverProcess.WaitForExit(5000)
        }
        Write-Host "Server stopped" -ForegroundColor Green
    }
    catch {
        Write-Host "Error stopping server: $($_.Exception.Message)" -ForegroundColor Red
    }
}
