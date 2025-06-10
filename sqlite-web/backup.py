#!/usr/bin/env -S uv venv --python 3.9 -- CWD=.venv/bin/python
# /// script
# dependencies = [
#  "cryptography>=40.0.0", # Using a recent version for security best practices
# ]
# ///

import argparse
import json
import subprocess  # No longer needed for dump, but good to keep if other CLI tools were used
import tarfile
import io
import os
import pathlib
import sys
import sqlite3  # Added for direct iterdump

from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import padding
from cryptography.fernet import Fernet


def load_db_paths(json_file_path: pathlib.Path) -> list[pathlib.Path]:
    """Loads database paths from the specified JSON file."""
    if not json_file_path.exists():
        print(
            f"Error: JSON file not found at {json_file_path}", file=sys.stderr
        )
        sys.exit(1)
    try:
        with open(json_file_path, "r") as f:
            data = json.load(f)
        if not isinstance(data, list):
            raise ValueError("JSON content must be a list.")
        paths = []
        for item in data:
            if not isinstance(item, dict) or "path" not in item:
                raise ValueError(
                    "Each item in JSON list must be a dict with a 'path' key."
                )
            db_path_str = item["path"]
            # Ensure path is absolute for consistency, especially if script is run from different dirs
            db_path = pathlib.Path(db_path_str).resolve()
            if not db_path.exists() or not db_path.is_file():
                print(
                    f"Warning: Database file not found or is not a file: {db_path}",
                    file=sys.stderr,
                )
            else:
                paths.append(db_path)
        return paths
    except json.JSONDecodeError:
        print(
            f"Error: Could not decode JSON from {json_file_path}",
            file=sys.stderr,
        )
        sys.exit(1)
    except ValueError as e:
        print(f"Error: Invalid JSON structure: {e}", file=sys.stderr)
        sys.exit(1)


def load_public_rsa_key(key_path: pathlib.Path):
    """Loads an RSA public key from a PEM file."""
    if not key_path.exists():
        print(
            f"Error: RSA public key file not found at {key_path}",
            file=sys.stderr,
        )
        sys.exit(1)
    try:
        with open(key_path, "rb") as key_file:
            public_key = serialization.load_pem_public_key(key_file.read())
        return public_key
    except Exception as e:
        print(f"Error loading RSA public key: {e}", file=sys.stderr)
        sys.exit(1)


def dump_sqlite_database(db_path: pathlib.Path) -> bytes | None:
    """Dumps SQLite database content using sqlite3.Connection.iterdump()."""
    try:
        print(f"Dumping database: {db_path}...")
        # Connect to the SQLite database
        # The database is opened in read-only mode by default for iterdump,
        # but explicitly using URI for read-only is good practice if supported by your sqlite version.
        # However, simple path connection is sufficient for iterdump.
        conn = sqlite3.connect(db_path)
        dump_lines = []
        for line in conn.iterdump():
            dump_lines.append(line)
        conn.close()

        full_dump = "\n".join(dump_lines) + "\n"
        return full_dump.encode("utf-8")  # Ensure it's bytes for encryption
    except sqlite3.Error as e:
        print(
            f"Error dumping database {db_path} using sqlite3.iterdump: {e}",
            file=sys.stderr,
        )
        return None
    except Exception as e:  # Catch any other unexpected errors
        print(
            f"An unexpected error occurred while dumping {db_path}: {e}",
            file=sys.stderr,
        )
        return None


def encrypt_data_aes(data: bytes) -> tuple[bytes, bytes]:
    """Encrypts data using AES (Fernet) and returns encrypted data and the AES key."""
    aes_key = Fernet.generate_key()
    f = Fernet(aes_key)
    encrypted_data = f.encrypt(data)
    return encrypted_data, aes_key


def encrypt_aes_key_rsa(aes_key: bytes, rsa_public_key) -> bytes:
    """Encrypts the AES key using the RSA public key."""
    encrypted_aes_key = rsa_public_key.encrypt(
        aes_key,
        padding.OAEP(
            mgf=padding.MGF1(algorithm=hashes.SHA256()),
            algorithm=hashes.SHA256(),
            label=None,
        ),
    )
    return encrypted_aes_key


def add_to_tar_archive(tar_archive: tarfile.TarFile, data: bytes, arcname: str):
    """Adds byte data to a TAR archive."""
    tarinfo = tarfile.TarInfo(name=arcname)
    tarinfo.size = len(data)
    # Reset modtime for reproducible builds, if desired
    # tarinfo.mtime = 0
    tar_archive.addfile(tarinfo, io.BytesIO(data))
    print(f"  Added {arcname} ({tarinfo.size} bytes) to archive.")


def main():
    parser = argparse.ArgumentParser(
        description="Dumps, encrypts, and archives SQLite databases using Python's sqlite3.iterdump.",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    parser.add_argument(
        "db_list_json",
        type=pathlib.Path,
        help="Path to the JSON file containing a list of database paths.",
    )
    parser.add_argument(
        "public_key_path",
        type=pathlib.Path,
        help="Path to the public RSA key file (PEM format).",
    )
    parser.add_argument(
        "-o",
        "--output",
        type=pathlib.Path,
        default=pathlib.Path("encrypted_backup.tar.gz"),
        help="Path for the output TAR GZ archive.",
    )
    args = parser.parse_args()

    db_paths = load_db_paths(args.db_list_json)
    if not db_paths:
        print(
            "No valid database paths found in the JSON file. Exiting.",
            file=sys.stderr,
        )
        sys.exit(1)

    rsa_public_key = load_public_rsa_key(args.public_key_path)

    output_tar_path = args.output.resolve()
    if output_tar_path.exists():
        print(
            f"Warning: Output file {output_tar_path} already exists and will be overwritten.",
            file=sys.stderr,
        )

    output_tar_path.parent.mkdir(parents=True, exist_ok=True)

    try:
        with tarfile.open(output_tar_path, "w:gz") as tar:
            print(f"Creating archive: {output_tar_path}")
            for db_path in db_paths:
                # Ensure db_path is absolute for sqlite3.connect
                absolute_db_path = db_path.resolve()
                print(
                    f"\nProcessing database: {absolute_db_path.name} (from {absolute_db_path})"
                )

                dump_content = dump_sqlite_database(absolute_db_path)
                if dump_content is None:
                    print(
                        f"Skipping {absolute_db_path.name} due to dump error."
                    )
                    continue

                encrypted_dump, aes_key = encrypt_data_aes(dump_content)
                print(
                    f"  Encrypted dump using AES (key length: {len(aes_key)*8}-bit)."
                )

                encrypted_aes_key = encrypt_aes_key_rsa(aes_key, rsa_public_key)
                print("  Encrypted AES key using RSA public key.")

                base_arcname = (
                    absolute_db_path.name
                )  # Use original name for files in tar
                dump_arcname = f"{base_arcname}.sql.enc"
                key_arcname = f"{base_arcname}.key.enc"

                add_to_tar_archive(tar, encrypted_dump, dump_arcname)
                add_to_tar_archive(tar, encrypted_aes_key, key_arcname)

            manifest_content = {
                "version": "1.1",
                "tool": "Python SQLite Encrypted Backup Script",
                "encryption_info": "Each .sql.enc file is AES encrypted using Fernet. The corresponding .key.enc file contains the RSA-OAEP-SHA256 encrypted AES key.",
                "rsa_key_details": "RSA-OAEP with MGF1 (SHA256) and SHA256 hash for message.",
                "aes_details": "Fernet (AES-128-CBC with PKCS7 padding and HMAC-SHA256 authentication)",
            }
            manifest_bytes = json.dumps(manifest_content, indent=2).encode(
                "utf-8"
            )
            add_to_tar_archive(tar, manifest_bytes, "manifest.json")

        print(f"\nSuccessfully created encrypted archive: {output_tar_path}")

    except Exception as e:
        print(f"\nAn error occurred during archiving: {e}", file=sys.stderr)
        if output_tar_path.exists():
            try:
                output_tar_path.unlink()
                print(f"Removed partially created archive: {output_tar_path}")
            except OSError as oe:
                print(
                    f"Could not remove partial archive {output_tar_path}: {oe}",
                    file=sys.stderr,
                )
        sys.exit(1)


if __name__ == "__main__":
    main()
