#!/usr/bin/env python3
"""
create_updater.py

Builds the Tauri application, finds the produced installer and its .sig file,
uploads them to a Supabase Storage bucket (if they don't already exist), and
generates a Tauri v2 compatible `updater.json` manifest that contains the
signature and a public URL to the uploaded installer.

Usage: python create_updater.py [--no-build] [--bucket BUCKET] [--notes "..."]

Environment variables (or a `.env` file in the repo root):
  SUPABASE_URL   - base URL of your Supabase instance (e.g. https://db.example.com)
  SUPABASE_KEY   - service role key (required for uploads)
  SUPABASE_BUCKET- (optional) defaults to `binaries`

This script requires the `requests` package. Install with:
  pip install -r requirements.txt
"""

from __future__ import annotations

import argparse
import datetime
import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path


def load_dotenv(path: Path) -> dict:
    env = {}
    if not path.exists():
        return env
    for raw in path.read_text(encoding="utf8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if "=" not in line:
            continue
        k, v = line.split("=", 1)
        v = v.strip().strip('"').strip("'")
        env[k.strip()] = v
    return env


def write_env_placeholder(path: Path) -> None:
    template = (
        "# Supabase configuration - replace the placeholders with real values\n"
        "SUPABASE_URL=https://your-supabase.example.com\n"
        "SUPABASE_KEY=your-service-role-key\n"
        "SUPABASE_BUCKET=binaries\n"
    )
    path.write_text(template, encoding="utf8")
    print(f"Wrote placeholder .env -> {path} (fill values and re-run)")


def run_build(repo_root: Path) -> None:
    print("Running: npm run tauri -- build (this can take a while)")
    subprocess.run(["npm", "run", "tauri", "--", "build"], cwd=repo_root, check=True)


def find_latest_installer(repo_root: Path) -> Path | None:
    bundle = repo_root / "src-tauri" / "target" / "release" / "bundle"
    if not bundle.exists():
        return None
    exts = [".msi", ".exe", ".dmg", ".AppImage", ".zip", ".tar.gz"]
    candidates = []
    for ext in exts:
        # glob patterns for nested directories
        candidates += list(bundle.rglob(f"*{ext}"))
    if not candidates:
        return None
    latest = max(candidates, key=lambda p: p.stat().st_mtime)
    return latest


def find_sig_file(installer: Path) -> Path | None:
    # Common signature filename produced by `tauri sign` is installer + ".sig"
    sig1 = installer.with_name(installer.name + ".sig")
    if sig1.exists():
        return sig1
    # also try installer + ".sig" by appending after suffix (some tools do this)
    sig2 = installer.with_suffix(installer.suffix + ".sig")
    if sig2.exists():
        return sig2
    # search for any .sig that starts with the installer stem
    for f in installer.parent.glob(f"{installer.stem}*.sig"):
        return f
    return None


def parse_version_from_filename(name: str) -> str:
    m = re.search(r"(\d+\.\d+(?:\.\d+)*)", name)
    return m.group(1) if m else name


def platform_identifier_for_filename(name: str, ext: str) -> str:
    name_lower = name.lower()
    if ext in (".msi", ".exe"):
        if re.search(r"(x64|x86_64|amd64|x86-64)", name_lower):
            return "windows-x86_64"
        return "windows"
    if ext in (".dmg",):
        if re.search(r"(arm|aarch64|arm64)", name_lower):
            return "darwin-aarch64"
        return "darwin-x86_64"
    return "unknown"


def read_signature_text(sig_path: Path) -> str:
    # signature files are typically small textual blobs
    return sig_path.read_text(encoding="utf8", errors="replace").strip()


def ensure_requests_installed() -> None:
    try:
        import requests  # type: ignore
    except Exception:
        print("Missing dependency: requests")
        print("Install with: pip install -r requirements.txt")
        sys.exit(1)


def supabase_object_exists(supabase_url: str, supabase_key: str, bucket: str, path: str) -> bool:
    import requests
    from urllib.parse import quote

    # URL-encode the object path
    path_enc = quote(path, safe="")

    headers = {"Authorization": f"Bearer {supabase_key}", "apikey": supabase_key}

    # Primary endpoint (Supabase hosted style)
    urls_to_try = [
        f"{supabase_url.rstrip('/')}/storage/v1/object/info/{bucket}/{path_enc}",
        # Fallback for self-hosted setups or older endpoints
        f"{supabase_url.rstrip('/')}/object/info/{bucket}/{path_enc}",
    ]

    last_exc = None
    for url in urls_to_try:
        try:
            r = requests.get(url, headers=headers, timeout=15)
        except Exception as e:
            last_exc = e
            # try next URL
            continue

        if r.status_code == 200:
            return True
        if r.status_code == 404:
            return False

        # If we get 400/other from this endpoint, capture body for debugging and try fallback
        try:
            body = r.text
        except Exception:
            body = "<no body>"
        print(f"supabase_object_exists: request to {url} returned status {r.status_code}: {body}")

    if last_exc:
        raise last_exc
    # Fallback: try listing objects under the bucket with the prefix (POST list)
    try:
        list_url = f"{supabase_url.rstrip('/')}/storage/v1/object/list/{bucket}"
        body = {"prefix": path}
        r = requests.post(list_url, headers=headers, json=body, timeout=15)
        if r.status_code == 200:
            try:
                items = r.json()
                # items is usually a list of objects
                for it in items:
                    if it.get("name") == path or it.get("name") == path:
                        return True
                return False
            except Exception:
                return False
        else:
            print(f"supabase_object_exists list fallback returned {r.status_code}: {r.text}")
    except Exception as e:
        print(f"supabase_object_exists list fallback exception: {e}")

    return False


def supabase_upload_file(supabase_url: str, supabase_key: str, bucket: str, path: str, file_path: Path) -> dict:
    import requests
    from urllib.parse import quote

    path_enc = quote(path, safe="")
    url = f"{supabase_url.rstrip('/')}/storage/v1/object/{bucket}/{path_enc}"
    headers = {
        "Authorization": f"Bearer {supabase_key}",
        "apikey": supabase_key,
        "Content-Type": "application/octet-stream",
    }
    with file_path.open("rb") as fh:
        # POST with raw bytes as body is accepted by Supabase storage endpoint
        r = requests.post(url, headers=headers, data=fh, timeout=60)
    try:
        return {"status_code": r.status_code, "json": r.json()}
    except Exception:
        return {"status_code": r.status_code, "text": r.text}


def public_url_for_object(supabase_url: str, bucket: str, path: str) -> str:
    # For Supabase storage the public URL pattern is: /storage/v1/object/public/{bucket}/{path}
    return f"{supabase_url.rstrip('/')}/storage/v1/object/public/{bucket}/{path}"


def write_updater_json(repo_root: Path, version: str, notes: str, pub_date: str, platform: str, signature: str, url: str) -> Path:
    manifest = {
        "version": version,
        "notes": notes,
        "pub_date": pub_date,
        "platforms": {platform: {"signature": signature, "url": url}},
    }
    out = repo_root / "public" / "updater.json"
    out.write_text(json.dumps(manifest, indent=2), encoding="utf8")
    return out


def main() -> int:
    parser = argparse.ArgumentParser(description="Build, upload Tauri installer to Supabase and create updater.json")
    parser.add_argument("--no-build", action="store_true", help="Skip running the build step")
    parser.add_argument("--bucket", default=None, help="Supabase storage bucket to use (default: from env or 'binaries')")
    parser.add_argument("--notes", default="", help="Release notes to include in updater.json")
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parent

    # load .env if present
    env = load_dotenv(repo_root / ".env") if (repo_root / ".env").exists() else {}

    supabase_url = os.environ.get("SUPABASE_URL") or env.get("SUPABASE_URL")
    supabase_key = os.environ.get("SUPABASE_KEY") or env.get("SUPABASE_KEY")
    bucket = args.bucket or os.environ.get("SUPABASE_BUCKET") or env.get("SUPABASE_BUCKET") or "binaries"

    if not supabase_url or not supabase_key:
        # create placeholder .env if missing
        if not (repo_root / ".env").exists():
            write_env_placeholder(repo_root / ".env")
        print("ERROR: SUPABASE_URL and SUPABASE_KEY must be set in environment or .env.\nFill in .env or export environment variables and re-run.")
        return 2

    ensure_requests_installed()

    # Build
    if not args.no_build:
        try:
            run_build(repo_root)
        except subprocess.CalledProcessError as e:
            print("Build failed:", e)
            return 3

    installer = find_latest_installer(repo_root)
    if not installer:
        print("No installer found in src-tauri/target/release/bundle. Are you sure the build produced an installer?")
        return 4

    sig = find_sig_file(installer)
    if not sig:
        print(f"No signature file found for {installer.name} (tried {installer.name + '.sig'})")
        return 5

    version = parse_version_from_filename(installer.name)
    platform = platform_identifier_for_filename(installer.name, installer.suffix)
    signature = read_signature_text(sig)

    print(f"Found installer: {installer}\nFound signature: {sig}\nVersion: {version}\nPlatform: {platform}")

    # Upload files if missing
    for f in (installer, sig):
        key = f.name
        try:
            exists = supabase_object_exists(supabase_url, supabase_key, bucket, key)
        except Exception as e:
            print(f"Failed to check object existence for {key}: {e}")
            return 6

        if exists:
            print(f"{key} already exists in bucket '{bucket}' - skipping upload")
        else:
            print(f"Uploading {key} to bucket '{bucket}'...")
            resp = supabase_upload_file(supabase_url, supabase_key, bucket, key, f)
            status = resp.get("status_code")
            if status not in (200, 201):
                print(f"Upload failed for {key}: {resp}")
                return 7
            print(f"Uploaded {key}")

    public_url = public_url_for_object(supabase_url, bucket, installer.name)
    pub_date = datetime.datetime.utcnow().replace(microsecond=0).isoformat() + "Z"

    manifest_path = write_updater_json(repo_root, version, args.notes, pub_date, platform, signature, public_url)
    print(f"Wrote updater manifest: {manifest_path}")

    # Do not copy artifacts to the repository root (user requested).
    # The installer and signature remain in the build bundle directory.
    print(f"Installer remains at: {installer}")
    print(f"Signature remains at: {sig}")

    print("Done. You can now deploy the updater.json and installer to your hosting or use the Supabase public URL above.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
