package globalgoutils

import (
	"crypto/md5"
	"encoding/hex"
	"errors"
	"io"
	"io/fs"
	"log"
	"net/http"
	"os"
	"path/filepath"
	"strings"
)

type StaticFileCacheControlHeader struct {
	// cache control to use
	BaseValue string
	// cache control to use if the url ends with __hash__
	IfHashedURL string
}

func (s *StaticFileCacheControlHeader) Chose(rawQuery string) string {
	if strings.HasSuffix(rawQuery, "__hash__") {
		return s.IfHashedURL
	}
	return s.BaseValue
}

type StaticFilesManager struct {
	FS                    fs.FS
	baseDistPath          string
	contentSecurityPolicy string
	// Takes a filename and returns the cache control header for it
	// calling it can be unoptimized, it's only called once per file
	getCacheControl func(string) StaticFileCacheControlHeader
}

// by default, if no hash only cache for 10 minutes, then ask for revalidation
func DefaultGetCacheControl(filename string) StaticFileCacheControlHeader {
	if strings.Contains(filename, "favicon") {
		return StaticFileCacheControlHeader{
			BaseValue:   "public, max-age=86400, must-revalidate",
			IfHashedURL: "public, max-age=86400, must-revalidate",
		}
	} else if strings.HasSuffix(filename, "__hash__.chunk.js") {
		return StaticFileCacheControlHeader{
			BaseValue:   "public, max-age=31536000, immutable",
			IfHashedURL: "public, max-age=31536000, immutable",
		}
	} else if strings.HasSuffix(filename, ".js") || strings.HasSuffix(filename, ".css") {
		return StaticFileCacheControlHeader{
			BaseValue:   "public, max-age=600, must-revalidate",
			IfHashedURL: "public, max-age=31536000, immutable",
		}
	}
	return StaticFileCacheControlHeader{
		BaseValue:   "private, max-age=600, must-revalidate",
		IfHashedURL: "private, max-age=31536000, immutable",
	}
}

// for baseDistPath, if it's an embed FS, don't put a startig slash
// recommended defaultCacheControl is "public, max-age=3600, must-revalidate"
func NewStaticFilesManager(fs fs.FS, baseDistPath string, contentSecurityPolicy string) *StaticFilesManager {
	return &StaticFilesManager{
		FS:                    fs,
		baseDistPath:          baseDistPath,
		contentSecurityPolicy: contentSecurityPolicy,
		getCacheControl:       DefaultGetCacheControl,
	}
}

const chunkFileEnd = ".chunk.js"

func (manager *StaticFilesManager) RegisterChunkHandlers(router *http.ServeMux) {
	chunkFiles, err := fs.ReadDir(manager.FS, manager.baseDistPath)
	if err != nil {
		log.Println("Error reading static files directory")
		panic(err)
	}
	const jsContentType = "application/javascript; charset=utf-8"

	for _, file := range chunkFiles {
		if strings.HasSuffix(file.Name(), chunkFileEnd) {
			fileName := file.Name()
			router.HandleFunc("/dist/"+fileName, manager.GenerateStaticFileHandler(filepath.Join(manager.baseDistPath, fileName), jsContentType))
		}
	}
}

func (manager *StaticFilesManager) WholeRouteRegisterHandlers(htmlPageURL string, fileBaseName string, router *http.ServeMux) {
	handlerHTML, handlerJS, handlerCSS := manager.WholeRouteHandlers(fileBaseName)
	if handlerHTML != nil {
		router.HandleFunc("GET "+htmlPageURL, handlerHTML)
		router.HandleFunc("GET "+htmlPageURL+".html", handlerHTML)
	}
	if handlerJS != nil {
		router.HandleFunc("GET /dist/"+fileBaseName+".bundle.js", handlerJS)
	}
	if handlerCSS != nil {
		router.HandleFunc("GET /dist/"+fileBaseName+".bundle.css", handlerCSS)
	}
}

// Returns handlers for the associated files to a base name, if they exist.
func (manager *StaticFilesManager) WholeRouteHandlers(fileBaseName string) (handlerHTML http.HandlerFunc, handlerJS http.HandlerFunc, handlerCSS http.HandlerFunc) {
	fileBaseName = filepath.Join(manager.baseDistPath, fileBaseName)
	if manager.fileExists(fileBaseName + ".html") {
		handlerHTML = manager.GenerateStaticFileHandler(fileBaseName+".html", "text/html")
	}
	if manager.fileExists(fileBaseName + ".bundle.js") {
		handlerJS = manager.GenerateStaticFileHandler(fileBaseName+".bundle.js", "application/javascript; charset=utf-8")
	}
	if manager.fileExists(fileBaseName + ".bundle.css") {
		handlerCSS = manager.GenerateStaticFileHandler(fileBaseName+".bundle.css", "text/css; charset=utf-8")
	}
	return
}

func (manager *StaticFilesManager) GenerateStaticFileHandler(file string, contentType string) http.HandlerFunc {
	hasGzip := manager.fileExists(file + ".gz")
	hasBrotli := manager.fileExists(file + ".br")
	etag, err := manager.readFileAndHashMD5(file)
	if err != nil {
		log.Println("Error reading static file " + file)
		panic(err)
	}

	cacheControl := manager.getCacheControl(file)

	return func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", contentType)

		w.Header().Set("Cache-Control", cacheControl.Chose(r.URL.RawQuery))

		w.Header().Set("ETag", etag)
		if manager.contentSecurityPolicy != "" {
			w.Header().Set("Content-Security-Policy", manager.contentSecurityPolicy)
		}

		if match := r.Header.Get("If-None-Match"); match == etag {
			// If ETag matches, send Not Modified status
			http.Error(w, http.StatusText(http.StatusNotModified), http.StatusNotModified)
			return
		}

		if hasBrotli && canBrotli(r) {
			w.Header().Set("Content-Encoding", "br")
			w.Header().Set("Vary", "Accept-Encoding")
			http.ServeFileFS(w, r, manager.FS, file+".br")
		} else if hasGzip && canGZIP(r) {
			w.Header().Set("Content-Encoding", "gzip")
			w.Header().Set("Vary", "Accept-Encoding")
			http.ServeFileFS(w, r, manager.FS, file+".gz")
		} else {
			http.ServeFileFS(w, r, manager.FS, file)
		}
	}
}

func (manager *StaticFilesManager) fileExists(path string) bool {
	if _, err := fs.Stat(manager.FS, path); errors.Is(err, os.ErrNotExist) {
		return false
	}
	return true
}

// readFileAndHashMD5 reads the content of a file, hashes it using MD5, and returns the hash.
// It also ensures proper closing of the file.
func (manager *StaticFilesManager) readFileAndHashMD5(filePath string) (string, error) {
	file, err := manager.FS.Open(filePath)
	if err != nil {
		return "", err
	}
	defer file.Close()
	hasher := md5.New()

	if _, err := io.Copy(hasher, file); err != nil {
		return "", err
	}

	hashSum := hasher.Sum(nil)

	hashString := hex.EncodeToString(hashSum)

	return hashString, nil
}

func canGZIP(req *http.Request) bool {
	if !strings.Contains(req.Header.Get("Accept-Encoding"), "gzip") ||
		strings.Contains(req.Header.Get("Connection"), "Upgrade") ||
		strings.Contains(req.Header.Get("Accept"), "text/event-stream") {
		return false
	}
	return true
}

func canBrotli(req *http.Request) bool {
	if !strings.Contains(req.Header.Get("Accept-Encoding"), "br") ||
		strings.Contains(req.Header.Get("Connection"), "Upgrade") ||
		strings.Contains(req.Header.Get("Accept"), "text/event-stream") {
		return false
	}
	return true
}

const iconifyScriptSource = "https://cdn.jsdelivr.net/npm/iconify-icon@2.3.0/dist/iconify-icon.min.js"

var iconifyIconsSources = []string{
	"https://api.iconify.design/",
	"https://api.unisvg.com/",
	"https://api.simplesvg.com/",
}

// Genereates a default Content Security Policy for static files, that lets iconify through.
func StaticsDefaultContentSecurityPolicy(hxi2Domain string) string {
	iconifyIconsSources := strings.Join(iconifyIconsSources, " ")
	if hxi2Domain != "" && strings.Trim(hxi2Domain, " ") != "" {
		hxi2Domain = " https://static." + hxi2Domain + " "
	}
	return `default-src 'self'; 
connect-src 'self' ` + iconifyIconsSources + `;
img-src 'self'` + hxi2Domain + `; 
script-src 'self' ` + iconifyScriptSource + ` https://unpkg.com/@github/filter-input-element@latest/dist/index.js ` + hxi2Domain + `;
style-src 'self' 'unsafe-inline' ` + hxi2Domain + `;
object-src ` + hxi2Domain + `; 
frame-src 'none'; 
base-uri 'self'; 
form-action 'self'; 
frame-ancestors 'none'; 
manifest-src 'self';`
}
