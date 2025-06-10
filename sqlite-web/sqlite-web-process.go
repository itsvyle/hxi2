package main

import (
	"log/slog"
	"os"
	"os/exec"
	"time"
)

type SqliteFile struct {
	Path string `json:"path"`
}

type SqliteWebProcess struct {
	Port          string
	CurrentFile   *SqliteFile
	SqliteWebHost string
	Process       *os.Process
	lastActivity  time.Time
	shutdownTimer *time.Timer
}

const idleTimeout = 10 * time.Minute

func (s *SqliteWebProcess) OpenFile(file *SqliteFile) {
	slog.Info("OpenFile called", "file", file.Path)
	if s.Process != nil {
		s.killCurrentProcess()
	}

	s.CurrentFile = file
	err := s.startProcess(file.Path)
	if err != nil {
		slog.Error("Failed to start process", "error", err, "file", file.Path)
		s.CurrentFile = nil
		return
	}
	s.resetShutdownTimer()
}

func (s *SqliteWebProcess) startProcess(filePath string) error {
	slog.Info("Starting sqlite-web process", "file", filePath, "port", s.Port)

	cmd := exec.Command("sqlite_web", "--port", s.Port, "--no-browser", "--host", s.SqliteWebHost, filePath)
	// cmd.Stdout = os.Stdout // For debugging, pipe stdout
	// cmd.Stderr = os.Stderr // For debugging, pipe stderr
	err := cmd.Start()
	if err != nil {
		return err
	}
	s.Process = cmd.Process
	slog.Info("sqlite-web process started", "PID", s.Process.Pid)
	go func() {
		err := cmd.Wait()
		if s.Process != nil && cmd.Process.Pid == s.Process.Pid {
			slog.Info("sqlite-web process exited", "PID", s.Process.Pid, "error", err)
			s.Process = nil
			s.CurrentFile = nil
			if s.shutdownTimer != nil {
				s.shutdownTimer.Stop()
			}
		}
	}()
	return nil
}

func (s *SqliteWebProcess) killCurrentProcess() {
	if s.Process == nil {
		slog.Info("No process to kill")
		return
	}
	slog.Info("Killing process", "PID", s.Process.Pid)
	if s.shutdownTimer != nil {
		s.shutdownTimer.Stop()
		s.shutdownTimer = nil
	}
	err := s.Process.Kill()
	if err != nil {
		slog.Error("Failed to kill process", "PID", s.Process.Pid, "error", err)
	} else {
		slog.Info("Process killed successfully", "PID", s.Process.Pid)
	}
	s.Process = nil
}

func (s *SqliteWebProcess) UnloadFile() {
	slog.Info("UnloadFile called")
	if s.Process != nil {
		s.killCurrentProcess()
	}
	s.CurrentFile = nil
	if s.shutdownTimer != nil {
		s.shutdownTimer.Stop()
		s.shutdownTimer = nil
	}
	slog.Info("File unloaded and process stopped.")
}

func (s *SqliteWebProcess) resetShutdownTimer() {
	s.lastActivity = time.Now()
	if s.shutdownTimer != nil {
		s.shutdownTimer.Stop()
	}
	s.shutdownTimer = time.AfterFunc(idleTimeout, s.shutdownIdleProcess)
}

func (s *SqliteWebProcess) shutdownIdleProcess() {
	slog.Info("Idle timeout reached. Shutting down sqlite-web process.")
	s.killCurrentProcess()
	s.CurrentFile = nil
}

func (s *SqliteWebProcess) Ping() {
	if s.Process != nil {
		s.resetShutdownTimer()
	}
}
