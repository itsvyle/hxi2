package main

import (
	"embed"
	"log/slog"
	"net/http"
	"os"
	"strings"
	"time"

	_ "embed"

	ggu "github.com/itsvyle/hxi2/global-go-utils"
	"github.com/jmoiron/sqlx"
	_ "github.com/mattn/go-sqlite3"
	"golang.org/x/oauth2"
)

const OneTimeCodeLength = 6

var ConfigRunningPort = "8080"
var ConfigDBPath = ""
var ConfigJWTPrivateKey = ""
var ConfigRedirectURL = ""
var ConfigDiscordApplicationID = ""
var ConfigDiscordClientID = ""
var ConfigDiscordClientSecret = ""
var ConfigDefaultLoginRedirect = "/"

var HXI2AuthURL = ""
var HXI2CookiesDomain = ""
var HXI2TLD = ""

var IsLocalDebugInstance = false

var authManager *ggu.AuthManager
var jwtManager *JWTManager
var JWTValidityDuration = 10 * time.Minute
var JWTRefreshTokenValidityDuration = 30 * 24 * time.Hour

var discordOauthConfig *oauth2.Config

var apiUsersCacher *ggu.Cacher[map[string]*DBApiUser]

//go:embed schema.sql
var sqlSchema string

func init() {

	ggu.InitGlobalSlog()

	//#region Load from environment
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

	if os.Getenv("CONFIG_RUNNING_PORT") != "" {
		ConfigRunningPort = os.Getenv("CONFIG_RUNNING_PORT")
	}
	if os.Getenv("HXI2_COOKIES_DOMAIN") != "" {
		HXI2CookiesDomain = os.Getenv("HXI2_COOKIES_DOMAIN")
	}
	if os.Getenv("CONFIG_DEFAULT_LOGIN_REDIRECT") != "" {
		ConfigDefaultLoginRedirect = os.Getenv("CONFIG_DEFAULT_LOGIN_REDIRECT")
	}
	if os.Getenv("CONFIG_JWT_PRIVATE_KEY") != "" {
		ConfigJWTPrivateKey = os.Getenv("CONFIG_JWT_PRIVATE_KEY")

		if ConfigJWTPrivateKey == "generate" {
			privateKey, publicKey, err := GenerateECDSAKeys()
			if err != nil {
				slog.With("error", err).Error("Failed to generate ECDSA keys")
				panic(err)
			}
			privateKeyPEM, _ := ExportKeyAsPEM(privateKey)
			publicKeyPEM, _ := ExportKeyAsPEM(publicKey)

			tnow := time.Now().Format(time.RFC3339)

			err = os.WriteFile("private_"+tnow+".pem", privateKeyPEM, 0600)
			if err != nil {
				slog.With("error", err).Error("Failed to write private key to file")
				panic(err)
			}
			err = os.WriteFile("public_"+tnow+".pem", publicKeyPEM, 0600)
			if err != nil {
				slog.With("error", err).Error("Failed to write public key to file")
				panic(err)
			}

			slog.Info("Generated ECDSA keys and saved to files private_" + tnow + ".pem and public_" + tnow + ".pem")

			ConfigJWTPrivateKey = string(privateKeyPEM)
		}

	} else {
		fail("CONFIG_JWT_PRIVATE_KEY is not defined")
	}

	ConfigDBPath = loadOrFail("CONFIG_DB_PATH")
	ConfigDiscordApplicationID = loadOrFail("CONFIG_DISCORD_APPLICATION_ID")
	ConfigDiscordClientID = loadOrFail("CONFIG_DISCORD_CLIENT_ID")
	ConfigDiscordClientSecret = loadOrFail("CONFIG_DISCORD_CLIENT_SECRET")
	HXI2AuthURL = loadOrFail("HXI2_AUTH_URL")
	Hxi2AuthEndpoint := loadOrFail("HXI2_AUTH_ENDPOINT")
	HXI2TLD = loadOrFail("HXI2_TLD")

	IsLocalDebugInstance = os.Getenv("LOCAL_DEBUG_INSTANCE") == "1"

	ConfigRedirectURL = HXI2AuthURL + "/api/discord_callback"

	if len(fails) > 0 {
		for _, f := range fails {
			slog.Error(f)
		}
		panic("Failed to load configuration")
	}
	//#endregion

	//#region Start database

	// if _, err := os.Stat(ConfigDBPath); os.IsNotExist(err) {
	// 	err = os.MkdirAll(ConfigDBPath, os.ModePerm)
	// 	if err != nil {
	// 		slog.With("error", err).Error("Failed to create database directory")
	// 		panic(err)
	// 	}
	// }
	sqlDB, err := sqlx.Open("sqlite3", ConfigDBPath)
	if err != nil {
		slog.With("error", err).Error("Failed to open database")
		panic(err)
	}
	err = sqlDB.Ping()
	if err != nil {
		slog.With("error", err).Error("Failed to ping database")
		panic(err)
	}

	sqlDB.MustExec(sqlSchema)

	DB = &DatabaseManager{
		DB:     sqlDB,
		logger: ggu.GetServiceSpecificLogger("DB", "\033[38;5;226m"),
	}

	slog.With("file", ConfigDBPath).Info("Connected to database")
	//#endregion

	// #region Cachers
	apiUsersCacher = ggu.NewCacher("apiUsersCacher", func() (map[string]*DBApiUser, error) {
		a, err := DB.ListAPIUsers()
		if err != nil {
			return nil, err
		}
		m := make(map[string]*DBApiUser, len(a))
		for i := range a {
			m[a[i].Token] = &a[i]
		}
		return m, nil
	}, 30*time.Second, 0)

	// #endregion

	//#region Create JWT manager
	jwtManager, err = NewJWTManager(ConfigJWTPrivateKey, JWTValidityDuration, JWTRefreshTokenValidityDuration)
	if err != nil {
		slog.With("error", err).Error("Failed to create JWT manager")
		panic(err)
	}
	//#endregion

	//#region Create auth manager
	// Currently unused, as all requests from the auth API are done through projects APIs
	a := &ggu.AuthManager{
		Logger:       ggu.GetAuthLogger(),
		AutoFetchKey: false,
		AuthURL:      HXI2AuthURL,
		AuthEndpoint: Hxi2AuthEndpoint,
		LoginPageURL: HXI2AuthURL + "/login",
		CookieDomain: HXI2CookiesDomain,
		RenewToken: func(_ *ggu.AuthManager, oldToken, refreshToken string) (res *ggu.AuthRenewalResponse, err error) {
			return RenewTokenActionner(oldToken, refreshToken)
		},
	}
	authManager, err = ggu.NewAuthManagerPublicKey(
		a,
		jwtManager.PublicKeyPEM,
	)
	if err != nil {
		slog.With("error", err).Error("Failed to create auth manager")
		panic(err)
	}
	//#endregion

	//#region Create oauth2 manager
	discordOauthConfig = &oauth2.Config{
		ClientID:     ConfigDiscordClientID,
		ClientSecret: ConfigDiscordClientSecret,
		RedirectURL:  ConfigRedirectURL,
		Scopes:       []string{"identify"},
		Endpoint: oauth2.Endpoint{
			AuthURL:  "https://discord.com/api/oauth2/authorize",
			TokenURL: "https://discord.com/api/oauth2/token",
		},
	}
	//#endregion

	//#region Create discord bot
	if os.Getenv("CONFIG_DISCORD_BOT_TOKEN") != "" {
		discordBot, err := NewDiscordBot(os.Getenv("CONFIG_DISCORD_BOT_TOKEN"))
		if err != nil {
			slog.With("error", err).Error("Failed to create discord bot")
			panic(err)
		}
		err = discordBot.Start()
	}
	//#endregion
}

//go:embed dist/*
var staticsFS embed.FS

func main() {
	DB.StartOneTimeCodeCleanupTimer()

	slog.Info("Starting auth-backend")
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

	loginHTML, loginJS, loginCSS := staticsManager.WholeRouteHandlers("login")
	redirectCookOpts := &ggu.OverwriteCookieOptions{
		Path: ggu.StringPtr("/api/discord_callback"),
	}
	router.Handle("/login", http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {

		code := r.URL.Query().Get("code")
		if code == "" && r.Method == http.MethodPost {
			code = r.FormValue("code")
		}
		{
			if code != "" {
				code := strings.TrimSpace(code)
				if len(code) != OneTimeCodeLength {
					LoginError(w, r, "Invalid one-time code")
					return
				}
				userID, err := DB.CheckOneTimeCode(code)
				if err != nil {
					LoginError(w, r, "Invalid one-time code")
					return
				}
				dbUser, err := DB.GetDBUserByID(userID)
				if err != nil {
					slog.With("error", err).Error("Failed to get user by ID after one-time code validation")
					LoginError(w, r, "Failed to get user information")
					return
				}
				if !setAuthCookies(w, r, dbUser) {
					return
				}

				redirectTo := ""
				redirectToCookie, err := r.Cookie("authRedirectTo")
				if err != nil || redirectToCookie == nil || redirectToCookie.Value == "" {
					redirectTo = r.URL.Query().Get("redirectTo")
					if redirectTo == "" {
						redirectTo = ConfigDefaultLoginRedirect
					}
				} else {
					redirectTo = redirectToCookie.Value
				}

				body := "<html><head><meta http-equiv=\"refresh\" content=\"0; url=" + redirectTo + "\"></head><body>Redirecting...</body></html>"
				w.Header().Set("Content-Type", "text/html; charset=utf-8")
				w.WriteHeader(http.StatusOK)
				_, err = w.Write([]byte(body))
				if err != nil {
					slog.With("error", err).Error("Failed to write redirect response")
				}
				return
			}
		}

		redirectTo := r.URL.Query().Get("redirectTo")
		if redirectTo == "" {
			redirectTo = ConfigDefaultLoginRedirect
		}

		http.SetCookie(w, ggu.GenerateCookieObject("authRedirectTo", redirectTo, 30*time.Minute, redirectCookOpts))

		loginHTML.ServeHTTP(w, r)
	}))
	router.Handle("/logout", http.HandlerFunc(HandleLogout))

	router.Handle("/dist/login.bundle.js", loginJS)
	router.Handle("/dist/login.bundle.css", loginCSS)

	router.Handle("/api/login", http.HandlerFunc(HandleLogin))

	router.Handle("/api/public-key", http.HandlerFunc(HandlerPublicKey))
	router.Handle("/api/discord_callback", http.HandlerFunc(HandleDiscordCallback))
	router.Handle("POST /api/renew", http.HandlerFunc(HandleRenewToken))

	router.Handle("GET /api/project/list_users", http.HandlerFunc(ProjectHandleListUsers))

	router.Handle("/", http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		http.Redirect(w, r, "/login", http.StatusFound)
	}))

	slog.With("port", ConfigRunningPort).Info("Server is running")
	slog.With("error", server.ListenAndServe()).Error("Server crashed")
}
