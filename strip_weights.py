#!/usr/bin/env python3
"""
Strip embedded weight data from Opus DNN *_data.c files.

This script removes the large static weight arrays from the *_data.c files
while preserving the init_* functions that are needed for runtime weight loading.

The weight data is guarded by `#ifndef USE_WEIGHTS_FILE` preprocessor blocks.
This script removes the content inside those blocks, drastically reducing file sizes
from ~10-50MB each to just a few KB.

Usage:
    python strip_weights.py [--dry-run]
"""

import os
import re
import sys
from pathlib import Path

# Files to process
DATA_FILES = [
    "pitchdnn_data.c",
    "fargan_data.c",
    "plc_data.c",
    "dred_rdovae_enc_data.c",
    "dred_rdovae_dec_data.c",
    "lace_data.c",
    "nolace_data.c",
    "bbwenet_data.c",
]

def strip_weights_from_file(filepath: Path, dry_run: bool = False) -> tuple[int, int]:
    """
    Strip weight data from a single *_data.c file.
    
    Returns (original_size, new_size) in bytes.
    """
    if not filepath.exists():
        print(f"  SKIP: {filepath.name} (not found)")
        return (0, 0)
    
    content = filepath.read_text(encoding='utf-8')
    original_size = len(content.encode('utf-8'))
    
    # Pattern to match #ifndef USE_WEIGHTS_FILE ... #endif /* USE_WEIGHTS_FILE */
    # This captures everything between these markers (the large weight arrays)
    # We need to be careful to match the correct #endif
    
    # Strategy: Find all #ifndef USE_WEIGHTS_FILE and their matching #endif
    # The content has nested #ifdef blocks so we need to track depth
    
    lines = content.split('\n')
    new_lines = []
    in_weights_block = False
    depth = 0
    removed_lines = 0
    
    for line in lines:
        stripped = line.strip()
        
        if stripped == '#ifndef USE_WEIGHTS_FILE':
            in_weights_block = True
            depth = 1
            # Keep the #ifndef line but add a comment
            new_lines.append(line)
            new_lines.append('/* Weight data stripped by strip_weights.py for crate size reduction */')
            continue
        
        if in_weights_block:
            # Track nested #if/#ifdef/#ifndef
            if stripped.startswith('#if'):
                depth += 1
            elif stripped.startswith('#endif'):
                depth -= 1
                if depth == 0:
                    # End of USE_WEIGHTS_FILE block
                    in_weights_block = False
                    new_lines.append(line)  # Keep the #endif
                    continue
            
            # Skip content inside the block (the weight data)
            removed_lines += 1
            continue
        
        new_lines.append(line)
    
    new_content = '\n'.join(new_lines)
    new_size = len(new_content.encode('utf-8'))
    
    if not dry_run and removed_lines > 0:
        filepath.write_text(new_content, encoding='utf-8')
    
    return (original_size, new_size)


def main():
    dry_run = '--dry-run' in sys.argv
    
    script_dir = Path(__file__).parent
    dnn_dir = script_dir / "vendored" / "opus" / "dnn"
    
    if not dnn_dir.exists():
        print(f"Error: DNN directory not found: {dnn_dir}")
        print("Run 'python vendor_opus.py' first to download Opus source.")
        sys.exit(1)
    
    print("=" * 60)
    print("Stripping weight data from Opus DNN files")
    if dry_run:
        print("(DRY RUN - no files will be modified)")
    print("=" * 60)
    print()
    
    total_original = 0
    total_new = 0
    
    for filename in DATA_FILES:
        filepath = dnn_dir / filename
        original, new = strip_weights_from_file(filepath, dry_run)
        
        if original > 0:
            reduction = (1 - new / original) * 100
            print(f"  {filename}:")
            print(f"    Original: {original:,} bytes ({original / 1_000_000:.2f} MB)")
            print(f"    Stripped: {new:,} bytes ({new / 1_000:.2f} KB)")
            print(f"    Reduction: {reduction:.1f}%")
            print()
            
            total_original += original
            total_new += new
    
    print("=" * 60)
    print(f"Total original: {total_original:,} bytes ({total_original / 1_000_000:.2f} MB)")
    print(f"Total stripped: {total_new:,} bytes ({total_new / 1_000:.2f} KB)")
    print(f"Total reduction: {(1 - total_new / total_original) * 100:.1f}%")
    print("=" * 60)
    
    if dry_run:
        print("\nRun without --dry-run to apply changes.")
    else:
        print("\nDone! Weight data has been stripped from source files.")
        print("The init_* functions are preserved for runtime weight loading.")


if __name__ == "__main__":
    main()
