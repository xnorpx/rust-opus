#!/usr/bin/env python3
"""
Generate Opus DNN weight blob.

This script:
1. Downloads Opus source to a temporary directory (using same commit as vendored)
2. Applies the binary mode patch for Windows compatibility
3. Compiles write_lpcnet_weights.c
4. Runs it to generate weights_blob.bin
5. Creates opus_data-<hash>.bin with the model hash
6. Cleans up the temporary directory

Usage:
    python generate_weights.py                # Generate weights to target/model
    python generate_weights.py --output DIR   # Output to specific directory
"""

import argparse
import hashlib
import os
import platform
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


OPUS_REPO_URL = "https://github.com/xiph/opus.git"


def run_command(cmd, cwd=None, check=True):
    """Run a command and return output."""
    print(f"  Running: {' '.join(cmd) if isinstance(cmd, list) else cmd}")
    result = subprocess.run(
        cmd,
        cwd=cwd,
        shell=isinstance(cmd, str),
        capture_output=True,
        text=True
    )
    if check and result.returncode != 0:
        print(f"  STDOUT: {result.stdout}")
        print(f"  STDERR: {result.stderr}")
        raise RuntimeError(f"Command failed with code {result.returncode}")
    return result


def get_vendored_commit(script_dir):
    """Read the commit hash from vendored/OPUS_VERSION, or None if not found."""
    version_file = script_dir / "vendored" / "OPUS_VERSION"
    
    if not version_file.exists():
        return None
    
    content = version_file.read_text()
    match = re.search(r'^commit:\s*([a-fA-F0-9]+)', content, re.MULTILINE)
    
    if not match:
        return None
    
    return match.group(1)


def clone_opus(target_dir, commit=None):
    """Clone Opus repository and optionally checkout specific commit."""
    if commit:
        # Need full history to checkout a specific commit
        print(f"  Cloning {OPUS_REPO_URL}...")
        run_command(["git", "clone", "--depth", "1", OPUS_REPO_URL, str(target_dir)])
        
        print(f"  Fetching full history...")
        run_command(["git", "fetch", "--unshallow"], cwd=target_dir, check=False)
        
        print(f"  Checking out commit {commit[:12]}...")
        run_command(["git", "checkout", commit], cwd=target_dir)
    else:
        # Just clone HEAD (shallow is fine)
        print(f"  Cloning {OPUS_REPO_URL} (HEAD)...")
        run_command(["git", "clone", "--depth", "1", OPUS_REPO_URL, str(target_dir)])
    
    return target_dir


def apply_binary_mode_patch(opus_dir):
    """Apply the binary mode patch to write_lpcnet_weights.c for Windows compatibility."""
    write_weights_file = opus_dir / "dnn" / "write_lpcnet_weights.c"
    
    if not write_weights_file.exists():
        raise RuntimeError(f"write_lpcnet_weights.c not found: {write_weights_file}")
    
    content = write_weights_file.read_text()
    
    # Check if already patched
    if '"wb"' in content:
        print("  Binary mode patch already applied")
        return
    
    # Apply the patch
    old_line = 'fopen("weights_blob.bin", "w")'
    new_line = 'fopen("weights_blob.bin", "wb")'
    
    if old_line not in content:
        raise RuntimeError("Could not find fopen line to patch")
    
    content = content.replace(old_line, new_line)
    write_weights_file.write_text(content)
    print("  Applied binary mode patch")


def apply_bwe_weights_patch(opus_dir):
    """Add OSCE BWE weights to write_lpcnet_weights.c."""
    write_weights_file = opus_dir / "dnn" / "write_lpcnet_weights.c"
    
    if not write_weights_file.exists():
        raise RuntimeError(f"write_lpcnet_weights.c not found: {write_weights_file}")
    
    content = write_weights_file.read_text()
    
    # Check if already patched
    if 'ENABLE_OSCE_BWE' in content:
        print("  BWE weights patch already applied")
        return
    
    # Add include for bbwenet_data.c
    old_include = '''#ifdef ENABLE_OSCE
#include "lace_data.c"
#include "nolace_data.c"
#endif'''
    new_include = '''#ifdef ENABLE_OSCE
#include "lace_data.c"
#include "nolace_data.c"
#endif
#ifdef ENABLE_OSCE_BWE
#include "bbwenet_data.c"
#endif'''
    
    if old_include not in content:
        raise RuntimeError("Could not find OSCE include block to patch")
    
    content = content.replace(old_include, new_include)
    
    # Add write_weights call for bbwenet
    old_main = '''#ifdef ENABLE_OSCE
#ifndef DISABLE_LACE
  write_weights(lacelayers_arrays, fout);
#endif
#ifndef DISABLE_NOLACE
  write_weights(nolacelayers_arrays, fout);
#endif
#endif
  fclose(fout);'''
    new_main = '''#ifdef ENABLE_OSCE
#ifndef DISABLE_LACE
  write_weights(lacelayers_arrays, fout);
#endif
#ifndef DISABLE_NOLACE
  write_weights(nolacelayers_arrays, fout);
#endif
#endif
#ifdef ENABLE_OSCE_BWE
#ifndef DISABLE_BBWENET
  write_weights(bbwenetlayers_arrays, fout);
#endif
#endif
  fclose(fout);'''
    
    if old_main not in content:
        raise RuntimeError("Could not find main() block to patch")
    
    content = content.replace(old_main, new_main)
    write_weights_file.write_text(content)
    print("  Applied BWE weights patch")


def get_model_hash(opus_dir):
    """Extract the model hash from autogen.sh."""
    autogen_script = opus_dir / "autogen.sh"
    
    if not autogen_script.exists():
        print("  Warning: autogen.sh not found")
        return None
    
    content = autogen_script.read_text()
    match = re.search(r'dnn/download_model\.sh\s+["\']?([a-fA-F0-9]+)["\']?', content)
    
    if match:
        return match.group(1)
    
    print("  Warning: Could not find model hash in autogen.sh")
    return None


def download_model(opus_dir, model_hash):
    """Download the Opus DNN model using the download script."""
    if not model_hash:
        raise RuntimeError("No model hash provided - cannot download model")
    
    print(f"  Model hash: {model_hash[:16]}...")
    
    if platform.system() == "Windows":
        download_script = opus_dir / "dnn" / "download_model.bat"
        if download_script.exists():
            print(f"  Running download_model.bat...")
            run_command(["cmd", "/c", "dnn\\download_model.bat", model_hash], cwd=opus_dir)
        else:
            raise RuntimeError(f"download_model.bat not found: {download_script}")
    else:
        download_script = opus_dir / "dnn" / "download_model.sh"
        if download_script.exists():
            print(f"  Running download_model.sh...")
            run_command(["chmod", "+x", str(download_script)])
            run_command(["dnn/download_model.sh", model_hash], cwd=opus_dir)
        else:
            raise RuntimeError(f"download_model.sh not found: {download_script}")
    
    print("  Model downloaded successfully")


def find_compiler():
    """Find a suitable C compiler. Returns (compiler_path, compiler_type, vcvarsall_path)."""
    if platform.system() == "Windows":
        # Try cl.exe from PATH (already in VS environment)
        cl_path = shutil.which("cl")
        if cl_path:
            return "cl", "msvc", None
        
        # Try vswhere to find Visual Studio
        vswhere_paths = [
            r"C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe",
            r"C:\Program Files\Microsoft Visual Studio\Installer\vswhere.exe",
        ]
        for vswhere in vswhere_paths:
            if os.path.exists(vswhere):
                try:
                    result = subprocess.run(
                        [vswhere, "-latest", "-products", "*", "-requires",
                         "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
                         "-property", "installationPath"],
                        capture_output=True, text=True
                    )
                    if result.returncode == 0 and result.stdout.strip():
                        vs_path = Path(result.stdout.strip())
                        # Find vcvarsall.bat to set up the environment
                        vcvarsall = vs_path / "VC" / "Auxiliary" / "Build" / "vcvarsall.bat"
                        if vcvarsall.exists():
                            return "cl", "msvc", str(vcvarsall)
                except Exception:
                    pass
        
        # Fallback to gcc/clang
        for compiler in ["clang", "gcc", "cc"]:
            if shutil.which(compiler):
                return compiler, "gcc", None
    else:
        for compiler in ["cc", "gcc", "clang"]:
            if shutil.which(compiler):
                return compiler, "gcc", None
    
    return None, None, None


def compile_write_weights(opus_dir, build_dir):
    """Compile write_lpcnet_weights.c."""
    compiler, compiler_type, vcvarsall = find_compiler()
    
    if not compiler:
        raise RuntimeError(
            "No C compiler found. Please install Visual Studio, GCC, or Clang.\n"
            "On Windows, run from a 'Developer Command Prompt for VS'."
        )
    
    print(f"  Using compiler: {compiler} ({compiler_type})")
    if vcvarsall:
        print(f"  Using vcvarsall: {vcvarsall}")
    build_dir = build_dir.resolve()
    opus_dir = opus_dir.resolve()
    
    # Source files
    source_files = [
        opus_dir / "dnn" / "write_lpcnet_weights.c",
        opus_dir / "dnn" / "parse_lpcnet_weights.c",
    ]
    
    for f in source_files:
        if not f.exists():
            raise RuntimeError(f"Source file not found: {f}")
    
    output_exe = build_dir / ("write_lpcnet_weights.exe" if platform.system() == "Windows" else "write_lpcnet_weights")
    
    include_dirs = [
        opus_dir / "dnn",
        opus_dir / "celt",
        opus_dir / "include",
        opus_dir / "silk",
        opus_dir,
    ]
    
    if compiler_type == "msvc":
        include_flags = [f"/I{d}" for d in include_dirs]
        cl_args = [
            compiler,
            *[str(f) for f in source_files],
            *include_flags,
            "/DENABLE_OSCE",
            "/DENABLE_OSCE_BWE",
            "/DOPUS_BUILD",
            "/D_CRT_SECURE_NO_WARNINGS",
            f"/Fe{output_exe}",
            f"/Fo{build_dir}\\",
            "/nologo",
            "/O2",
        ]
        
        if vcvarsall:
            # Run cl.exe through vcvarsall.bat to set up environment
            cl_cmd_str = ' '.join(f'"{arg}"' if ' ' in str(arg) else str(arg) for arg in cl_args)
            cmd = f'"{vcvarsall}" x64 && {cl_cmd_str}'
            run_command(cmd, cwd=str(build_dir))
        else:
            # cl.exe is already in PATH with proper environment
            run_command(cl_args, cwd=str(build_dir))
    else:
        include_flags = [f"-I{d}" for d in include_dirs]
        cmd = [
            compiler,
            *[str(f) for f in source_files],
            *include_flags,
            "-DENABLE_OSCE",
            "-DENABLE_OSCE_BWE",
            "-DOPUS_BUILD",
            "-o", str(output_exe),
            "-O2",
        ]
        run_command(cmd, cwd=str(build_dir))
    
    if not output_exe.exists():
        raise RuntimeError(f"Compilation failed - {output_exe} not created")
    
    print(f"  Compiled: {output_exe.name}")
    return output_exe


def generate_weights(output_dir):
    """Generate the weights blob file."""
    script_dir = Path(__file__).parent.resolve()
    
    print("=" * 60)
    print("Opus DNN Weights Generator")
    print("=" * 60)
    
    # Get vendored commit
    print("\n[1/9] Reading vendored commit...")
    commit = get_vendored_commit(script_dir)
    if commit:
        print(f"  Vendored commit: {commit[:12]}")
    else:
        print("  OPUS_VERSION not found, will use HEAD")
    
    # Create temp directory
    print("\n[2/9] Creating temporary directory...")
    temp_dir = Path(tempfile.mkdtemp(prefix="opus_weights_"))
    opus_dir = temp_dir / "opus"
    build_dir = temp_dir / "build"
    build_dir.mkdir()
    print(f"  Temp dir: {temp_dir}")
    
    try:
        # Clone opus
        print("\n[3/9] Cloning Opus repository...")
        clone_opus(opus_dir, commit)
        
        # Get model hash and download model
        print("\n[4/9] Downloading DNN model...")
        model_hash = get_model_hash(opus_dir)
        download_model(opus_dir, model_hash)
        
        # Apply binary mode patch
        print("\n[5/9] Applying binary mode patch...")
        apply_binary_mode_patch(opus_dir)
        
        # Apply BWE weights patch
        print("\n[6/9] Applying BWE weights patch...")
        apply_bwe_weights_patch(opus_dir)
        
        # Compile
        print("\n[7/9] Compiling write_lpcnet_weights...")
        exe = compile_write_weights(opus_dir, build_dir)
        
        # Run
        print("\n[8/9] Generating weights blob...")
        run_command([str(exe)], cwd=build_dir)
        
        weights_file = build_dir / "weights_blob.bin"
        if not weights_file.exists():
            raise RuntimeError("weights_blob.bin was not created")
        
        print(f"  Generated: {weights_file.stat().st_size:,} bytes")
        
        # Copy to output
        print("\n[9/9] Copying to output directory...")
        output_dir.mkdir(parents=True, exist_ok=True)
        
        if model_hash:
            bin_name = f"opus_data-{model_hash}.bin"
        else:
            bin_name = "opus_data.bin"
        
        final_path = output_dir / bin_name
        shutil.copy2(weights_file, final_path)
        
        # Generate MD5
        md5_hash = hashlib.md5()
        with open(final_path, 'rb') as f:
            for chunk in iter(lambda: f.read(8192), b''):
                md5_hash.update(chunk)
        
        md5_value = md5_hash.hexdigest()
        md5_path = output_dir / f"{bin_name}.md5"
        md5_path.write_text(f"{md5_value}  {bin_name}\n")
        
        print(f"\n  Output: {final_path}")
        print(f"  Size: {final_path.stat().st_size:,} bytes ({final_path.stat().st_size / 1024 / 1024:.2f} MB)")
        print(f"  MD5: {md5_value}")
        
        return final_path, md5_path
        
    finally:
        # Cleanup
        print(f"\nCleaning up temp directory...")
        shutil.rmtree(temp_dir, ignore_errors=True)


def main():
    parser = argparse.ArgumentParser(
        description="Generate Opus DNN weights blob"
    )
    parser.add_argument(
        "--output", "-o",
        type=Path,
        default=Path(__file__).parent / "target" / "model",
        help="Output directory (default: target/model)"
    )
    
    args = parser.parse_args()
    
    try:
        bin_path, md5_path = generate_weights(args.output)
        print("\n" + "=" * 60)
        print("SUCCESS!")
        print(f"Weights file: {bin_path}")
        print(f"MD5 file: {md5_path}")
        print("=" * 60)
    except Exception as e:
        print(f"\nERROR: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
