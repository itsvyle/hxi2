package main

import (
	"embed"
	"log/slog"
	"net/http"
	"os"
	"time"

	ggu "github.com/itsvyle/hxi2/global-go/utils"
)

var authManager *ggu.AuthManager
var ConfigRunningPort = "42005"
var HXI2TLD = ""

func init() {
	var err error
	ggu.InitGlobalSlog()

	fails := []string{}
	fail := func(key string) {
		fails = append(fails, key)
	}
	loadOrFail := func(key string) string {
		v := os.Getenv(key)
		if v == "" {
			fail(key + " is not defined")
		}
		return v
	}

	var ConfigDBPath = loadOrFail("CONFIG_DB_PATH")
	if os.Getenv("CONFIG_RUNNING_PORT") != "" {
		ConfigRunningPort = os.Getenv("CONFIG_RUNNING_PORT")
	}
	HXI2TLD = os.Getenv("HXI2_TLD")

	if len(fails) > 0 {
		for _, f := range fails {
			slog.Error(f)
		}
		panic("Failed to load configuration")
	}

	// #region Auth manager

	authManager, err = ggu.NewAuthManagerFromEnv()
	if err != nil {
		panic(err)
	}
	// #endregion
}

//go:embed dist/*
var staticsFS embed.FS

func main() {
	slog.Info("Starting tree-backend")
	router := http.NewServeMux()

	server := &http.Server{
		Addr:              ":" + ConfigRunningPort,
		Handler:           router,
		ReadHeaderTimeout: 5 * time.Second,
	}

	staticsManager := ggu.NewStaticFilesManager(
		staticsFS,
		"dist",
		ggu.StaticsDefaultContentSecurityPolicy(HXI2TLD),
	)

	staticsManager.RegisterChunkHandlers(router)

	/* addHTML, addJS, addCSS := staticsManager.WholeRouteHandlers("add")
	router.Handle("/dist/add.bundle.js", addJS)
	if addCSS != nil {
		router.Handle("/dist/add.bundle.css", addCSS)
	}

	router.HandleFunc("/add", func(w http.ResponseWriter, r *http.Request) {
		c, err := authManager.AuthenticateHTTPRequest(w, r, false)
		if err != nil || !c.CheckPermHTTP(w, ggu.RoleStudent) {
			return
		}
		addHTML.ServeHTTP(w, r)
	})

	treeHTML, treeJS, treeCSS := staticsManager.WholeRouteHandlers("tree")
	router.Handle("/dist/tree.bundle.js", treeJS)
	if treeCSS != nil {
		router.Handle("/dist/tree.bundle.css", treeCSS)
	}

	router.HandleFunc("/tree", func(w http.ResponseWriter, r *http.Request) {
		c, err := authManager.AuthenticateHTTPRequest(w, r, false)
		if err != nil || !c.CheckPermHTTP(w, ggu.RoleStudent) {
			return
		}
		treeHTML.ServeHTTP(w, r)
	})

	router.Handle("/api/list_users", ggu.GzipMiddleware(http.HandlerFunc(HandleListUsers)))
	router.Handle("/api/list_relations", ggu.GzipMiddleware(http.HandlerFunc(HandleListRelations)))
	router.Handle("POST /api/relation", http.HandlerFunc(HandlePostRelation))
	router.Handle("DELETE /api/relation", http.HandlerFunc(HandleDeleteRelation))
	router.Handle("GET /api/global_tree", ggu.GzipMiddleware(http.HandlerFunc(HandleGetGlobalTree))) */

	slog.With("port", ConfigRunningPort).Info("Server is running")
	slog.With("error", server.ListenAndServe()).Error("Server crashed")
}
