#!/usr/bin/env python3
"""Assemble a static, optionally GPG signed apt repository from .deb files.

The result is a plain directory tree that can be served by any web server,
including GitHub Pages:

    <out>/apt/dists/<suite>/Release[.gpg]
    <out>/apt/dists/<suite>/InRelease
    <out>/apt/dists/<suite>/main/binary-<arch>/Packages[.gz|.xz]
    <out>/apt/pool/main/<letter>/<package>/*.deb
    <out>/install.sh
    <out>/index.html

`apt-ftparchive` is intentionally not used: it is not part of a stock runner
image, while python3 always is.

Usage:
    python3 packaging/apt/build-repo.py --debs dist --out pages \\
        [--suite stable] [--component main] [--origin ...] [--label ...] \\
        [--gpg-key-id KEYID] [--gpg-key-file key.asc] [--gpg-passphrase-file pw.txt]
"""

import argparse
import email.utils
import gzip
import hashlib
import lzma
import os
import shutil
import subprocess
import sys
import time

REPO_BASE = "https://borneelphukan.github.io/nixobdo-pdf"
KEYRING_NAME = "nixobdo-pdf-archive-keyring.gpg"

FIELDS = (
    "Package",
    "Version",
    "Architecture",
    "Maintainer",
    "Installed-Size",
    "Depends",
    "Recommends",
    "Suggests",
    "Provides",
    "Conflicts",
    "Breaks",
    "Replaces",
    "Section",
    "Priority",
    "Homepage",
    "Multi-Arch",
    "Essential",
    "Description",
)


def log(message):
    print(f"==> {message}", flush=True)


def die(message):
    print(f"error: {message}", file=sys.stderr)
    raise SystemExit(1)


def parse_control(text):
    fields = {}
    key = None
    for line in text.splitlines():
        if not line.strip():
            continue
        if line[0] in " \t":
            if key is not None:
                fields[key] = fields[key] + "\n" + line
            continue
        if ":" not in line:
            continue
        key, _, value = line.partition(":")
        key = key.strip()
        fields[key] = value.strip()
    return fields


def deb_control(deb_path):
    raw = subprocess.run(
        ["dpkg-deb", "--field", deb_path],
        check=True,
        capture_output=True,
    )
    return parse_control(raw.stdout.decode("utf-8", "replace"))


def sha256_of(path):
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def write_bytes(path, data):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "wb") as handle:
        handle.write(data)


def gpg(args, stdin=None):
    return subprocess.run(
        ["gpg", "--batch", "--yes", *args],
        input=stdin,
        capture_output=True,
    )


def sign_release(dist_dir, key_id, key_file, passphrase_file):
    if key_file:
        log(f"importing GPG key from {key_file}")
        with open(key_file, "rb") as handle:
            result = gpg(["--import"], stdin=handle.read())
        if result.returncode != 0:
            die(f"failed to import GPG key: {result.stderr.decode(errors='replace')}")

    probe = gpg(["--list-secret-keys", "--with-colons", key_id])
    if probe.returncode != 0:
        die(
            f"no secret key {key_id} available; check GPG_PRIVATE_KEY / GPG_KEY_ID "
            "repository secrets"
        )

    passphrase_args = []
    if passphrase_file:
        with open(passphrase_file, "rb") as handle:
            if handle.read().strip():
                passphrase_args = [
                    "--pinentry-mode",
                    "loopback",
                    "--passphrase-file",
                    passphrase_file,
                ]

    release = os.path.join(dist_dir, "Release")
    log("signing Release -> InRelease and Release.gpg")
    for extra, output in (
        (["--clearsign"], "InRelease"),
        (["--detach-sign", "--armor"], "Release.gpg"),
    ):
        result = gpg(
            [
                *passphrase_args,
                "--local-user",
                key_id,
                "--output",
                os.path.join(dist_dir, output),
                *extra,
                release,
            ]
        )
        if result.returncode != 0:
            die(f"signing {output} failed: {result.stderr.decode(errors='replace')}")

    exported = gpg(["--export", key_id])
    if exported.returncode != 0 or not exported.stdout:
        die("exporting the public key failed")
    return exported.stdout


def build_index_html(out_dir, suite, component, archs, packages, signed):
    trusted = "" if signed else "&nbsp;&nbsp;&nbsp;<strong>(unsigned repository)</strong>"
    keyring_line = (
        f"curl -fsSL {REPO_BASE}/{KEYRING_NAME} | sudo tee "
        "/usr/share/keyrings/nixobdo-pdf.gpg >/dev/null"
        if signed
        else "# no signing key: the repository is used with [trusted=yes]"
    )
    sources = f"""Types: deb
URIs: {REPO_BASE}/apt/
Suites: {suite}
Components: {component}
Architectures: {' '.join(archs)}
Signed-By: /usr/share/keyrings/nixobdo-pdf.gpg"""

    downloads = "\n".join(
        f'<a href="{REPO_BASE}/apt/pool/{component}/{pool_letter(pkg)}/{pkg}/">'
        f"apt/pool/{component}/{pool_letter(pkg)}/{pkg}/</a>"
        for pkg in packages
    )

    html = f"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>nixobdo-pdf &mdash; apt repository</title>
<style>
  body {{ font-family: system-ui, -apple-system, "Segoe UI", sans-serif;
         max-width: 46rem; margin: 3rem auto; padding: 0 1.25rem;
         line-height: 1.6; color: #1f2430; }}
  h1 {{ font-size: 1.6rem; }}
  pre {{ background: #f4f5f9; border: 1px solid #dcdfe8; border-radius: 8px;
         padding: 0.9rem 1rem; overflow-x: auto; font-size: 0.9rem; }}
  code {{ font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }}
  .muted {{ color: #5b6478; font-size: 0.92rem; }}
  a {{ color: #4f46e5; }}
</style>
</head>
<body>
<h1>nixobdo-pdf apt repository{trusted}</h1>
<p>Suite <code>{suite}</code>, component <code>{component}</code>,
architectures <code>{' '.join(archs)}</code>.</p>

<h2>Install</h2>
<pre><code>curl -fsSL {REPO_BASE}/install.sh | sudo bash</code></pre>

<p>Or manually:</p>
<pre><code>{keyring_line}
sudo tee /etc/apt/sources.list.d/nixobdo-pdf.sources &gt;/dev/null &lt;&lt;'EOF'
{sources}
EOF
sudo apt update
sudo apt install nixobdo-pdf</code></pre>

<p class="muted">Direct download:
{downloads}</p>
</body>
</html>
"""
    write_bytes(os.path.join(out_dir, "index.html"), html.encode("utf-8"))


def pool_letter(package):
    return package[0].lower() if package[0].isalpha() else "lib" + package[0]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--debs", required=True, help="directory containing .deb files")
    parser.add_argument("--out", required=True, help="output directory to serve")
    parser.add_argument("--suite", default="stable")
    parser.add_argument("--codename", default=None, help="defaults to --suite")
    parser.add_argument("--component", default="main")
    parser.add_argument(
        "--origin", default="nixobdo-pdf", help="repository origin label"
    )
    parser.add_argument("--label", default="nixobdo-pdf")
    parser.add_argument(
        "--description", default="nixobdo-pdf packages for Debian and Ubuntu"
    )
    parser.add_argument("--gpg-key-id", default=None)
    parser.add_argument("--gpg-key-file", default=None)
    parser.add_argument("--gpg-passphrase-file", default=None)
    parser.add_argument(
        "--install-script",
        default=os.path.join(os.path.dirname(os.path.abspath(__file__)), "install.sh"),
    )
    args = parser.parse_args()

    debs = sorted(
        os.path.join(args.debs, name)
        for name in os.listdir(args.debs)
        if name.endswith(".deb")
    )
    if not debs:
        die(f"no .deb files found in {args.debs}")

    out_dir = os.path.abspath(args.out)
    apt_dir = os.path.join(out_dir, "apt")
    dist_dir = os.path.join(apt_dir, "dists", args.suite)
    if os.path.isdir(apt_dir):
        shutil.rmtree(apt_dir)
    os.makedirs(dist_dir, exist_ok=True)

    archs = []
    packages = []
    for deb in debs:
        control = deb_control(deb)
        package = control.get("Package")
        arch = control.get("Architecture", "amd64")
        if not package:
            die(f"missing Package field in {deb}")
        if arch not in archs:
            archs.append(arch)
        if package not in packages:
            packages.append(package)

        letter = pool_letter(package)
        pool_dir = os.path.join(apt_dir, "pool", args.component, letter, package)
        os.makedirs(pool_dir, exist_ok=True)
        target = os.path.join(pool_dir, os.path.basename(deb))
        shutil.copy2(deb, target)
        relative = os.path.relpath(target, apt_dir)

        binary_dir = os.path.join(dist_dir, args.component, f"binary-{arch}")
        os.makedirs(binary_dir, exist_ok=True)
        packages_path = os.path.join(binary_dir, "Packages")

        with open(packages_path, "a", encoding="utf-8") as handle:
            for field in FIELDS:
                value = control.get(field)
                if value is None:
                    continue
                if field == "Description":
                    handle.write(f"Description:{value}\n")
                else:
                    handle.write(f"{field}: {value}\n")
            handle.write(f"Filename: {relative}\n")
            handle.write(f"Size: {os.path.getsize(target)}\n")
            handle.write(f"Sha256: {sha256_of(target)}\n")
            handle.write("\n")
        log(f"indexed {package} {control.get('Version')} ({arch})")

    # The uncompressed Packages file is kept on purpose: apt only resolves a
    # binary-<arch>/Packages target once an entry for it exists in Release, and
    # it then downloads whichever compressed variant it prefers.
    for arch in archs:
        binary_dir = os.path.join(dist_dir, args.component, f"binary-{arch}")
        packages_path = os.path.join(binary_dir, "Packages")
        with open(packages_path, "rb") as handle:
            raw = handle.read()
        write_bytes(packages_path + ".gz", gzip.compress(raw, 9))
        write_bytes(
            packages_path + ".xz",
            lzma.compress(raw, preset=6, check=lzma.CHECK_CRC64),
        )
        log(f"compressed Packages for {arch}")

    signed = False
    if args.gpg_key_id:
        signed = True
    else:
        log("no GPG key id given: publishing an unsigned repository")
        write_bytes(os.path.join(out_dir, "UNSIGNED"), b"unsigned\n")

    release_lines = [
        f"Origin: {args.origin}",
        f"Label: {args.label}",
        f"Suite: {args.suite}",
        f"Codename: {args.codename or args.suite}",
        f"Components: {args.component}",
        f"Architectures: {' '.join(archs)}",
        f"Description: {args.description}",
        f"Date: {email.utils.formatdate(time.time(), usegmt=True)}",
    ]
    if not signed:
        release_lines.insert(2, "Acquire-By-Hash: no")

    checksums = []
    for root, _dirs, names in os.walk(dist_dir):
        for name in sorted(names):
            if name in ("Release", "Release.gpg", "InRelease"):
                continue
            path = os.path.join(root, name)
            checksums.append(
                (
                    sha256_of(path),
                    os.path.getsize(path),
                    os.path.relpath(path, dist_dir),
                )
            )

    release_lines.append("SHA256:")
    for digest, size, relative in sorted(checksums, key=lambda item: item[2]):
        release_lines.append(f" {digest} {size} {relative}")

    release_text = "\n".join(release_lines) + "\n"
    write_bytes(os.path.join(dist_dir, "Release"), release_text.encode("utf-8"))
    log(f"wrote Release ({len(checksums)} checksummed files)")

    if signed:
        public_key = sign_release(
            dist_dir, args.gpg_key_id, args.gpg_key_file, args.gpg_passphrase_file
        )
        write_bytes(os.path.join(out_dir, KEYRING_NAME), public_key)

    if os.path.isfile(args.install_script):
        shutil.copy2(args.install_script, os.path.join(out_dir, "install.sh"))
        os.chmod(os.path.join(out_dir, "install.sh"), 0o755)
    else:
        die(f"install script not found: {args.install_script}")

    build_index_html(out_dir, args.suite, args.component, archs, packages, signed)
    log(f"repository ready in {out_dir} (signed={signed})")


if __name__ == "__main__":
    main()
