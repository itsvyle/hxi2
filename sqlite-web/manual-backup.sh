#!/bin/sh
# This script is used to backup the sqlite-web database.
set -e
if [ -z "$SQLITE_BACKUP_PUBLIC_KEY" ]; then
    echo "Error: SQLITE_BACKUP_PUBLIC_KEY is not set."
    exit 1
fi

if [ -z "$SQLITE_BACKUP_OUTPUT_DIR" ]; then
    echo "Error: SQLITE_BACKUP_OUTPUT_DIR is not set."
    exit 1
fi

# Reproduce this:
# outputFile := fmt.Sprintf("backup_%s.tar", time.Now().Format("20060102_150405"))
# outputPath := filepath.Join(BackupOutputDirectory, outputFile)

# cmd := exec.Command("uv", "run", "backup.py", "-o", outputPath, ConfigPath, BackupPublicKeyPath)
outputFile="backup_$(date +%Y%m%d_%H%M%S).tar"
outputPath="$SQLITE_BACKUP_OUTPUT_DIR/$outputFile"

uv run /app/backup.py -o "$outputPath" "$SQLITE_WEB_CONFIG_PATH" "$SQLITE_BACKUP_PUBLIC_KEY"
