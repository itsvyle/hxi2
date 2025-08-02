package main

import (
	"encoding/json"
	"log/slog"
	"net/http"
	"net/http/httputil"
	"net/url"
	"os"
	"time"

	ggu "github.com/itsvyle/hxi2/global-go/utils"
)

var authManager *ggu.AuthManager
var ConfigRunningPort = "42004"
var HXI2TLD = ""
var BackupPublicKeyPath string
var BackupOutputDirectory string

var ConfigPath string
var sqliteFiles = []SqliteFile{}

var MainProcess = &SqliteWebProcess{
	Port:          "42005",
	CurrentFile:   nil,
	SqliteWebHost: "127.0.0.1",
}

func init() {
	var err error
	ggu.InitGlobalSlog()

	if port := os.Getenv("SQLITE_WEB_PORT"); port != "" {
		MainProcess.Port = port
	} else {
		slog.Warn("No SQLITE_WEB_PORT environment variable set, using default port " + MainProcess.Port)
	}

	if host := os.Getenv("SQLITE_WEB_HOST"); host != "" {
		MainProcess.SqliteWebHost = host
	} else {
		slog.Warn("No SQLITE_WEB_HOST environment variable set, using default host " + MainProcess.SqliteWebHost)
	}

	if configPath := os.Getenv("SQLITE_WEB_FILES_CONFIG"); configPath != "" {
		slog.Info("Loading sqlite-web files config", "path", configPath)
		ConfigPath = configPath
		configFile, err := os.Open(configPath)
		if err != nil {
			slog.Error("Failed to open sqlite-web files config", "path", configPath, "error", err)
			panic(err)
		}
		defer configFile.Close()

		decoder := json.NewDecoder(configFile)
		err = decoder.Decode(&sqliteFiles)
		if err != nil {
			slog.Error("Failed to decode sqlite-web files config", "path", configPath, "error", err)
			panic(err)
		}
		slog.Info("Loaded sqlite-web files", "count", len(sqliteFiles))
	} else {
		slog.Error("No sqlite-web files config found, exiting")
		panic("No sqlite-web files config found")
	}

	if publicKeyPath := os.Getenv("SQLITE_BACKUP_PUBLIC_KEY"); publicKeyPath != "" {
		slog.Info("Found backup public key path", "path", publicKeyPath)
		BackupPublicKeyPath = string(publicKeyPath)
	} else {
		slog.Warn("No SQLITE_BACKUP_PUBLIC_KEY environment variable set, backups will not be available")
	}
	if outputDir := os.Getenv("SQLITE_BACKUP_OUTPUT_DIR"); outputDir != "" {
		slog.Info("Found backup output directory", "path", outputDir)
		BackupOutputDirectory = string(outputDir)
	} else {
		slog.Warn("No SQLITE_BACKUP_OUTPUT_DIR environment variable set, backups will not be available")
	}

	if os.Getenv("CONFIG_RUNNING_PORT") != "" {
		ConfigRunningPort = os.Getenv("CONFIG_RUNNING_PORT")
	}
	HXI2TLD = os.Getenv("HXI2_TLD")

	authManager, err = ggu.NewAuthManagerFromEnv()
	if err != nil {
		panic(err)
	}
}

func dbsHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method == http.MethodPost {
		action := r.FormValue("action")
		if action == "unload" {
			if MainProcess.CurrentFile != nil {
				slog.Info("Unloading current file", "path", MainProcess.CurrentFile.Path)
				MainProcess.UnloadFile()
				// Give some time for the process to shut down if needed, though UnloadFile should be synchronous for the kill.
				// time.Sleep(500 * time.Millisecond)
			}
			http.Redirect(w, r, "/dbs", http.StatusFound)
			return
		}

		filePath := r.FormValue("file")
		if filePath == "" {
			http.Error(w, "File path is required", http.StatusBadRequest)
			return
		}
		var selectedFile *SqliteFile
		for _, f := range sqliteFiles {
			if f.Path == filePath {
				selectedFile = &f
				break
			}
		}
		if selectedFile == nil {
			http.Error(w, "Invalid file path selected", http.StatusBadRequest)
			return
		}
		MainProcess.OpenFile(selectedFile)
		time.Sleep(1 * time.Second)
		http.Redirect(w, r, "/", http.StatusFound)
		slog.Info("Switched sqlite-web to file", "path", selectedFile.Path)
		return
	}

	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	html := `
<!DOCTYPE html>
<html>
<head>
	<title>Select SQLite Database</title>
</head>
<body>
	<h1>Select SQLite Database</h1>
	<form method="POST" action="/dbs">
		<label for="file">Choose a database:</label>
		<select name="file" id="file">`
	for _, f := range sqliteFiles {
		selected := ""
		if MainProcess.CurrentFile != nil && MainProcess.CurrentFile.Path == f.Path {
			selected = " selected"
		}
		html += `<option value="` + f.Path + `"` + selected + `>` + f.Path + `</option>`
	}
	html += `
		</select>
		<input type="submit" value="Open Database">
	</form>
`
	if MainProcess.CurrentFile != nil {
		html += `<p>Currently selected: ` + MainProcess.CurrentFile.Path + `</p>`
		html += `
	<form method="POST" action="/dbs" style="margin-top: 10px;">
		<input type="hidden" name="action" value="unload">
		<input type="submit" value="Unload Current Database">
	</form>
`
		if MainProcess.shutdownTimer != nil {
			remaining := idleTimeout - time.Since(MainProcess.lastActivity)
			if remaining > 0 {
				html += `<p>Will be suspended for inactivity in: ` + remaining.Round(time.Second).String() + `</p>`
			} else {
				html += `<p>Process is being suspended for inactivity or has been suspended.</p>`
			}
		}
	} else {
		html += `<p>No database currently selected.</p>`
	}
	html += `
	<a href="/backup">Backup Current Database</a>
</body>
</html>`
	_, err := w.Write([]byte(html))
	if err != nil {
		slog.Error("Failed to write HTML response", "error", err)
		http.Error(w, "Failed to render page", http.StatusInternalServerError)
	}
}

func backupHandler(w http.ResponseWriter, r *http.Request) {
	if BackupOutputDirectory == "" || BackupPublicKeyPath == "" {
		http.Error(w, "Backup functionality is not configured", http.StatusServiceUnavailable)
		return
	}
	err := DoBackup()
	if err != nil {
		http.Error(w, "Backup failed: "+err.Error(), http.StatusInternalServerError)
		return
	}
	w.WriteHeader(http.StatusOK)
	w.Write([]byte("Backup completed successfully"))
	return
}

func authCheckMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {

		c, err := authManager.AuthenticateHTTPRequest(w, r, false)
		if err != nil || !c.CheckPermHTTP(w, ggu.RoleAdmin) {
			return
		}

		if r.URL.Path == "/backup" {
			next.ServeHTTP(w, r)
			return
		}

		if MainProcess.CurrentFile == nil {
			if r.URL.Path == "/dbs" {
				next.ServeHTTP(w, r)
			} else if r.URL.Path == "/" {
				http.Redirect(w, r, "/dbs", http.StatusFound)
			} else {
				http.Error(w, "No sqlite file selected. Go to /dbs to select one.", http.StatusServiceUnavailable)
			}
			return
		} else if r.URL.Path != "/dbs" {
			MainProcess.Ping()
		}
		next.ServeHTTP(w, r)
	})
}

func main() {

	if BackupOutputDirectory != "" && BackupPublicKeyPath != "" {
		slog.Info("Executing startup backup")
		go DoBackup()
	}

	slog.Info("Starting sqlite-web backend")
	router := http.NewServeMux()

	sqliteEndpoint := "http://" + MainProcess.SqliteWebHost + ":" + MainProcess.Port
	targetUrl, err := url.Parse(sqliteEndpoint)
	if err != nil {
		slog.With("error", err).Error("Failed to parse target URL for proxy")
		os.Exit(1)
	}
	proxy := httputil.NewSingleHostReverseProxy(targetUrl)

	router.Handle("/", authCheckMiddleware(proxy))
	router.HandleFunc("/dbs", dbsHandler)
	router.HandleFunc("/backup", backupHandler)

	server := &http.Server{
		Addr:              ":" + ConfigRunningPort,
		Handler:           router,
		ReadHeaderTimeout: 5 * time.Second,
	}

	slog.With("port", ConfigRunningPort).Info("Server is running")
	slog.With("sqlite_endpoint", sqliteEndpoint).Info("Proxying to sqlite-web")
	slog.With("error", server.ListenAndServe()).Error("Server crashed")
}
