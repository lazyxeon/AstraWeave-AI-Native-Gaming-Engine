#!/usr/bin/env python3
"""
KTX2 Migration Validation Script
Validates that all .ktx2 files are in proper KTX2 format and checks metadata.

Usage:
    python validate_ktx2_migration.py [--verbose]
"""

import sys
from pathlib import Path
import json

ASSETS_DIR = Path("assets/materials/baked")
BACKUP_DIR = Path("assets/materials/baked_backup")

# Magic bytes
KTX2_MAGIC = b"\xAB\x4B\x54\x58\x20\x32\x30\xBB\x0D\x0A\x1A\x0A"
AWTEX2_MAGIC = b"AW_TEX2\0"

class ValidationStats:
    def __init__(self):
        self.total = 0
        self.valid_ktx2 = 0
        self.invalid = 0
        self.missing_meta = 0
        self.errors = []

def check_file_format(file_path):
    """Check file format and return detailed info"""
    try:
        with open(file_path, 'rb') as f:
            magic = f.read(12)
            if magic == KTX2_MAGIC:
                return ('KTX2', True)
            elif magic.startswith(AWTEX2_MAGIC):
                return ('AWTEX2', False)
            else:
                return (f'UNKNOWN (magic: {magic.hex()})', False)
    except Exception as e:
        return (f'ERROR: {e}', False)

def validate_metadata(ktx2_path):
    """Validate metadata file exists and is valid JSON"""
    meta_path = Path(str(ktx2_path) + ".meta.json")
    
    if not meta_path.exists():
        return (False, "Missing .meta.json")
    
    try:
        with open(meta_path, 'r') as f:
            meta = json.load(f)
            
        required_fields = ["source_path", "output_path", "sha256"]
        missing = [f for f in required_fields if f not in meta]
        
        if missing:
            return (False, f"Missing fields: {', '.join(missing)}")
        
        return (True, "Valid")
        
    except json.JSONDecodeError as e:
        return (False, f"Invalid JSON: {e}")
    except Exception as e:
        return (False, f"Error: {e}")

def compare_with_backup(ktx2_path, backup_dir):
    """Compare file size with backup if available"""
    backup_path = backup_dir / ktx2_path.name
    
    if not backup_path.exists():
        return None
    
    try:
        original_size = backup_path.stat().st_size
        new_size = ktx2_path.stat().st_size
        diff_pct = ((new_size - original_size) / original_size) * 100
        
        return {
            'original_size': original_size,
            'new_size': new_size,
            'diff_bytes': new_size - original_size,
            'diff_pct': diff_pct
        }
    except Exception as e:
        return {'error': str(e)}

def main():
    import argparse
    parser = argparse.ArgumentParser(description='Validate KTX2 migration')
    parser.add_argument('--verbose', '-v', action='store_true',
                        help='Show detailed information for each file')
    parser.add_argument('--compare-backup', action='store_true',
                        help='Compare file sizes with backup')
    args = parser.parse_args()
    
    if not ASSETS_DIR.exists():
        print(f"❌ Assets directory not found: {ASSETS_DIR}")
        sys.exit(1)
    
    ktx2_files = sorted(ASSETS_DIR.glob("*.ktx2"))
    
    if not ktx2_files:
        print(f"❌ No .ktx2 files found in {ASSETS_DIR}")
        sys.exit(1)
    
    print(f"Validating {len(ktx2_files)} .ktx2 files\n")
    
    stats = ValidationStats()
    stats.total = len(ktx2_files)
    
    for ktx2_file in ktx2_files:
        format_type, is_valid = check_file_format(ktx2_file)
        meta_valid, meta_msg = validate_metadata(ktx2_file)
        
        if is_valid and meta_valid:
            stats.valid_ktx2 += 1
            status = "[OK]"
        else:
            stats.invalid += 1
            stats.errors.append(ktx2_file.name)
            status = "[FAIL]"
        
        if not meta_valid:
            stats.missing_meta += 1
        
        if args.verbose or not is_valid or not meta_valid:
            print(f"{status} {ktx2_file.name}")
            print(f"   Format: {format_type}")
            print(f"   Metadata: {meta_msg}")
            
            if args.compare_backup:
                comparison = compare_with_backup(ktx2_file, BACKUP_DIR)
                if comparison:
                    if 'error' in comparison:
                        print(f"   Backup compare: {comparison['error']}")
                    else:
                        print(f"   Size: {comparison['new_size']:,} bytes "
                              f"(Δ {comparison['diff_pct']:+.1f}%)")
            print()
    
    # Summary
    print(f"{'='*60}")
    print(f"VALIDATION SUMMARY")
    print(f"{'='*60}")
    print(f"Total files:        {stats.total}")
    print(f"Valid KTX2:         {stats.valid_ktx2}")
    print(f"Invalid/Legacy:     {stats.invalid}")
    print(f"Missing metadata:   {stats.missing_meta}")
    
    if stats.errors:
        print(f"\n[FAIL] Invalid files:")
        for error in stats.errors:
            print(f"   - {error}")
        sys.exit(1)
    else:
        print(f"\n[OK] All files validated successfully!")

if __name__ == "__main__":
    main()
