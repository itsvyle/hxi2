package main

import (
	"embed"
	"io/fs"
	"log/slog"
	"net/http"
	"os"
	"path"
	"time"

	_ "embed"

	ggu "github.com/itsvyle/hxi2/global-go-utils"
)

var ConfigRunningPort = "8037"

func init() {
	ggu.InitGlobalSlog()

	if os.Getenv("CONFIG_RUNNING_PORT") != "" {
		ConfigRunningPort = os.Getenv("CONFIG_RUNNING_PORT")
	}
}

//go:embed memes/* main-out.html*
var staticsFS embed.FS

const mp3ContentType = "audio/mpeg"

func main() {
	slog.Info("Starting soundboard backend")
	router := http.NewServeMux()

	server := &http.Server{
		Addr:              ":" + ConfigRunningPort,
		Handler:           router,
		ReadHeaderTimeout: 5 * time.Second,
	}

	staticsManager := ggu.NewStaticFilesManager(
		staticsFS,
		"",
		ggu.StaticsDefaultContentSecurityPolicy(""),
	)

	indexHandler := staticsManager.GenerateStaticFileHandler("main-out.html", "text/html")
	router.Handle("/index.html", indexHandler)
	router.Handle("/memes/pause_icon.png", staticsManager.GenerateStaticFileHandler("memes/pause_icon.png", "image/png"))

	sounds, err := getFilesFromDir("memes/sound", staticsFS)
	if err != nil {
		slog.With("error", err).Error("Failed to get sound files")
		panic("Failed to get sound files")
	}

	for _, sound := range sounds {
		router.HandleFunc("/"+sound, staticsManager.GenerateStaticFileHandler(sound, mp3ContentType))
	}

	imgs, err := getFilesFromDir("memes/img", staticsFS)
	if err != nil {
		slog.With("error", err).Error("Failed to get image files")
		panic("Failed to get image files")
	}

	for _, img := range imgs {
		ty := "image/png"
		if path.Ext(img) == ".jpg" {
			ty = "image/jpeg"
		}
		router.HandleFunc("/"+img, staticsManager.GenerateStaticFileHandler(img, ty))
	}

	router.Handle("/", indexHandler)

	slog.With("port", ConfigRunningPort).Info("Server is running")
	slog.With("error", server.ListenAndServe()).Error("Server crashed")
}

// Excludes compressed files
func getFilesFromDir(dirName string, f fs.FS) ([]string, error) {
	files, err := fs.ReadDir(f, dirName)
	if err != nil {
		return nil, err
	}

	var fileNames []string
	for _, file := range files {
		if path.Ext(file.Name()) == ".gz" || path.Ext(file.Name()) == ".br" {
			continue
		}
		fileNames = append(fileNames, path.Join(dirName, file.Name()))
	}

	return fileNames, nil
}
