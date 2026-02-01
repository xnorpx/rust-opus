#!/usr/bin/env python3
"""Check if vendored Opus needs updating. Exits with code 1 if update needed."""

import sys
import re
import urllib.request
import json
from pathlib import Path


def get_latest_commit():
    """Get latest commit SHA from GitHub."""
    url = "https://api.github.com/repos/xiph/opus/commits/main"
    req = urllib.request.Request(url)
    req.add_header("Accept", "application/vnd.github.v3+json")
    req.add_header("User-Agent", "opus-version-check")
    
    with urllib.request.urlopen(req) as response:
        data = json.loads(response.read().decode())
        return data["sha"]


def get_vendored_commit():
    """Get current vendored commit from OPUS_VERSION."""
    version_file = Path(__file__).parent / "vendored" / "OPUS_VERSION"
    if not version_file.exists():
        return None
    
    match = re.search(r'^commit:\s*([a-fA-F0-9]+)', version_file.read_text(), re.MULTILINE)
    return match.group(1) if match else None


def main():
    print("Checking Opus version...")
    
    vendored = get_vendored_commit()
    if not vendored:
        print("ERROR: No vendored version found")
        return 1
    
    latest = get_latest_commit()
    
    print(f"  Vendored: {vendored[:12]}")
    print(f"  Latest:   {latest[:12]}")
    
    if vendored == latest:
        print("✓ Up to date")
        return 0
    else:
        print("✗ Update needed! Run: python vendor_opus.py")
        return 1


if __name__ == "__main__":
    sys.exit(main())
