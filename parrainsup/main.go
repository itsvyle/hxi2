package main

import (
	"embed"
	"log/slog"
	"net/http"
	"os"
	"strconv"
	"time"

	_ "embed"

	ggu "github.com/itsvyle/hxi2/global-go/utils"
)

var authManager *ggu.AuthManager
var ConfigRunningPort = "42005"
var HXI2TLD = ""

var mainUsersCacher *ggu.Cacher[map[int64]*MainUser]

//go:embed promo-active.txt
var promoActiveStr string

var promoActive int

func init() {
	var err error
	promoActive, err = strconv.Atoi(promoActiveStr)
	if err != nil {
		slog.Error("Failed to parse promo-active.txt", "error", err)
		promoActive = 0
		panic("e")
	}

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

	// #region Database
	DB = NewDatabaseManager(ConfigDBPath)
	// #endregion

	// #region Auth manager

	authManager, err = ggu.NewAuthManagerFromEnv()
	if err != nil {
		panic(err)
	}
	// #endregion

	// #region Cachers
	mainUsersCacher = ggu.NewCacher("mainUsersCacher", func() (map[int64]*MainUser, error) {
		users, err := DB.ListVisibleMainUsers()
		if err != nil {
			return nil, err
		}
		return users, nil
	}, 60*time.Second, 5)
	// #endregion
}

//go:embed dist/*
var staticsFS embed.FS

func main() {
	slog.Info("Starting parrainsup-backend")
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

	mainHTML, mainJS, mainCSS := staticsManager.WholeRouteHandlers("main")
	router.Handle("/dist/main.bundle.js", mainJS)
	if mainCSS != nil {
		router.Handle("/dist/main.bundle.css", mainCSS)
	}

	router.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/" && r.URL.Path != "/index.html" {
			http.NotFound(w, r)
			return
		}
		c, err := authManager.AuthenticateHTTPRequestIncludingTemporary(w, r, false)
		if err != nil || !CheckClaimsIncludingTemp(c) {
			return
		}
		mainHTML.ServeHTTP(w, r)
	})

	editHTML, editJS, editCSS := staticsManager.WholeRouteHandlers("edit")
	router.Handle("/dist/edit.bundle.js", editJS)
	if editCSS != nil {
		router.Handle("/dist/edit.bundle.css", editCSS)
	}

	router.HandleFunc("/edit", func(w http.ResponseWriter, r *http.Request) {
		c, err := authManager.AuthenticateHTTPRequest(w, r, false)
		if err != nil || !c.CheckPermHTTP(w, ggu.RoleStudent) {
			return
		}
		if c.Promotion != promoActive {
			http.Error(w, "You are not part of the active promotion - you can't edit a profile on Parrainsup", http.StatusForbidden)
			return
		}
		editHTML.ServeHTTP(w, r)
	})

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

	router.Handle("/api/list_relations", ggu.GzipMiddleware(http.HandlerFunc(HandleListRelations)))
	router.Handle("POST /api/relation", http.HandlerFunc(HandlePostRelation))
	router.Handle("DELETE /api/relation", http.HandlerFunc(HandleDeleteRelation))
	router.Handle("GET /api/global_tree", ggu.GzipMiddleware(http.HandlerFunc(HandleGetGlobalTree))) */
	router.Handle("GET /api/list_users", ggu.GzipMiddleware(http.HandlerFunc(HandleListUsers)))
	router.Handle("GET /api/me", http.HandlerFunc(HandleGetUserMyself))
	router.Handle("PUT /api/me", http.HandlerFunc(HandleUpdateUserMyself))
	router.Handle("GET /temp", authManager.HandleTempLogin("parrainsup", "https://parrainsup."+HXI2TLD))

	slog.With("port", ConfigRunningPort).Info("Server is running")
	slog.With("error", server.ListenAndServe()).Error("Server crashed")
}
