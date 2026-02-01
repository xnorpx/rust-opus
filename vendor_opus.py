#!/usr/bin/env python3
"""
Opus Vendor Script

Downloads the Opus codec source from GitHub, vendors it locally,
generates version tracking, runs autogen, and creates Rust bindings.
"""

import argparse
import os
import sys
import subprocess
import shutil
import platform
import tempfile
from datetime import datetime
from pathlib import Path
import urllib.request
import json


def run_command(cmd, cwd=None, check=True):
    """Run a command and return output."""
    print(f"Running: {' '.join(cmd) if isinstance(cmd, list) else cmd}")
    result = subprocess.run(
        cmd,
        cwd=cwd,
        shell=isinstance(cmd, str),
        capture_output=True,
        text=True
    )
    if check and result.returncode != 0:
        print(f"STDOUT: {result.stdout}")
        print(f"STDERR: {result.stderr}")
        raise RuntimeError(f"Command failed with code {result.returncode}")
    return result


def get_latest_commit_info(repo_owner, repo_name, branch="main"):
    """Get the latest commit info from GitHub API."""
    url = f"https://api.github.com/repos/{repo_owner}/{repo_name}/commits/{branch}"
    req = urllib.request.Request(url)
    req.add_header("Accept", "application/vnd.github.v3+json")
    req.add_header("User-Agent", "opus-vendor-script")
    
    with urllib.request.urlopen(req) as response:
        data = json.loads(response.read().decode())
        return {
            "sha": data["sha"],
            "date": data["commit"]["committer"]["date"][:10],
            "message": data["commit"]["message"].split("\n")[0]
        }


def get_current_vendored_commit(vendored_dir):
    """Read the current vendored commit from OPUS_VERSION file."""
    version_file = vendored_dir / "OPUS_VERSION"
    if not version_file.exists():
        return None
    
    content = version_file.read_text()
    import re
    match = re.search(r'^commit:\s*([a-fA-F0-9]+)', content, re.MULTILINE)
    if match:
        return match.group(1)
    return None


def clone_repo(repo_url, target_dir, commit=None):
    """Clone a repository to the target directory."""
    if target_dir.exists():
        print(f"Removing existing directory: {target_dir}")
        shutil.rmtree(target_dir)
    
    target_dir.parent.mkdir(parents=True, exist_ok=True)
    
    print(f"Cloning {repo_url} to {target_dir}...")
    run_command(["git", "clone", "--depth", "1", repo_url, str(target_dir)])
    
    if commit:
        # For a specific commit, we need a full clone
        run_command(["git", "fetch", "--unshallow"], cwd=target_dir, check=False)
        run_command(["git", "checkout", commit], cwd=target_dir)
    
    # Get the actual commit we're at
    result = run_command(["git", "rev-parse", "HEAD"], cwd=target_dir)
    actual_commit = result.stdout.strip()
    
    # Remove .git directory so it can be checked in as regular files
    git_dir = target_dir / ".git"
    if git_dir.exists():
        print(f"Removing {git_dir} to allow vendoring as regular files...")
        shutil.rmtree(git_dir)
    
    return actual_commit


def generate_version_file(vendored_dir, commit_sha, commit_date):
    """Generate the OPUS_VERSION file."""
    version_content = f"""# Opus Vendor Information
#
# This file tracks the exact version of the vendored Opus source.

source: https://github.com/xiph/opus
commit: {commit_sha}
date: {commit_date}
branch: main

# To update, run this script again:
# python vendor_opus.py
#
# Or manually:
# git clone https://github.com/xiph/opus vendored/opus
# cd vendored/opus && git checkout <NEW_COMMIT>
"""
    
    version_file = vendored_dir / "OPUS_VERSION"
    version_file.write_text(version_content)
    print(f"Generated {version_file}")


def parse_autogen_for_model_hash(opus_dir):
    """Parse autogen.sh to extract the model hash."""
    autogen_script = opus_dir / "autogen.sh"
    
    if not autogen_script.exists():
        print(f"  Warning: {autogen_script} not found")
        return None
    
    content = autogen_script.read_text()
    
    # Look for pattern: dnn/download_model.sh "hash"
    import re
    match = re.search(r'dnn/download_model\.sh\s+["\']?([a-fA-F0-9]+)["\']?', content)
    
    if match:
        model_hash = match.group(1)
        print(f"  Found model hash: {model_hash}")
        return model_hash
    else:
        print("  Warning: Could not find model hash in autogen.sh")
        return None


def download_opus_model(opus_dir, model_hash):
    """Download the Opus DNN model using the appropriate script."""
    if not model_hash:
        print("  No model hash provided, skipping model download")
        return
    
    system = platform.system()
    
    # Run from opus root directory - the tarball extracts with dnn/ prefix
    if system == "Windows":
        download_script = opus_dir / "dnn" / "download_model.bat"
        if download_script.exists():
            print(f"  Running download_model.bat with hash {model_hash[:16]}...")
            run_command(["cmd", "/c", "dnn\\download_model.bat", model_hash], cwd=opus_dir)
        else:
            print(f"  Warning: {download_script} not found")
    else:
        download_script = opus_dir / "dnn" / "download_model.sh"
        if download_script.exists():
            print(f"  Running download_model.sh with hash {model_hash[:16]}...")
            run_command(["chmod", "+x", str(download_script)])
            run_command(["dnn/download_model.sh", model_hash], cwd=opus_dir)
        else:
            print(f"  Warning: {download_script} not found")


def generate_bindings(opus_dir, output_dir):
    """Generate Rust bindings using bindgen."""
    include_dir = opus_dir / "include"
    header_file = include_dir / "opus.h"
    
    if not header_file.exists():
        print(f"Warning: {header_file} not found")
        # Try to find opus.h elsewhere
        for h in opus_dir.rglob("opus.h"):
            if "include" in str(h):
                header_file = h
                include_dir = h.parent
                break
    
    output_file = output_dir / "bindings.rs"
    output_dir.mkdir(parents=True, exist_ok=True)
    
    print(f"Generating bindings from {include_dir}...")
    
    # Create a wrapper header that includes all public opus headers
    wrapper_header = include_dir / "opus_all.h"
    wrapper_content = """/* Wrapper header for bindgen - includes all public Opus APIs */
#include "opus.h"
#include "opus_multistream.h"
#include "opus_projection.h"
"""
    wrapper_header.write_text(wrapper_content)
    
    try:
        cmd = [
            "bindgen",
            str(wrapper_header),
            "--output", str(output_file),
            # Add module-level attribute to suppress broken doc link warnings
            # (Doxygen @param [in]/[out] syntax is misinterpreted as Rust doc links)
            "--raw-line", "#![allow(rustdoc::broken_intra_doc_links)]",
            "--",
            f"-I{include_dir}",
        ]
        run_command(cmd)
        print(f"Generated bindings at {output_file}")
    except Exception as e:
        raise RuntimeError(f"bindgen failed: {e}. Please install bindgen-cli: cargo install bindgen-cli")
    finally:
        if wrapper_header.exists():
            wrapper_header.unlink()


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Vendor Opus source code and generate Rust bindings"
    )
    parser.add_argument(
        "--bindings-only",
        action="store_true",
        help="Only regenerate bindings from existing vendored source (skip download)"
    )
    parser.add_argument(
        "--force",
        action="store_true", 
        help="Force update even if already at latest commit"
    )
    args = parser.parse_args()

    project_dir = Path(__file__).parent.resolve()
    vendored_dir = project_dir / "vendored"
    opus_dir = vendored_dir / "opus"
    src_dir = project_dir / "src"
    
    print("=" * 60)
    print("Opus Vendor Script")
    print("=" * 60)

    # Handle --bindings-only mode
    if args.bindings_only:
        if not opus_dir.exists():
            print(f"Error: Vendored source not found at {opus_dir}")
            print("Run without --bindings-only first to download the source.")
            sys.exit(1)
        
        print("\nRegenerating Rust bindings...")
        src_dir.mkdir(parents=True, exist_ok=True)
        generate_bindings(opus_dir, src_dir)
        print("\n" + "=" * 60)
        print("Done! Bindings regenerated.")
        print("=" * 60)
        return
    
    # Step 1: Get latest commit info from GitHub
    print("\n[1/5] Fetching latest commit info from GitHub...")
    try:
        commit_info = get_latest_commit_info("xiph", "opus")
        print(f"  Latest commit: {commit_info['sha'][:12]}")
        print(f"  Date: {commit_info['date']}")
        print(f"  Message: {commit_info['message']}")
    except Exception as e:
        print(f"  Warning: Could not fetch commit info: {e}")
        commit_info = {
            "sha": "unknown",
            "date": datetime.now().strftime("%Y-%m-%d"),
            "message": "unknown"
        }
    
    # Check if already up to date
    current_commit = get_current_vendored_commit(vendored_dir)
    if current_commit and commit_info["sha"] != "unknown":
        if current_commit == commit_info["sha"] and not args.force:
            print(f"\n  Already up to date at commit {current_commit[:12]}")
            print("\n" + "=" * 60)
            print("No update needed! Use --force to update anyway.")
            print("=" * 60)
            return
        else:
            print(f"  Current vendored commit: {current_commit[:12]}")
            print(f"  Updating to: {commit_info['sha'][:12]}")
    
    # Step 2: Clone the Opus repository
    print("\n[2/5] Cloning Opus repository...")
    try:
        actual_commit = clone_repo(
            "https://github.com/xiph/opus.git",
            opus_dir,
            commit=None  # Use latest
        )
        print(f"  Cloned at commit: {actual_commit[:12]}")
        commit_info["sha"] = actual_commit
    except Exception as e:
        print(f"  Error cloning repository: {e}")
        sys.exit(1)
    
    # Step 3: Generate OPUS_VERSION file
    print("\n[3/5] Generating OPUS_VERSION file...")
    generate_version_file(vendored_dir, commit_info["sha"], commit_info["date"])
    
    # Step 4: Parse autogen.sh and download model
    print("\n[4/5] Downloading Opus DNN model...")
    try:
        model_hash = parse_autogen_for_model_hash(opus_dir)
        if model_hash:
            download_opus_model(opus_dir, model_hash)
    except Exception as e:
        print(f"  Warning: model download failed (may be OK): {e}")
    
    # Step 5: Generate bindings
    print("\n[5/5] Generating Rust bindings...")
    src_dir.mkdir(parents=True, exist_ok=True)
    generate_bindings(opus_dir, src_dir)
    
    print("\n" + "=" * 60)
    print("Done!")
    print("=" * 60)
    print(f"""
Updated files:
  - {vendored_dir / "OPUS_VERSION"}
  - {vendored_dir / "opus"} (source code)
  - {src_dir / "bindings.rs"}

To build and test:
  cargo build
  cargo test

To regenerate bindings manually:
  bindgen vendored/opus/include/opus_all.h -o src/bindings.rs -- -I vendored/opus/include
""")


if __name__ == "__main__":
    main()
