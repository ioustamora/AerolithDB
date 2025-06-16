# Aerolith Renaming Instructions

This guide will help you rename your project from "aerolithsdb" to "aerolith" systematically.

## ⚠️ IMPORTANT: Backup First!
Before starting, create a backup of your entire project directory.

## Step-by-Step Process

### Step 1: Rename Directories
Run the directory renaming script:
```powershell
.\rename-directories.ps1
```

This will rename all directories from `aerolithdb-*` to `aerolith-*`.

### Step 2: Update All File Contents  
Run the comprehensive renaming script:
```powershell
.\rename-aerolith.ps1
```

This will update all references in your code files, documentation, and configuration files.

### Step 3: Verify the Changes
Run the verification script:
```powershell
.\verify-renaming.ps1
```

This will check for any remaining old references and confirm the directory structure.

### Step 4: Test the Build
Verify everything still works:
```powershell
cargo check --workspace
cargo test --workspace  
cargo build --workspace
```

## What Gets Renamed

### Directory Names:
- `aerolithdb-core` → `aerolith-core`
- `aerolithdb-consensus` → `aerolith-consensus`
- `aerolithdb-storage` → `aerolith-storage`
- `aerolithdb-network` → `aerolith-network`
- `aerolithdb-cache` → `aerolith-cache`
- `aerolithdb-security` → `aerolith-security`
- `aerolithdb-query` → `aerolith-query`
- `aerolithdb-api` → `aerolith-api`
- `aerolithdb-plugins` → `aerolith-plugins`
- `aerolithdb-cli` → `aerolith-cli`

### Code References:
- `aerolithsdb` → `aerolith`
- `aerolithsDB` → `Aerolith`
- `aerolithsClient` → `AerolithClient`
- `aerolithsConfig` → `AerolithConfig`
- `aerolithSDB_` → `AEROLITH_` (environment variables)

### Files Updated:
- All `Cargo.toml` files
- All Rust source files (`.rs`)
- Documentation files (`.md`)
- PowerShell scripts (`.ps1`)
- Configuration files

## Manual Updates (if needed)

After running the scripts, you may need to manually update:

1. **README.md**: Project title and descriptions
2. **GitHub repository references**: Update URLs if applicable
3. **License files**: Update project name references
4. **CI/CD configurations**: Update any pipeline references

## Troubleshooting

If you encounter issues:

1. **Build Errors**: Check `cargo check --workspace` output for specific problems
2. **Missing Dependencies**: Verify all `Cargo.toml` files were updated correctly  
3. **Test Failures**: Review test files for hardcoded references
4. **Documentation**: Check `.md` files for formatting issues

## Note on Target Directory

The scripts automatically exclude the `target/` directory and other build artifacts. After renaming, you may want to clean and rebuild:

```powershell
cargo clean
cargo build --workspace
```

This ensures all build artifacts use the new naming convention.
