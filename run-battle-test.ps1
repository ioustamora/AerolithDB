# aerolithsDB Network Battle Test Runner
# This script sets up and runs the comprehensive multi-node battle test

param(
    [switch]$Clean,
    [switch]$Verbose,
    [string]$LogLevel = "info"
)

Write-Host "🚀 aerolithsDB Network Battle Test Runner" -ForegroundColor Cyan
Write-Host "=====================================" -ForegroundColor Cyan

# Set error action preference
$ErrorActionPreference = "Stop"

try {
    # Clean previous test data if requested
    if ($Clean) {
        Write-Host "🧹 Cleaning previous test data..." -ForegroundColor Yellow
        if (Test-Path "test-data") {
            Remove-Item -Recurse -Force "test-data"
        }
        if (Test-Path "test-results") {
            Remove-Item -Recurse -Force "test-results"
        }
    }

    # Create necessary directories
    Write-Host "📁 Creating test directories..." -ForegroundColor Green
    New-Item -ItemType Directory -Force -Path "test-data" | Out-Null
    New-Item -ItemType Directory -Force -Path "test-results" | Out-Null

    # Set environment variables for testing
    $env:RUST_LOG = $LogLevel
    $env:aerolithSDB_TEST_MODE = "true"
    $env:aerolithSDB_TEST_TIMEOUT = "300" # 5 minutes timeout

    # Display test configuration
    Write-Host ""
    Write-Host "🔧 Test Configuration:" -ForegroundColor Magenta
    Write-Host "   Log Level: $LogLevel" -ForegroundColor White
    Write-Host "   Clean Mode: $Clean" -ForegroundColor White
    Write-Host "   Verbose: $Verbose" -ForegroundColor White
    Write-Host "   Test Timeout: 300 seconds" -ForegroundColor White
    Write-Host ""

    # Build the project first
    Write-Host "🔨 Building aerolithsDB..." -ForegroundColor Blue
    if ($Verbose) {
        cargo build --release
    } else {
        cargo build --release | Out-Null
    }
    
    if ($LASTEXITCODE -ne 0) {
        throw "Build failed with exit code $LASTEXITCODE"
    }
    
    Write-Host "✅ Build completed successfully" -ForegroundColor Green

    # Run the battle test
    Write-Host ""
    Write-Host "🔥 Starting Network Battle Test..." -ForegroundColor Red
    Write-Host "This may take several minutes to complete..." -ForegroundColor Yellow
    Write-Host ""

    $testStartTime = Get-Date

    # Run the comprehensive test
    if ($Verbose) {
        cargo test --release --test network_battle_test test_network_battle_comprehensive -- --nocapture
    } else {
        cargo test --release --test network_battle_test test_network_battle_comprehensive -- --nocapture --quiet
    }

    $testExitCode = $LASTEXITCODE
    $testEndTime = Get-Date
    $testDuration = $testEndTime - $testStartTime

    Write-Host ""
    Write-Host "⏱️ Test completed in: $($testDuration.TotalSeconds.ToString('F2')) seconds" -ForegroundColor Cyan

    if ($testExitCode -eq 0) {
        Write-Host "🎉 Battle Test PASSED!" -ForegroundColor Green
        
        # Display results if available
        if (Test-Path "test-results/battle_test_report.txt") {
            Write-Host ""
            Write-Host "📊 Test Report:" -ForegroundColor Magenta
            Write-Host "===============" -ForegroundColor Magenta
            Get-Content "test-results/battle_test_report.txt" | Write-Host
        }
        
        Write-Host ""
        Write-Host "✅ All tests completed successfully!" -ForegroundColor Green
        Write-Host "📁 Test artifacts saved in: test-results/" -ForegroundColor Cyan
    } else {
        Write-Host "❌ Battle Test FAILED!" -ForegroundColor Red
        Write-Host "Exit code: $testExitCode" -ForegroundColor Red
        
        # Show recent logs if available
        $logFiles = Get-ChildItem -Path "test-data" -Filter "*.log" -Recurse -ErrorAction SilentlyContinue
        if ($logFiles) {
            Write-Host ""
            Write-Host "📋 Recent log entries:" -ForegroundColor Yellow
            foreach ($logFile in $logFiles | Select-Object -First 3) {
                Write-Host "--- $($logFile.FullName) ---" -ForegroundColor Gray
                Get-Content $logFile.FullName -Tail 10 -ErrorAction SilentlyContinue | Write-Host
            }
        }
        
        exit $testExitCode
    }

} catch {
    Write-Host ""
    Write-Host "💥 Error occurred during test execution:" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    
    if ($_.Exception.InnerException) {
        Write-Host "Inner exception: $($_.Exception.InnerException.Message)" -ForegroundColor Red
    }
    
    exit 1
} finally {
    # Cleanup environment variables
    Remove-Item Env:RUST_LOG -ErrorAction SilentlyContinue
    Remove-Item Env:aerolithSDB_TEST_MODE -ErrorAction SilentlyContinue
    Remove-Item Env:aerolithSDB_TEST_TIMEOUT -ErrorAction SilentlyContinue
}

Write-Host ""
Write-Host "🏁 Test runner completed." -ForegroundColor Cyan
