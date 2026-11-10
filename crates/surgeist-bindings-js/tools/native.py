"""Validate the pinned native SDK artifact before an offline Cargo invocation.

Default operation never downloads. --download explicitly acquires the one
qualified upstream artifact into --archive. Run under the repository's process
supervisor; the requested command replaces this process and inherits its lifetime.
"""

import argparse
import gzip
import hashlib
import json
import os
import platform
from pathlib import Path
import subprocess
import tempfile
import sys


VERSION = "152.2.0"
TARGET = "aarch64-apple-darwin"
ARCHIVE_SHA256 = "680617c57021c1ad7490749e4c10c4504978e84c3e3a288b8bb39a31a9e0d1f8"
ARCHIVE_SIZE = 157_813_600
COMPRESSED_SHA256 = "a4a5484444ad4b5f10dc7bed141236dd9dbeabf2006d14520f19b935d4db4227"
URL = (
    "https://github.com/denoland/rusty_v8/releases/download/"
    f"v{VERSION}/librusty_v8_release_{TARGET}.a.gz"
)


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def validate(path):
    if path.stat().st_size != ARCHIVE_SIZE or digest(path) != ARCHIVE_SHA256:
        raise SystemExit("The native archive does not match the qualified artifact")


def acquire(path):
    path.parent.mkdir(parents=True, exist_ok=True)
    # The temporary directory is owned by this invocation, on the destination
    # filesystem so publication of the verified archive can be atomic.
    with tempfile.TemporaryDirectory(prefix=".v8-download-", dir=path.parent) as temporary:
        compressed = Path(temporary) / "archive.gz"
        subprocess.run(
            ["curl", "--fail", "--location", "--max-time", "180", "--max-filesize", "40635999", "--output",
             str(compressed), URL],
            check=True,
        )
        if digest(compressed) != COMPRESSED_SHA256:
            raise SystemExit("The download does not match the published release digest")
        decoded = Path(temporary) / "archive.a"
        size = 0
        with gzip.open(compressed, "rb") as source, decoded.open("xb") as destination:
            while chunk := source.read(1024 * 1024):
                size += len(chunk)
                if size > ARCHIVE_SIZE:
                    raise SystemExit("The decoded archive exceeds the qualified size")
                destination.write(chunk)
        validate(decoded)
        # Never overwrite another invocation's output. A concurrent successful
        # acquisition is acceptable only if it is the same qualified artifact.
        try:
            os.link(decoded, path)
        except FileExistsError:
            validate(path)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", required=True, type=Path)
    parser.add_argument("--download", action="store_true")
    parser.add_argument("command", nargs=argparse.REMAINDER)
    arguments = parser.parse_args()
    path = arguments.archive.expanduser().resolve()
    if not path.exists():
        if not arguments.download:
            raise SystemExit("Native archive missing; explicit --download is required")
        acquire(path)
    validate(path)
    environment = {
        "RUSTY_V8_ARCHIVE": str(path),
        "RUSTY_V8_ARCHIVE_SHA256": ARCHIVE_SHA256,
    }
    command = arguments.command
    if command[:1] == ["--"]:
        command = command[1:]
    if command:
        # Prevent an accidental cross-target build using this qualified archive.
        target = os.environ.get("CARGO_BUILD_TARGET", TARGET)
        if sys.platform != "darwin" or platform.machine() != "arm64" or target != TARGET or any(item == "--target" or item.startswith("--target=") for item in command):
            raise SystemExit("This artifact runner qualifies only its native target")
        os.execvpe(command[0], command, {**os.environ, **environment})
    print(json.dumps({"version": VERSION, "target": TARGET, **environment}))


if __name__ == "__main__":
    main()
