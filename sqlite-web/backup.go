package main

import (
	"fmt"
	"io"
	"log"
	"log/slog"
	"os"
	"os/exec"
	"path/filepath"
	"time"
)

func DoBackup() error {
	if BackupPublicKeyPath == "" {
		slog.Error("BackupPublicKey is not set, cannot perform backup")
		return fmt.Errorf("BackupPublicKey is not set, cannot perform backup")
	}
	if BackupOutputDirectory == "" {
		slog.Error("BackupOutputDirectory is not set, cannot perform backup")
		return fmt.Errorf("BackupOutputDirectory is not set, cannot perform backup")
	}
	outputFile := fmt.Sprintf("backup_%s.tar", time.Now().Format("20060102_150405"))
	outputPath := filepath.Join(BackupOutputDirectory, outputFile)

	cmd := exec.Command("uv", "run", "backup.py", "-o", outputPath, ConfigPath, BackupPublicKeyPath)

	stdoutPipe, err := cmd.StdoutPipe()
	if err != nil {
		slog.Error("Failed to create stdout pipe", "error", err)
		return err
	}

	stderrPipe, err := cmd.StderrPipe()
	if err != nil {
		slog.Error("Failed to create stderr pipe", "error", err)
		return err
	}

	if err := cmd.Start(); err != nil {
		slog.Error("Failed to start backup command", "error", err)
		return err
	}

	copyOutput := func(r io.ReadCloser, logger *log.Logger) {
		buf := make([]byte, 1024)
		for {
			n, err := r.Read(buf)
			if n > 0 {
				logger.Printf("%s", buf[:n])
			}
			if err == io.EOF {
				break
			}
			if err != nil {
				slog.Error("Error reading from pipe", "error", err)
				break
			}
		}
	}

	stdoutLogger := log.New(os.Stdout, "[backup] ", log.LstdFlags)
	stderrLogger := log.New(os.Stderr, "[backup] ", log.LstdFlags)

	go copyOutput(stdoutPipe, stdoutLogger)
	go copyOutput(stderrPipe, stderrLogger)

	err = cmd.Wait()

	if err != nil {
		slog.Error("Failed to execute backup command", "error", err)
		return fmt.Errorf("backup command failed: %w", err)
	}

	slog.Info("Backup completed successfully", "outputPath", outputPath)
	return nil
}
