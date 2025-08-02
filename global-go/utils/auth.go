package globalgoutils

import (
	"bytes"
	"crypto/ecdsa"
	"crypto/x509"
	"database/sql"
	"encoding/base64"
	"encoding/json"
	"encoding/pem"
	"fmt"
	"io"
	"log/slog"
	"net/http"
	"net/url"
	"os"
	"time"

	"github.com/cristalhq/jwt/v5"
)

const CookieToken = "HXI2_TOKEN"
const CookieRefresh = "HXI2_REFRESH"
const CookieSmallData = "HXI2_SMALL_DATA"
const AuthRemotePublicKeyPath = "/api/public-key"
const AuthRemoteRenewPath = "/api/renew"

const (
	RoleAdmin   = 0b1000
	RoleStudent = 0b0001
)

const (
	APIRoleListUsers      = 1 << 1 // 2
	APIRoleAuthentication = 1 << 2 // 4
)

// this is stored in a cookie accessible by javascript, and is used only to display client side information
type SmallData struct {
	UserID      int64  `json:"userID"`
	Username    string `json:"username"`
	FirstName   string `json:"firstName"`
	LastName    string `json:"lastName"`
	Permissions int    `json:"permissions"`
	Promotion   int    `json:"promotion"`
}

type HXI2JWTClaims struct {
	jwt.RegisteredClaims
	Username    string `json:"username"`
	Permissions int    `json:"permissions"`
	Promotion   int    `json:"promotion"`
}

func (c *HXI2JWTClaims) IDInt() int64 {
	id, _ := ParseInt64(c.Subject)
	return id
}

func (c *HXI2JWTClaims) HasPermission(permission int) bool {
	return BitfieldHasPermission(c.Permissions, permission)
}

func (c *HXI2JWTClaims) IsAdmin() bool {
	return c.HasPermission(RoleAdmin)
}

func (c *HXI2JWTClaims) IsStudent() bool {
	return c.HasPermission(RoleStudent)
}

func (c *HXI2JWTClaims) CheckPermHTTP(w http.ResponseWriter, permission int) bool {
	if !c.HasPermission(permission) {
		http.Error(w, "Unauthorized", http.StatusUnauthorized)
		return false
	}
	return true
}

func BitfieldHasPermission(bitfield int, permission int) bool {
	return bitfield&permission != 0
}

type AuthManager struct {
	AutoFetchKey  bool
	LoginPageURL  string
	AuthURL       string
	AuthEndpoint  string
	CookieDomain  string
	verifier      jwt.Verifier
	Logger        *slog.Logger
	projectAPIKey string
	// the oldToken has already been verified for the signature
	RenewToken func(a *AuthManager, oldToken, refreshToken string) (res *AuthRenewalResponse, err error)
}

func NewAuthManagerFromEnv() (*AuthManager, error) {
	Hxi2AuthURL := os.Getenv("HXI2_AUTH_URL")
	if Hxi2AuthURL == "" {
		return nil, fmt.Errorf("HXI2_AUTH_URL not set")
	}

	Hxi2AuthEndpoint := os.Getenv("HXI2_AUTH_ENDPOINT")
	if Hxi2AuthEndpoint == "" {
		return nil, fmt.Errorf("HXI2_AUTH_ENDPOINT not set")
	}

	HXI2AuthCookieDomain := os.Getenv("HXI2_COOKIES_DOMAIN")
	if HXI2AuthCookieDomain == "" {
		return nil, fmt.Errorf("HXI2_COOKIES_DOMAIN not set")
	}
	HXI2PublicKeyPEM := os.Getenv("HXI2_PUBLIC_KEY_PEM")

	var err error
	var a *AuthManager = &AuthManager{
		Logger:       GetAuthLogger(),
		AutoFetchKey: false,
		AuthURL:      Hxi2AuthURL,
		AuthEndpoint: Hxi2AuthEndpoint,
		LoginPageURL: Hxi2AuthURL + "/login",
		CookieDomain: HXI2AuthCookieDomain,
		RenewToken:   DefaultRenewToken,
	}
	HXI2ProjectAPIKey := os.Getenv("HXI2_PROJECT_API_KEY")
	if HXI2ProjectAPIKey != "" {
		a.projectAPIKey = HXI2ProjectAPIKey
		a.Logger.Debug("Project API key set")
	} else {
		a.Logger.Info("Couldn't find a HXI2_PROJECT_API_KEY; you will need one to make requests to the authentication api in production")
	}

	if HXI2PublicKeyPEM == "" {
		a, err = NewAuthManagerAutoFetchKey(a)
	} else {
		a, err = NewAuthManagerPublicKey(a, HXI2PublicKeyPEM)
	}
	if err != nil {
		return nil, err
	}

	return a, nil

}

func loadECDSAPublicKey(pemData []byte) (*ecdsa.PublicKey, error) {
	// Decode the PEM block
	block, _ := pem.Decode(pemData)
	if block == nil || block.Type != "PUBLIC KEY" {
		return nil, fmt.Errorf("failed to decode PEM block or incorrect type: %s", block.Type)
	}

	// Parse the public key
	pubKey, err := x509.ParsePKIXPublicKey(block.Bytes)
	if err != nil {
		return nil, fmt.Errorf("failed to parse public key: %w", err)
	}

	// Assert that the parsed key is an ECDSA public key
	ecdsaPubKey, ok := pubKey.(*ecdsa.PublicKey)
	if !ok {
		return nil, fmt.Errorf("public key is not of type ECDSA")
	}

	return ecdsaPubKey, nil
}

func GetAuthLogger() *slog.Logger {
	return GetServiceSpecificLogger("AUTHEN", "\033[38;2;100;0;150m")
}

func NewAuthManagerPublicKey(a *AuthManager, publicKeyPEM string) (*AuthManager, error) {
	var err error
	publicKey, err := loadECDSAPublicKey([]byte(publicKeyPEM))
	if err != nil {
		return nil, err
	}

	a.verifier, err = jwt.NewVerifierES(jwt.ES256, publicKey)
	if err != nil {
		return nil, err
	}

	return a, nil
}

func NewAuthManagerAutoFetchKey(a *AuthManager) (*AuthManager, error) {
	const retryIntervalSeconds = 2

	a.AutoFetchKey = true

	for try := 0; try < 5; try++ {
		err := a.FetchKey()
		if err != nil {
			if try == 4 {
				return nil, err
			}
			a.Logger.With("error", err, "try", try).Error("Failed to fetch public key, retrying in " + fmt.Sprint(retryIntervalSeconds) + " seconds")
			time.Sleep(retryIntervalSeconds * time.Second)
		} else {
			a.Logger.Info("Fetched initial public key")
			break
		}
	}

	ticker := time.NewTicker(5 * time.Minute)
	quit := make(chan struct{})
	go func() {
		for {
			select {
			case <-ticker.C:
				err := a.FetchKey()
				if err != nil {
					a.Logger.With("error", err).Error("Failed to refetch public authentication key")
				}
			case <-quit:
				ticker.Stop()
				return
			}
		}
	}()

	return a, nil
}

func (a *AuthManager) FetchKey() error {
	// Fetch the public key from the remote URL
	if !a.AutoFetchKey {
		panic("FetchKey called on non-auto-fetch key AuthManager")
	}
	if a.AuthEndpoint == "" {
		panic("RemoteURL not set")
	}
	fetchURL := a.AuthEndpoint + AuthRemotePublicKeyPath

	req, err := http.NewRequest(http.MethodGet, fetchURL, nil)
	if err != nil {
		a.Logger.With("error", err, "url", fetchURL).Error("Failed to create project request")
		return err
	}

	req.Header.Set("Content-Type", "application/json")
	if a.projectAPIKey != "" {
		req.Header.Set("Authorization", "Bearer "+a.projectAPIKey)
	}

	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		return fmt.Errorf("failed to fetch public key: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("failed to fetch public key: status code %d", resp.StatusCode)
	}

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return fmt.Errorf("failed to read response body: %w", err)
	}

	publicKey, err := loadECDSAPublicKey(body)
	if err != nil {
		return fmt.Errorf("failed to load ECDSA public key: %w", err)
	}

	v, err := jwt.NewVerifierES(jwt.ES256, publicKey)
	if err != nil {
		return fmt.Errorf("failed to create JWT verifier: %w", err)
	}

	a.verifier = v

	return nil
}

// doesn't check expiry
func (a *AuthManager) VerifyTokenNoDate(token string) (*HXI2JWTClaims, error) {
	if a.verifier == nil {
		return nil, fmt.Errorf("verifier not set")
	}
	t, err := jwt.Parse([]byte(token), a.verifier)
	if err != nil {
		return nil, err
	}

	claims := &HXI2JWTClaims{}
	err = t.DecodeClaims(&claims)
	if err != nil {
		return nil, err
	}

	return claims, nil
}

type AuthRenewalRequest struct {
	Token        string `json:"token"`
	RefreshToken string `json:"refresh_token"`
}

type AuthRenewalResponse struct {
	Token                 string     `json:"token"`
	RefreshToken          string     `json:"refresh_token"`
	RefreshTokenExpiresAt time.Time  `json:"refresh_token_expires_at"`
	SmallData             *SmallData `json:"small_data"`
}

func DefaultRenewToken(a *AuthManager, token, refreshToken string) (*AuthRenewalResponse, error) {
	data, err := json.Marshal(AuthRenewalRequest{
		Token:        token,
		RefreshToken: refreshToken,
	})
	if err != nil {
		a.Logger.With("error", err, "token", token, "refreshToken", refreshToken).Error("Failed to marshal renewal request data")
		return nil, err
	}

	url := a.AuthEndpoint + AuthRemoteRenewPath
	req, err := http.NewRequest(http.MethodPost, url, bytes.NewBuffer(data))
	if err != nil {
		a.Logger.With("error", err, "url", url).Error("Failed to create request to renew token")
		return nil, err
	}

	req.Header.Set("Content-Type", "application/json")
	if a.projectAPIKey != "" {
		req.Header.Set("Authorization", "Bearer "+a.projectAPIKey)
	}

	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		a.Logger.With("error", err, "url", url).Error("Failed to send request to renew token")
		return nil, err
	}
	defer resp.Body.Close()

	bodyBytes, err := io.ReadAll(resp.Body)
	if err != nil {
		a.Logger.With("error", err, "url", url).Error("Failed to read response body")
		return nil, err
	}

	if resp.StatusCode != http.StatusOK {
		err = fmt.Errorf("status code not OK: %s", resp.Status)
		a.Logger.With("status", resp.Status, "url", url, "resBody", string(bodyBytes)).Error("Failed to renew token")
		return nil, err
	}

	var renewalResponse AuthRenewalResponse
	err = json.Unmarshal(bodyBytes, &renewalResponse)
	if err != nil {
		a.Logger.With("error", err, "url", url, "resBody", string(bodyBytes)).Error("Failed to unmarshal response body")
		return nil, err
	}

	return &renewalResponse, nil
}

type ProcessRequestAuthResponse struct {
	Claims          *HXI2JWTClaims
	RenewalResponse *AuthRenewalResponse
}

func (a *AuthManager) ProcessRequestAuth(providedToken string, providedRefreshToken string) (*ProcessRequestAuthResponse, error) {
	claims, err := a.VerifyTokenNoDate(providedToken)
	if err != nil {
		return nil, fmt.Errorf("failed to verify token: %w", err)
	}
	var renewalResponse *AuthRenewalResponse = nil
	if !claims.IsValidAt(time.Now().UTC()) {
		if providedRefreshToken == "" {
			return nil, fmt.Errorf("failed to get refresh cookie: %w", err)
		}
		renewalResponse, err = a.RenewToken(a, providedToken, providedRefreshToken)
		if err != nil {
			return nil, fmt.Errorf("failed to renew token")
		}

		claims, err = a.VerifyTokenNoDate(renewalResponse.Token)
		if err != nil || !claims.IsValidAt(time.Now().UTC()) {
			a.Logger.With("error", err).Error("Failed to verify renewed token")
			return nil, fmt.Errorf("failed to verify renewed token")
		}
	}

	return &ProcessRequestAuthResponse{
		Claims:          claims,
		RenewalResponse: renewalResponse,
	}, nil
}

// if isAPI it won't redirect, it will return a 401
// if this function returns an error, it has already sent a response, just exit your handler
func (a *AuthManager) AuthenticateHTTPRequest(w http.ResponseWriter, r *http.Request, isAPI bool) (*HXI2JWTClaims, error) {
	redirect := func(e error) (*HXI2JWTClaims, error) {
		if isAPI {
			http.Error(w, "Unauthorized", http.StatusUnauthorized)
			return nil, e
		}

		scheme := "http"
		if r.TLS != nil {
			scheme = "https"
		}

		fullURL := fmt.Sprintf("%s://%s%s", scheme, r.Host, r.URL.RequestURI())

		http.Redirect(w, r, a.LoginPageURL+"?redirectTo="+url.QueryEscape(fullURL), http.StatusTemporaryRedirect)
		return nil, e
	}
	tokenCookie, err := r.Cookie(CookieToken)
	if err != nil || tokenCookie == nil || tokenCookie.Value == "" {
		return redirect(fmt.Errorf("no token cookie"))
	}

	providedRefresh := ""
	refreshCookie, err := r.Cookie(CookieRefresh)
	if err == nil && refreshCookie != nil {
		providedRefresh = refreshCookie.Value
	}

	res, err := a.ProcessRequestAuth(tokenCookie.Value, providedRefresh)
	if err != nil {
		a.Logger.With("error", err).Error("Failed to process request auth")
		return redirect(err)
	}
	if res == nil {
		a.Logger.Error("ProcessRequestAuth returned nil")
		return redirect(fmt.Errorf("ProcessRequestAuth returned nil"))
	}

	if res.RenewalResponse != nil {
		authTokenRes := res.RenewalResponse
		cookieDuration := time.Until(authTokenRes.RefreshTokenExpiresAt)

		http.SetCookie(w, GenerateCookieObject(CookieToken, authTokenRes.Token, cookieDuration, &OverwriteCookieOptions{
			Path:   StringPtr("/"),
			Domain: StringPtr(a.CookieDomain),
		}))
		http.SetCookie(w, GenerateCookieObject(CookieRefresh, authTokenRes.RefreshToken, cookieDuration, &OverwriteCookieOptions{
			Path:   StringPtr("/"),
			Domain: StringPtr(a.CookieDomain),
		}))
		smallDataBytes, err := json.Marshal(authTokenRes.SmallData)
		if err != nil {
			a.Logger.With("error", err).Error("Failed to marshal small data")
			return nil, err
		}

		smallDataBase64 := base64.StdEncoding.EncodeToString(smallDataBytes)
		c := GenerateCookieObject(CookieSmallData, smallDataBase64, cookieDuration, &OverwriteCookieOptions{
			Domain:   StringPtr(a.CookieDomain),
			HttpOnly: BoolPtr(false),
			Path:     StringPtr("/"),
		})
		http.SetCookie(w, c)
	}

	return res.Claims, nil
}

type ProjectUser struct {
	ID          int64          `db:"ID" json:"id"`
	Username    string         `db:"username" json:"username"`
	FirstName   string         `db:"first_name" json:"firstName"`
	LastName    sql.NullString `db:"last_name" json:"lastName"`
	DiscordID   string         `db:"discord_id" json:"discordID"`
	Promotion   int            `db:"promotion" json:"promotion"`
	Permissions int            `db:"permissions" json:"permissions"`
}

func (p *ProjectUser) DisplayName() string {
	if p.FirstName == "" {
		return p.Username
	}
	if p.LastName.Valid && p.LastName.String != "" {
		return p.FirstName + " " + p.LastName.String[0:1] + "."
	}
	return p.FirstName
}

func (a *AuthManager) ProjectGetRequest(url string) ([]byte, error) {
	req, err := http.NewRequest(http.MethodGet, url, nil)
	if err != nil {
		a.Logger.With("error", err, "url", url).Error("Failed to create project request")
		return nil, err
	}

	req.Header.Set("Content-Type", "application/json")
	if a.projectAPIKey != "" {
		req.Header.Set("Authorization", "Bearer "+a.projectAPIKey)
	}

	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		a.Logger.With("error", err, "url", url).Error("Failed to send project request")
		return nil, err
	}
	defer resp.Body.Close()

	bodyBytes, err := io.ReadAll(resp.Body)
	if err != nil {
		a.Logger.With("error", err, "url", url).Error("Failed to read response body")
		return nil, err
	}

	if resp.StatusCode != http.StatusOK {
		err = fmt.Errorf("status code not OK: %s", resp.Status)
		a.Logger.With("status", resp.Status, "url", url, "resBody", string(bodyBytes)).Error("Failed to execute project request")
		return nil, err
	}

	return bodyBytes, nil
}

func (a *AuthManager) ProjectListUsers() ([]ProjectUser, error) {
	url := a.AuthEndpoint + "/api/project/list_users"

	bodyBytes, err := a.ProjectGetRequest(url)
	if err != nil {
		a.Logger.With("error", err, "url", url, "resBody", string(bodyBytes)).Error("Failed to get project users")
		return nil, err
	}

	var projectUsers []ProjectUser
	err = json.Unmarshal(bodyBytes, &projectUsers)
	if err != nil {
		a.Logger.With("error", err, "url", url, "resBody", string(bodyBytes)).Error("Failed to unmarshal response body")
		return nil, err
	}

	return projectUsers, nil
}
