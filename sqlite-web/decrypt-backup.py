#!/usr/bin/env -S uv venv --python 3.9 -- CWD=.venv/bin/python
# /// script
# dependencies = [
#  "cryptography>=40.0.0",
# ]
# ///

import argparse
import tarfile
import io
import os
import pathlib
import sys
import json
import getpass  # For password-protected private keys

from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import padding
from cryptography.fernet import Fernet, InvalidToken
from cryptography.hazmat.primitives.serialization import ssh  # For OpenSSH keys
from cryptography.hazmat.backends import default_backend


def load_private_rsa_key(key_path: pathlib.Path, password: str | None = None):
    """
    Loads an RSA private key from a PEM file.
    Handles both traditional PEM and OpenSSH private key formats.
    Prompts for a password if the key is encrypted and no password is provided.
    """
    if not key_path.exists():
        print(
            f"Error: RSA private key file not found at {key_path}",
            file=sys.stderr,
        )
        sys.exit(1)

    key_bytes = key_path.read_bytes()
    loaded_key = None

    # Attempt to load as traditional PEM (PKCS1 or PKCS8)
    try:
        loaded_key = serialization.load_pem_private_key(
            key_bytes,
            password=password.encode() if password else None,
            backend=default_backend(),
        )
        print("Successfully loaded private key (PEM format).")
        return loaded_key
    except (TypeError, ValueError) as e:
        # TypeError if password needed but not given, ValueError for bad format/password
        pem_load_error = e
    except Exception as e:  # Catch other potential cryptography errors
        pem_load_error = e
        print(f"Unexpected error loading PEM key: {e}", file=sys.stderr)

    # Attempt to load as OpenSSH private key format
    try:
        loaded_key = serialization.load_ssh_private_key(
            key_bytes,
            password=password.encode() if password else None,
            backend=default_backend(),
        )
        print("Successfully loaded private key (OpenSSH format).")
        return loaded_key
    except (TypeError, ValueError) as e:
        ssh_load_error = e
    except Exception as e:
        ssh_load_error = e
        print(f"Unexpected error loading SSH key: {e}", file=sys.stderr)

    # If both failed, report errors
    print(
        f"Error: Could not load private key from {key_path}.", file=sys.stderr
    )
    if "pem_load_error" in locals() and pem_load_error:
        print(f"  PEM load attempt failed: {pem_load_error}", file=sys.stderr)
        if "MAC check failed" in str(pem_load_error) or "Bad decrypt" in str(
            pem_load_error
        ):
            print(
                "    This might indicate an incorrect password for an encrypted PEM key.",
                file=sys.stderr,
            )
        elif (
            "Could not deserialize key data" in str(pem_load_error)
            and password is None
        ):
            print(
                "    The key might be password-protected. Try providing a password.",
                file=sys.stderr,
            )
    if "ssh_load_error" in locals() and ssh_load_error:
        print(
            f"  OpenSSH load attempt failed: {ssh_load_error}", file=sys.stderr
        )
        if "Incorrect passphrase" in str(
            ssh_load_error
        ) or "Bad passphrase" in str(ssh_load_error):
            print(
                "    This might indicate an incorrect password for an encrypted OpenSSH key.",
                file=sys.stderr,
            )
        elif (
            "Key is encrypted, but no password" in str(ssh_load_error)
            and password is None
        ):
            print(
                "    The key might be password-protected. Try providing a password.",
                file=sys.stderr,
            )

    # If password was not provided and key might be encrypted, prompt
    if password is None and (
        (
            isinstance(pem_load_error, TypeError)
            and "password was not given" in str(pem_load_error).lower()
        )
        or (
            isinstance(ssh_load_error, TypeError)
            and "password was not given" in str(ssh_load_error).lower()
        )
        or (
            "Could not deserialize key data" in str(pem_load_error)
            if "pem_load_error" in locals()
            else False
        )
        or (
            "Key is encrypted, but no password" in str(ssh_load_error)
            if "ssh_load_error" in locals()
            else False
        )
    ):
        print(f"The private key at {key_path} might be password-protected.")
        try:
            new_password = getpass.getpass(
                "Enter private key password (leave blank if none): "
            )
            if new_password:
                return load_private_rsa_key(
                    key_path, new_password
                )  # Recursive call with password
            else:  # User chose not to enter password, proceed to exit
                pass
        except Exception as e:
            print(f"Error reading password: {e}", file=sys.stderr)

    sys.exit(1)


def decrypt_aes_key_rsa(
    encrypted_aes_key: bytes, rsa_private_key
) -> bytes | None:
    """Decrypts the AES key using the RSA private key."""
    try:
        aes_key = rsa_private_key.decrypt(
            encrypted_aes_key,
            padding.OAEP(
                mgf=padding.MGF1(algorithm=hashes.SHA256()),
                algorithm=hashes.SHA256(),
                label=None,
            ),
        )
        return aes_key
    except Exception as e:
        print(f"Error decrypting AES key with RSA: {e}", file=sys.stderr)
        print(
            "  This could be due to a wrong private key or corrupted key data.",
            file=sys.stderr,
        )
        return None


def decrypt_data_aes(encrypted_data: bytes, aes_key: bytes) -> bytes | None:
    """Decrypts data using AES (Fernet) and the provided AES key."""
    try:
        f = Fernet(aes_key)
        decrypted_data = f.decrypt(encrypted_data)
        return decrypted_data
    except InvalidToken:
        print(
            "Error: AES decryption failed. The data may be corrupted, or the AES key is incorrect (InvalidToken).",
            file=sys.stderr,
        )
        return None
    except Exception as e:
        print(f"Error during AES decryption: {e}", file=sys.stderr)
        return None


def main():
    parser = argparse.ArgumentParser(
        description="Decrypts and extracts SQLite database dumps from an encrypted TAR archive.",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    parser.add_argument(
        "archive_path",
        type=pathlib.Path,
        help="Path to the encrypted TAR GZ archive.",
    )
    parser.add_argument(
        "private_key_path",
        type=pathlib.Path,
        help="Path to the RSA private key file (PEM or OpenSSH format).",
    )
    parser.add_argument(
        "-o",
        "--output-dir",
        type=pathlib.Path,
        default=pathlib.Path("decrypted_dumps"),
        help="Directory to save the decrypted SQL dumps.",
    )
    parser.add_argument(
        "-p",
        "--password",
        type=str,
        default=None,
        help="Password for the RSA private key, if it's encrypted. If not provided, and key is encrypted, you'll be prompted.",
    )
    args = parser.parse_args()

    if not args.archive_path.exists() or not args.archive_path.is_file():
        print(
            f"Error: Archive file not found or is not a file: {args.archive_path}",
            file=sys.stderr,
        )
        sys.exit(1)

    rsa_private_key = load_private_rsa_key(args.private_key_path, args.password)

    args.output_dir.mkdir(parents=True, exist_ok=True)
    print(f"Decrypted files will be saved to: {args.output_dir.resolve()}")

    encrypted_files_data = {}  # To store {'base_name': {'key_data': bytes, 'sql_data': bytes}}

    try:
        with tarfile.open(args.archive_path, "r:gz") as tar:
            print(f"\nProcessing archive: {args.archive_path}")
            for member in tar:
                if not member.isfile():
                    continue

                arcname = pathlib.Path(member.name)
                file_content = tar.extractfile(member)
                if file_content is None:
                    print(
                        f"Warning: Could not extract file {arcname.name} from archive.",
                        file=sys.stderr,
                    )
                    continue
                file_content = file_content.read()

                if arcname.name == "manifest.json":
                    try:
                        manifest = json.loads(file_content.decode("utf-8"))
                        print("  Found manifest:")
                        print(json.dumps(manifest, indent=4, sort_keys=True))
                    except Exception as e:
                        print(f"  Warning: Could not parse manifest.json: {e}")
                    continue

                # Use stem to get name without final suffix, then again for potential .sql or .key
                # e.g., db.sqlite3.sql.enc -> db.sqlite3.sql -> db.sqlite3
                # e.g., db.key.enc -> db.key -> db
                # We need a consistent base name. Let's assume original name was `dbname.sqlite3`
                # and it became `dbname.sqlite3.sql.enc` or `dbname.sqlite3.key.enc`

                # A simpler way is to remove known extensions
                base_name_str = arcname.name
                is_key_file = False
                is_sql_file = False

                if base_name_str.endswith(".key.enc"):
                    base_name_str = base_name_str[: -len(".key.enc")]
                    is_key_file = True
                elif base_name_str.endswith(".sql.enc"):
                    base_name_str = base_name_str[: -len(".sql.enc")]
                    is_sql_file = True
                else:
                    print(
                        f"  Skipping unrecognized file in archive: {arcname.name}"
                    )
                    continue

                if base_name_str not in encrypted_files_data:
                    encrypted_files_data[base_name_str] = {}

                if is_key_file:
                    encrypted_files_data[base_name_str]["key_data"] = (
                        file_content
                    )
                    encrypted_files_data[base_name_str]["key_arcname"] = (
                        arcname.name
                    )
                elif is_sql_file:
                    encrypted_files_data[base_name_str]["sql_data"] = (
                        file_content
                    )
                    encrypted_files_data[base_name_str]["sql_arcname"] = (
                        arcname.name
                    )

            # Now process the collected file data
            if not encrypted_files_data:
                print(
                    "No encrypted database files found in the archive.",
                    file=sys.stderr,
                )
                sys.exit(1)

            decryption_successful_count = 0
            for base_name, data_parts in encrypted_files_data.items():
                print(f"\nAttempting to decrypt for base name: {base_name}")

                if "key_data" not in data_parts:
                    print(
                        f"  Error: Missing encrypted AES key file for '{base_name}' (expected {data_parts.get('sql_arcname', base_name + '.sql.enc')}'s corresponding key file).",
                        file=sys.stderr,
                    )
                    continue
                if "sql_data" not in data_parts:
                    print(
                        f"  Error: Missing encrypted SQL data file for '{base_name}' (expected {data_parts.get('key_arcname', base_name + '.key.enc')}'s corresponding data file).",
                        file=sys.stderr,
                    )
                    continue

                encrypted_aes_key = data_parts["key_data"]
                encrypted_sql_dump = data_parts["sql_data"]

                print(
                    f"  Decrypting AES key from {data_parts['key_arcname']}..."
                )
                aes_key = decrypt_aes_key_rsa(
                    encrypted_aes_key, rsa_private_key
                )
                if aes_key is None:
                    print(
                        f"  Failed to decrypt AES key for {base_name}. Skipping this database.",
                        file=sys.stderr,
                    )
                    continue
                print(f"  AES key decrypted (length: {len(aes_key)*8}-bit).")

                print(
                    f"  Decrypting SQL dump from {data_parts['sql_arcname']}..."
                )
                decrypted_sql_dump = decrypt_data_aes(
                    encrypted_sql_dump, aes_key
                )
                if decrypted_sql_dump is None:
                    print(
                        f"  Failed to decrypt SQL dump for {base_name}. Skipping this database.",
                        file=sys.stderr,
                    )
                    continue
                print(f"  SQL dump decrypted successfully.")

                output_sql_path = args.output_dir / f"{base_name}.sql"
                try:
                    with open(
                        output_sql_path, "wb"
                    ) as f_out:  # Write as bytes (UTF-8 from dump)
                        f_out.write(decrypted_sql_dump)
                    print(
                        f"  Successfully decrypted and saved: {output_sql_path}"
                    )
                    decryption_successful_count += 1
                except IOError as e:
                    print(
                        f"  Error writing decrypted file {output_sql_path}: {e}",
                        file=sys.stderr,
                    )

            if decryption_successful_count > 0:
                print(
                    f"\nDecryption complete. {decryption_successful_count} database(s) successfully decrypted to {args.output_dir.resolve()}"
                )
            else:
                print(
                    "\nDecryption finished, but no databases were successfully decrypted.",
                    file=sys.stderr,
                )

    except tarfile.TarError as e:
        print(
            f"Error reading TAR archive {args.archive_path}: {e}",
            file=sys.stderr,
        )
        sys.exit(1)
    except Exception as e:
        print(f"An unexpected error occurred: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
