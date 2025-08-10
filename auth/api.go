package main

import (
	"encoding/base64"
	"encoding/json"
	"errors"
	"fmt"
	"log/slog"
	"net/http"
	"net/url"
	"strings"
	"time"

	"github.com/cristalhq/jwt/v5"
	ggu "github.com/itsvyle/hxi2/global-go/utils"
	"golang.org/x/net/publicsuffix"
)

func HandlerPublicKey(w http.ResponseWriter, r *http.Request) {
	if !checkProjectApiAuth(w, r, ggu.APIRoleAuthentication) {
		return
	}
	w.Header().Set("Content-Type", "text/plain")
	_, err := w.Write([]byte(jwtManager.PublicKeyPEM))
	if err != nil {
		slog.With("request", r, "error", err).Error("Failed to write public key to response")
	}
}

func LoginError(w http.ResponseWriter, r *http.Request, err string) {
	http.SetCookie(w, ggu.GenerateCookieObject(
		"authError",
		err,
		5*time.Minute,
		&ggu.OverwriteCookieOptions{
			Domain:   ggu.StringPtr(HXI2CookiesDomain),
			HttpOnly: ggu.BoolPtr(false),
			Secure:   ggu.BoolPtr(false),
		},
	))
	headerAccepts := r.Header.Get("Accept")
	if headerAccepts != "" {
		if strings.Contains(headerAccepts, "text/html") {
			errBase64 := base64.StdEncoding.EncodeToString([]byte(err))
			body := "<html><head><meta http-equiv=\"refresh\" content=\"0; url=" + authManager.LoginPageURL + "\"><style>html{color-scheme: dark;}</style><script>localStorage.setItem('authError', '" + errBase64 + "');</script></head><body>Redirecting...</body></html>"
			w.Header().Set("Content-Type", "text/html; charset=utf-8")
			w.WriteHeader(http.StatusOK)
			_, err := w.Write([]byte(body))
			if err != nil {
				slog.With("error", err).Error("Failed to write redirect response")
			}
			return
		}
	}
	http.Redirect(w, r, authManager.LoginPageURL, http.StatusSeeOther)
}

// Passed to here when wanting to login
func HandleLogin(w http.ResponseWriter, r *http.Request) {
	cookConf := &ggu.OverwriteCookieOptions{
		Domain: ggu.StringPtr(HXI2CookiesDomain),
		Path:   ggu.StringPtr("/api"),
	}

	state, err := ggu.GenerateRandomString(32)
	if err != nil {
		slog.With("error", err).Error("Failed to generate random string")
		LoginError(w, r, "Failed to generate random string")
		return
	}

	stateCookie := ggu.GenerateCookieObject("state", state, 5*time.Minute, cookConf)
	http.SetCookie(w, stateCookie)

	url := discordOauthConfig.AuthCodeURL(state)

	http.Redirect(w, r, url, http.StatusFound)
}

func HandleLogout(w http.ResponseWriter, r *http.Request) {
	http.SetCookie(w, ggu.GenerateCookieObject(ggu.CookieToken, "", -1*time.Minute, &ggu.OverwriteCookieOptions{
		Domain: ggu.StringPtr(HXI2CookiesDomain),
		Path:   ggu.StringPtr("/"),
	}))
	http.SetCookie(w, ggu.GenerateCookieObject(ggu.CookieSmallData, "", -1*time.Minute, &ggu.OverwriteCookieOptions{
		Domain: ggu.StringPtr(HXI2CookiesDomain),
		Path:   ggu.StringPtr("/"),
	}))

	refreshTokenCookie, err := r.Cookie(ggu.CookieRefresh)
	if err != nil {
		if !errors.Is(err, http.ErrNoCookie) {
			slog.With("error", err).Error("Failed to get refresh token cookie")
			LoginError(w, r, "Failed to get refresh token cookie")
			return
		}
	}
	if refreshTokenCookie.Value != "" {
		err = DB.DeleteRefreshToken(refreshTokenCookie.Value)
		if err != nil {
			slog.With("error", err).Error("Failed to delete refresh token pair")
			LoginError(w, r, "Failed to delete refresh token pair")
			return
		}
	}
	http.SetCookie(w, ggu.GenerateCookieObject(ggu.CookieRefresh, "", -1*time.Minute, &ggu.OverwriteCookieOptions{
		Domain: ggu.StringPtr(HXI2CookiesDomain),
		Path:   ggu.StringPtr("/"),
	}))

	http.Redirect(w, r, ConfigDefaultLoginRedirect, http.StatusFound)
}

type DiscordAuthorizationInfo struct {
	ID            string `json:"id"`
	Username      string `json:"username"`
	Discriminator string `json:"discriminator"`
	Avatar        string `json:"avatar"`
	GlobalName    string `json:"global_name"`
}

func HandleDiscordCallback(w http.ResponseWriter, r *http.Request) {
	stateCookie, err := r.Cookie("state")
	if err != nil {
		slog.With("error", err).Error("Failed to get state cookie")
		LoginError(w, r, "Failed to get state cookie")
		return
	}

	clientState := stateCookie.Value

	redirectTo := ConfigDefaultLoginRedirect
	redirectCookie, err := r.Cookie("authRedirectTo")
	if err != nil {
		if !errors.Is(err, http.ErrNoCookie) {
			slog.With("error", err).Error("Failed to get redirect cookie")
			LoginError(w, r, "Failed to get redirect cookie")
			return
		}
	} else if redirectCookie.Value != "" {
		redirectTo = redirectCookie.Value
		if !IsHXI2BaseDomain(redirectTo) {
			redirectTo = ConfigDefaultLoginRedirect
			slog.With("redirectTo", redirectTo).Error("Tried to redirect to a different TLD")
		}
	}

	if clientState != r.URL.Query().Get("state") {
		slog.With("clientState", clientState, "serverState", r.URL.Query().Get("state")).Error("State mismatch")
		LoginError(w, r, "State mismatch")
		return
	}

	code := r.URL.Query().Get("code")

	token, err := discordOauthConfig.Exchange(r.Context(), code)

	if err != nil {
		slog.With("error", err).Error("Failed to exchange code for token")
		LoginError(w, r, "Failed to exchange code for token")
		return
	}
	redirectCookOpts := &ggu.OverwriteCookieOptions{
		Path: ggu.StringPtr("/api/discord_callback"),
	}
	// remove state and redirect cookies
	http.SetCookie(w, ggu.GenerateCookieObject("state", "", -1*time.Minute, nil))
	http.SetCookie(w, ggu.GenerateCookieObject("authRedirectTo", "", -1*time.Minute, redirectCookOpts))

	// get user info
	resp, body, err := ggu.GetWithAuthorizationHeader("https://discord.com/api/users/@me", fmt.Sprintf("%s %s", token.TokenType, token.AccessToken))
	if err != nil {
		slog.With("error", err).Error("Failed to get user info")
		LoginError(w, r, "Failed to get user info")
		return
	}

	if resp.StatusCode != http.StatusOK {
		slog.With("status", resp.Status).Error("Failed to get user info")
		LoginError(w, r, "Failed to get user info")
		return
	}

	dai := &DiscordAuthorizationInfo{}
	err = json.Unmarshal([]byte(body), dai)
	if err != nil {
		slog.With("error", err).Error("Failed to unmarshal json")
		LoginError(w, r, "Failed to unmarshal json")
		return
	}

	if dai.ID == "" {
		slog.Error("ID is empty")
		LoginError(w, r, "ID is empty")
		return
	}

	dbUser, err := DB.GetDBUserByDiscordID(dai.ID)
	if err != nil {
		slog.With("error", err).Error("Failed to get user by discord ID")
		LoginError(w, r, "Failed to get user by discord ID")
		return
	}
	if dbUser == nil {
		slog.With("discordID", dai.ID).Info("dbUser is nil, but no error thrown")
		LoginError(w, r, "User not found")
		return
	}

	if dai.Username != dbUser.Username {
		dbUser.Username = dai.Username
		err = DB.UpdateUser(dbUser)
		if err != nil {
			slog.With("error", err, "discordID", dai.ID, "newUsername", dai.Username).Error("Failed to update username on login")
			return
		}
	}

	if !setAuthCookies(w, r, dbUser) {
		return
	}

	http.Redirect(w, r, redirectTo, http.StatusFound)
}

func setAuthCookies(w http.ResponseWriter, r *http.Request, dbUser *DBUser) bool {
	cl := dbUser.GetNewJWTClaims()

	tokenString, err := jwtManager.GenerateToken(cl)
	if err != nil {
		slog.With("error", err).Error("Failed to generate token")
		LoginError(w, r, "Failed to generate token")
		return false
	}

	refreshToken, err := JWTGenerateRefreshToken()
	if err != nil {
		slog.With("error", err).Error("Failed to generate refresh token")
		LoginError(w, r, "Failed to generate refresh token")
		return false
	}

	jti := cl.ID
	if jti == "" {
		slog.With("claims", cl).Error("Generated JTI is empty - this isn't normal at all")
		LoginError(w, r, "JTI is empty")
		return false
	}

	err = DB.AddRefreshTokenPair(dbUser.ID, refreshToken, jti)
	if err != nil {
		slog.With("error", err).Error("Failed to add refresh token pair")
		LoginError(w, r, "Failed to add refresh token pair")
		return false
	}

	smallData := dbUser.GetSmallData()

	smallDataBytes, err := json.Marshal(smallData)
	if err != nil {
		slog.With("error", err).Error("Failed to marshal small data")
		LoginError(w, r, "Failed to marshal small data")
		return false
	}

	smallDataBase64 := base64.StdEncoding.EncodeToString(smallDataBytes)

	if currentRefreshToken, err := r.Cookie(ggu.CookieRefresh); err == nil && currentRefreshToken.Value != "" {
		err = DB.DeleteRefreshToken(currentRefreshToken.Value)
		if err != nil {
			slog.With("error", err).Error("Failed to delete refresh token pair")
		}
	}

	// set the cookies
	http.SetCookie(w, ggu.GenerateCookieObject(ggu.CookieToken, tokenString, jwtManager.RefreshTokenValidityDuration, &ggu.OverwriteCookieOptions{
		Domain: ggu.StringPtr(HXI2CookiesDomain),
		Path:   ggu.StringPtr("/"),
	}))
	http.SetCookie(w, ggu.GenerateCookieObject(ggu.CookieRefresh, refreshToken, jwtManager.RefreshTokenValidityDuration, &ggu.OverwriteCookieOptions{
		Path:   ggu.StringPtr("/"),
		Domain: ggu.StringPtr(HXI2CookiesDomain),
	}))
	c := ggu.GenerateCookieObject(ggu.CookieSmallData, smallDataBase64, jwtManager.RefreshTokenValidityDuration, &ggu.OverwriteCookieOptions{
		Domain:   ggu.StringPtr(HXI2CookiesDomain),
		HttpOnly: ggu.BoolPtr(false),
		Path:     ggu.StringPtr("/"),
	})
	http.SetCookie(w, c)

	return true
}

// Unused, prefer the project API, and serve the list from the microservices
func HandleListUsers(w http.ResponseWriter, r *http.Request) {
	user, err := authManager.AuthenticateHTTPRequest(w, r, true)
	if err != nil {
		return
	}
	if !user.HasPermission(ggu.RoleAdmin) {
		http.Error(w, "Unauthorized", http.StatusUnauthorized)
		return
	}

	users, err := DB.ListUsers()
	if err != nil {
		slog.With("error", err).Error("Failed to list users")
		http.Error(w, "Failed to list users", http.StatusInternalServerError)
		return
	}

	usersBytes, err := json.Marshal(users)
	if err != nil {
		slog.With("error", err).Error("Failed to marshal users")
		http.Error(w, "Failed to marshal users", http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")

	_, err = w.Write(usersBytes)
	if err != nil {
		slog.With("error", err).Error("Failed to write users to response")
	}
	// w.WriteHeader(http.StatusOK)
}

func HandleRenewToken(w http.ResponseWriter, r *http.Request) {
	if !checkProjectApiAuth(w, r, ggu.APIRoleAuthentication) {
		return
	}
	var renewRequest ggu.AuthRenewalRequest
	err := json.NewDecoder(r.Body).Decode(&renewRequest)
	if err != nil {
		http.Error(w, "Failed to decode renew request", http.StatusBadRequest)
		return
	}

	if renewRequest.Token == "" || renewRequest.RefreshToken == "" {
		http.Error(w, "Token or refreshToken is empty", http.StatusBadRequest)
		return
	}

	// slog.With("token", renewRequest.Token, "refreshToken", renewRequest.RefreshToken).Info("Renewing token")

	// verify token
	_, err = jwt.Parse([]byte(renewRequest.Token), jwtManager.verifier)
	if err != nil {
		http.Error(w, "Failed to verify token", http.StatusUnauthorized)
		return
	}

	res, err := RenewTokenActionner(renewRequest.Token, renewRequest.RefreshToken)

	if err != nil {
		http.Error(w, "Failed to renew token", http.StatusInternalServerError)
		return
	}

	resBytes, err := json.Marshal(res)
	if err != nil {
		http.Error(w, "Failed to marshal response", http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	_, err = w.Write(resBytes)
	if err != nil {
		slog.With("error", err).Error("Failed to write response")
	}

}

func HandleTempLogin(w http.ResponseWriter, r *http.Request) {
	username := r.URL.Query().Get("username")
	code := r.URL.Query().Get("code")
	if username == "" || code == "" {
		http.Error(w, "Username or code is empty", http.StatusBadRequest)
		return
	}
	tempo, err := DB.CheckTempCode(username, code)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	if tempo == nil {
		http.Error(w, "Temporary code not found or expired", http.StatusUnauthorized)
		return
	}

	newClaims, err := tempo.GetNewClaims()
	if err != nil {
		slog.With("error", err).Error("Failed to get new claims from temporary code")
		http.Error(w, "Failed to get new claims from temporary code", http.StatusInternalServerError)
		return
	}

	newToken, err := jwtManager.builder.Build(newClaims)
	if err != nil {
		slog.With("error", err).Error("Failed to build new token from temporary code")
		http.Error(w, "Failed to build new token from temporary code", http.StatusInternalServerError)
		return
	}
	http.SetCookie(w, ggu.GenerateCookieObject(ggu.CookieToken, newToken.String(), newClaims.ExpiresAt.Add(1*time.Hour).Sub(time.Now().UTC()), &ggu.OverwriteCookieOptions{
		Domain: ggu.StringPtr(HXI2CookiesDomain),
		Path:   ggu.StringPtr("/"),
	}))

	redirectTo := ConfigDefaultLoginRedirect
	redirectCookie, err := r.Cookie("authRedirectTo")
	if err != nil {
		if !errors.Is(err, http.ErrNoCookie) {
			slog.With("error", err).Error("Failed to get redirect cookie")
			LoginError(w, r, "Failed to get redirect cookie")
			return
		}
	} else if redirectCookie.Value != "" {
		redirectTo = redirectCookie.Value
		if !IsHXI2BaseDomain(redirectTo) {
			redirectTo = ConfigDefaultLoginRedirect
			slog.With("redirectTo", redirectTo).Error("Tried to redirect to a different TLD")
		}
	}

	http.Redirect(w, r, redirectTo, http.StatusFound)
}

func HandleTempRenew(w http.ResponseWriter, r *http.Request) {
	if !checkProjectApiAuth(w, r, ggu.APIRoleAuthentication) {
		return
	}
	var renewRequest ggu.AuthRenewalRequest
	err := json.NewDecoder(r.Body).Decode(&renewRequest)
	if err != nil {
		http.Error(w, "Failed to decode renew request", http.StatusBadRequest)
		return
	}

	if renewRequest.Token == "" {
		http.Error(w, "Token is empty", http.StatusBadRequest)
		return
	}

	token := renewRequest.Token

	var cl *ggu.HXI2JWTClaims
	t, err := jwt.Parse([]byte(token), jwtManager.verifier)
	if err != nil {
		http.Error(w, "Failed to verify token", http.StatusUnauthorized)
		return
	}
	err = t.DecodeClaims(&cl)
	if err != nil {
		http.Error(w, "Failed to decode claims", http.StatusUnauthorized)
		return
	}

	if !cl.Temporary {
		http.Error(w, "Token is not temporary", http.StatusBadRequest)
		return
	}

	tempo, err := DB.GetTempFromUsername(cl.Username)
	if err != nil || tempo == nil {
		slog.With("error", err).Error("Failed to get temporary code from username")
		http.Error(w, "Failed to get temporary code from username", http.StatusInternalServerError)
		return
	}

	if tempo.RecheckAfter > 0 {
		if tempo.ExpiresAt.Unix() < time.Now().Unix() {
			slog.With("username", cl.Username).Info("Temporary code expired")
			http.Error(w, "Temporary code expired", http.StatusUnauthorized)
			return
		}

		newClaims, err := tempo.GetNewClaims()
		if err != nil {
			http.Error(w, "Failed to get new claims", http.StatusInternalServerError)
			return
		}

		newToken, err := jwtManager.builder.Build(newClaims)
		if err != nil {
			slog.With("error", err).Error("Failed to build new token")
			http.Error(w, "Failed to build new token", http.StatusInternalServerError)
			return
		}
		token = newToken.String()
	}

	w.Header().Set("Content-Type", "text/plain")
	_, err = w.Write([]byte(token))
	if err != nil {
		slog.With("error", err).Error("Failed to write token to response")
	}
}

func IsHXI2BaseDomain(rawURL string) bool {
	parsedURL, err := url.Parse(rawURL)
	if err != nil {
		return false
	}
	host := parsedURL.Hostname()
	baseDomain, err := publicsuffix.EffectiveTLDPlusOne(host)
	if err != nil {
		return false
	}
	return baseDomain == HXI2TLD
}
