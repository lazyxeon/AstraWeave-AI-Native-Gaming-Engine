#!/usr/bin/env python3
"""
AstraWeave Asset Migration Script: AWTEX2 → KTX2
Migrates all legacy AWTEX2 texture files to industry-standard KTX2 format.

Usage:
    python migrate_awtex2_to_ktx2.py [--dry-run] [--backup-dir <path>]
"""

import os
import sys
import subprocess
import json
import shutil
from pathlib import Path
from datetime import datetime

# Configuration
ASSETS_DIR = Path("assets/materials/baked")
BACKUP_DIR = Path("assets/materials/baked_backup")
CARGO_BIN = "cargo"
PACKAGE = "aw_asset_cli"

# AWTEX2 magic: "AW_TEX2\0" or "AWTEX2\0\0"
AWTEX2_MAGIC = b"AW_TEX2\0"
AWTEX2_MAGIC_ALT = b"AWTEX2\0\0"
# KTX2 magic: 0xAB 0x4B 0x54 0x58 0x20 0x32 0x30 0xBB 0x0D 0x0A 0x1A 0x0A
KTX2_MAGIC = b"\xAB\x4B\x54\x58\x20\x32\x30\xBB\x0D\x0A\x1A\x0A"

class MigrationStats:
    def __init__(self):
        self.total = 0
        self.already_ktx2 = 0
        self.migrated = 0
        self.failed = 0
        self.errors = []

def check_magic_bytes(file_path):
    """Determine if file is AWTEX2 or KTX2"""
    try:
        with open(file_path, 'rb') as f:
            magic = f.read(12)
            if magic.startswith(KTX2_MAGIC):
                return 'KTX2'
            elif magic.startswith(AWTEX2_MAGIC) or magic.startswith(AWTEX2_MAGIC_ALT):
                return 'AWTEX2'
            else:
                return 'UNKNOWN'
    except Exception as e:
        return f'ERROR: {e}'

def find_source_from_meta(ktx2_path):
    """Find source PNG path from .meta.json"""
    meta_path = Path(str(ktx2_path) + ".meta.json")
    if not meta_path.exists():
        return None
    
    try:
        with open(meta_path, 'r') as f:
            meta = json.load(f)
            return meta.get("source_path")
    except Exception as e:
        print(f"  [FAIL] Failed to read metadata: {e}")
        return None

def backup_file(file_path, backup_dir):
    """Create backup of original file"""
    backup_dir.mkdir(parents=True, exist_ok=True)
    backup_path = backup_dir / file_path.name
    shutil.copy2(file_path, backup_path)
    
    # Also backup metadata if exists
    meta_path = Path(str(file_path) + ".meta.json")
    if meta_path.exists():
        backup_meta = backup_dir / meta_path.name
        shutil.copy2(meta_path, backup_meta)

def migrate_asset(ktx2_path, dry_run=False, backup_dir=None):
    """Migrate a single AWTEX2 file to KTX2"""
    print(f"\nProcessing: {ktx2_path.name}")
    
    # Check current format
    format_type = check_magic_bytes(ktx2_path)
    print(f"  Format: {format_type}")
    
    if format_type == 'KTX2':
        print(f"  [OK] Already KTX2 format - skipping")
        return 'SKIP'
    
    if format_type not in ['AWTEX2']:
        print(f"  [FAIL] Unknown format - skipping")
        return 'ERROR'
    
    # Find source PNG
    source_path = find_source_from_meta(ktx2_path)
    if not source_path:
        print(f"  [FAIL] No source_path in metadata")
        return 'ERROR'
    
    source_path = Path(source_path)
    if not source_path.exists():
        print(f"  [FAIL] Source not found: {source_path}")
        return 'ERROR'
    
    print(f"  Source: {source_path}")
    
    if dry_run:
        print(f"  [DRY RUN] Would re-bake from {source_path}")
        return 'DRY_RUN'
    
    # Backup original
    if backup_dir:
        print(f"  Backing up to {backup_dir / ktx2_path.name}")
        backup_file(ktx2_path, backup_dir)
    
    # Re-bake using new KTX2 writer
    output_dir = ktx2_path.parent
    cmd = [
        CARGO_BIN, "run", "--release", "-p", PACKAGE, "--",
        "bake-texture",
        str(source_path),
        str(output_dir)
    ]
    
    print(f"  Re-baking texture...")
    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=120
        )
        
        if result.returncode != 0:
            print(f"  [FAIL] Bake failed:")
            print(f"    {result.stderr}")
            return 'ERROR'
        
        # Verify KTX2 magic bytes
        new_format = check_magic_bytes(ktx2_path)
        if new_format != 'KTX2':
            print(f"  [FAIL] Migration failed - not KTX2: {new_format}")
            return 'ERROR'
        
        print(f"  [OK] Successfully migrated to KTX2")
        return 'SUCCESS'
        
    except subprocess.TimeoutExpired:
        print(f"  [FAIL] Timeout (120s)")
        return 'ERROR'
    except Exception as e:
        print(f"  [FAIL] Exception: {e}")
        return 'ERROR'

def main():
    import argparse
    parser = argparse.ArgumentParser(description='Migrate AWTEX2 textures to KTX2')
    parser.add_argument('--dry-run', action='store_true',
                        help='Show what would be done without doing it')
    parser.add_argument('--backup-dir', type=Path, default=BACKUP_DIR,
                        help='Backup directory (default: assets/materials/baked_backup)')
    parser.add_argument('--no-backup', action='store_true',
                        help='Skip backup (not recommended)')
    args = parser.parse_args()
    
    if not ASSETS_DIR.exists():
        print(f"[ERROR] Assets directory not found: {ASSETS_DIR}")
        print(f"        Please run from repository root")
        sys.exit(1)
    
    # Find all .ktx2 files
    ktx2_files = sorted(ASSETS_DIR.glob("*.ktx2"))
    
    if not ktx2_files:
        print(f"[ERROR] No .ktx2 files found in {ASSETS_DIR}")
        sys.exit(1)
    
    print(f"Found {len(ktx2_files)} .ktx2 files")
    
    if args.dry_run:
        print(f"DRY RUN MODE - no changes will be made")
    
    if not args.no_backup and not args.dry_run:
        print(f"Backups will be saved to: {args.backup_dir}")
    
    stats = MigrationStats()
    stats.total = len(ktx2_files)
    
    # Process each file
    for ktx2_file in ktx2_files:
        result = migrate_asset(
            ktx2_file,
            dry_run=args.dry_run,
            backup_dir=None if args.no_backup else args.backup_dir
        )
        
        if result == 'SUCCESS':
            stats.migrated += 1
        elif result == 'SKIP':
            stats.already_ktx2 += 1
        elif result == 'ERROR':
            stats.failed += 1
            stats.errors.append(ktx2_file.name)
    
    # Summary report
    print(f"\n{'='*60}")
    print(f"MIGRATION SUMMARY")
    print(f"{'='*60}")
    print(f"Total files:        {stats.total}")
    print(f"Already KTX2:       {stats.already_ktx2}")
    print(f"Migrated:           {stats.migrated}")
    print(f"Failed:             {stats.failed}")
    
    if stats.errors:
        print(f"\n[FAIL] Failed files:")
        for error in stats.errors:
            print(f"   - {error}")
    
    if args.dry_run:
        print(f"\n[DRY RUN] This was a DRY RUN - no changes were made")
    elif stats.failed == 0:
        print(f"\n[OK] Migration completed successfully!")
    else:
        print(f"\n[WARNING] Migration completed with {stats.failed} errors")
        sys.exit(1)

if __name__ == "__main__":
    main()
