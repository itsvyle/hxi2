package main

import (
	"crypto/sha256"
	"database/sql"
	"encoding/hex"
	"errors"
	"fmt"
	"log/slog"
	"time"

	"github.com/cristalhq/jwt/v5"
	ggu "github.com/itsvyle/hxi2/global-go/utils"
	"github.com/jmoiron/sqlx"
)

var OneTimeCodeValidityDuration = 10 * time.Minute

type DatabaseManager struct {
	DB     *sqlx.DB
	logger *slog.Logger
}

var DB *DatabaseManager

type DBUser struct {
	ID                  int64          `db:"ID" json:"id"`
	Username            string         `db:"username" json:"username"`
	FirstName           string         `db:"first_name" json:"firstName"`
	LastName            sql.NullString `db:"last_name" json:"lastName"`
	DiscordID           string         `db:"discord_id" json:"discordID"`
	AccountCreatedDate  time.Time      `db:"account_created_date" json:"-"`
	AccountModifiedDate time.Time      `db:"account_modified_date" json:"-"`
	Promotion           int            `db:"promotion" json:"promotion"`
	Permissions         int            `db:"permissions" json:"permissions"`
}

func (u *DBUser) CheckSchema() error {
	if u.ID == 0 {
		return errors.New("ID is missing")
	}
	if u.FirstName == "" {
		return errors.New("first_name is missing")
	}
	if u.DiscordID == "" {
		return errors.New("discord_id is missing")
	}
	if u.AccountCreatedDate.IsZero() {
		return errors.New("account_created_date is missing")
	}
	if u.AccountModifiedDate.IsZero() {
		return errors.New("account_modified_date is missing")
	}
	return nil
}

func (u *DBUser) GetNewJWTClaims() *ggu.HXI2JWTClaims {
	return &ggu.HXI2JWTClaims{
		RegisteredClaims: jwt.RegisteredClaims{
			Subject: fmt.Sprintf("%d", u.ID),
		},
		Username:    u.Username,
		Permissions: u.Permissions,
		Promotion:   u.Promotion,
	}
}

func (u *DBUser) GetSmallData() *ggu.SmallData {
	return &ggu.SmallData{
		UserID:      u.ID,
		Username:    u.Username,
		FirstName:   u.FirstName,
		LastName:    u.LastName.String,
		Permissions: u.Permissions,
		Promotion:   u.Promotion,
	}
}

type DBRefreshToken struct {
	AssociatedUserID int64     `db:"associated_user_id"`
	RefreshTokenHash string    `db:"refresh_token_hash"`
	JTIHash          string    `db:"jti_hash"`
	CreatedAt        time.Time `db:"created_at"`
}

func (rt *DBRefreshToken) CheckSchema() error {
	if rt.AssociatedUserID == 0 {
		return errors.New("associated_user_id is missing")
	}
	if rt.RefreshTokenHash == "" {
		return errors.New("refresh_token_hash is missing")
	}
	if rt.JTIHash == "" {
		return errors.New("jti_hash is missing")
	}
	if rt.CreatedAt.IsZero() {
		return errors.New("created_at is missing")
	}
	return nil
}

func (db *DatabaseManager) AddNewDBUser(user *DBUser) error {
	var err error
	if user.ID == 0 {
		user.ID, err = ggu.Generate32BitsNumber()
		if err != nil {
			return err
		}
	}
	user.AccountModifiedDate = time.Now().UTC()
	user.AccountCreatedDate = time.Now().UTC()

	if err := user.CheckSchema(); err != nil {
		return err
	}

	_, err = db.DB.NamedExec(`
		INSERT INTO users (ID, first_name, last_name, discord_id, account_created_date, account_modified_date, promotion, permissions)
		VALUES (:ID, :first_name, :last_name, :discord_id, :account_created_date, :account_modified_date, :promotion, :permissions)
	`, user)
	return err
}

func (db *DatabaseManager) ListUsers() ([]DBUser, error) {
	users := []DBUser{}
	err := db.DB.Select(&users, "SELECT * FROM users")
	return users, err
}

func (db *DatabaseManager) UpdateUser(user *DBUser) error {
	if user.ID == 0 {
		return errors.New("no ID field on the user object")
	}
	user.AccountModifiedDate = time.Now().UTC()
	if err := user.CheckSchema(); err != nil {
		return err
	}
	_, err := db.DB.NamedExec(`
		UPDATE users
		SET first_name = :first_name, last_name = :last_name, discord_id = :discord_id, account_modified_date = :account_modified_date, promotion = :promotion, permissions = :permissions, username = :username
		WHERE ID = :ID
	`, user)
	return err
}

func (db *DatabaseManager) GetDBUserByDiscordID(discordID string) (*DBUser, error) {
	user := &DBUser{}
	err := db.DB.Get(user, "SELECT * FROM users WHERE discord_id = ?", discordID)
	return user, err
}

func (db *DatabaseManager) GetDBUserByID(userID int64) (*DBUser, error) {
	user := &DBUser{}
	err := db.DB.Get(user, "SELECT * FROM users WHERE ID = ?", userID)
	return user, err
}

func (db *DatabaseManager) AddRefreshTokenPair(userID int64, refreshToken, jti string) error {
	refreshTokenHash, err := db.hashToken(refreshToken)
	if err != nil {
		return err
	}
	jtiHash, err := db.hashToken(jti)
	if err != nil {
		return err
	}
	_, err = db.DB.Exec("INSERT INTO refresh_tokens (associated_user_id, refresh_token_hash, jti_hash, created_at) VALUES (?, ?, ?, ?)", userID, refreshTokenHash, jtiHash, time.Now().UTC())
	return err
}

func (db *DatabaseManager) DeleteRefreshTokenHash(hash string) error {
	_, err := db.DB.Exec("DELETE FROM refresh_tokens WHERE refresh_token_hash = ?", hash)
	if errors.Is(err, sql.ErrNoRows) {
		return nil
	}
	return err
}

// this first checks if the old data is valid, then issues new ones
// returns: user id, new refresh token, new jti, error
func (db *DatabaseManager) RenewRefreshToken(refreshToken, jti string) (int64, string, string, error) {
	refreshTokenHash, err := db.hashToken(refreshToken)
	if err != nil {
		return 0, "", "", err
	}
	jtiHash, err := db.hashToken(jti)
	if err != nil {
		return 0, "", "", err
	}

	var rt DBRefreshToken
	err = db.DB.Get(&rt, "SELECT * FROM refresh_tokens WHERE refresh_token_hash = ? AND jti_hash = ?", refreshTokenHash, jtiHash)
	if err != nil {
		return 0, "", "", err
	}

	if err := rt.CheckSchema(); err != nil {
		return 0, "", "", err
	}

	if time.Now().UTC().Sub(rt.CreatedAt) > jwtManager.RefreshTokenValidityDuration {
		return 0, "", "", errors.New("refresh token expired")
	}

	newRefreshToken, err := JWTGenerateRefreshToken()
	if err != nil {
		return 0, "", "", err
	}
	newJTI, err := JWTGenerateJITToken()
	if err != nil {
		return 0, "", "", err
	}

	err = db.AddRefreshTokenPair(rt.AssociatedUserID, newRefreshToken, newJTI)
	if err != nil {
		db.logger.With("error", err).Error("Failed to add new refresh token pair")
		return 0, "", "", fmt.Errorf("failed to add new refresh token pair")
	}

	err = db.DeleteRefreshTokenHash(refreshTokenHash)
	if err != nil {
		db.logger.With("error", err).Error("Failed to delete old refresh token")
		return 0, "", "", fmt.Errorf("failed to delete old refresh token")
	}

	return rt.AssociatedUserID, newRefreshToken, newJTI, nil
}

func (db *DatabaseManager) DeleteRefreshToken(refreshToken string) error {
	refreshTokenHash, err := db.hashToken(refreshToken)
	if err != nil {
		return err
	}
	return db.DeleteRefreshTokenHash(refreshTokenHash)
}

type DBApiUser struct {
	ID          int64     `db:"id" json:"id"`
	Username    string    `db:"username" json:"username"`
	Token       string    `db:"token" json:"token"`
	Permissions int       `db:"permissions" json:"permissions"`
	CreatedAt   time.Time `db:"created_at" json:"createdAt"`
	ExpiresAt   time.Time `db:"expires_at" json:"expiresAt"`
}

func (a *DBApiUser) HasPermission(permission int) bool {
	return (a.Permissions & permission) == permission
}

func (db *DatabaseManager) ListAPIUsers() ([]DBApiUser, error) {
	apiUsers := []DBApiUser{}
	err := db.DB.Select(&apiUsers, "SELECT * FROM API_TOKENS")
	if err != nil {
		db.logger.With("error", err).Error("Failed to list API users")
		return nil, err
	}
	return apiUsers, nil
}

type DBOneTimeCode struct {
	ID        int64     `db:"ID" json:"id"`
	UserID    int64     `db:"user_id" json:"userId"`
	CodeHash  string    `db:"code_hash" json:"codeHash"`
	CreatedAt time.Time `db:"created_at" json:"createdAt"`
	ExpiresAt time.Time `db:"expires_at" json:"expiresAt"`
}

func (db *DatabaseManager) StartOneTimeCodeCleanupTimer() {
	cleanupInterval := 5 * time.Minute
	timer := time.NewTimer(cleanupInterval)

	go func() {
		for {
			<-timer.C

			_, err := db.DB.Exec("DELETE FROM ONE_TIME_CODES WHERE expires_at < ?", time.Now().UTC())
			if err != nil && !errors.Is(err, sql.ErrNoRows) {
				db.logger.With("error", err).Error("Failed to clean up expired one-time codes")
			}

			timer.Reset(cleanupInterval)
		}
	}()
}

func (db *DatabaseManager) CheckOneTimeCode(code string) (int64, error) {
	if code == "" {
		return 0, errors.New("code is empty")
	}

	codeHash, err := db.hashToken(code)
	if err != nil {
		return 0, fmt.Errorf("failed to hash code: %w", err)
	}

	var oneTimeCode DBOneTimeCode
	err = db.DB.Get(&oneTimeCode, "SELECT * FROM ONE_TIME_CODES WHERE code_hash = ?", codeHash)
	if err != nil {
		if errors.Is(err, sql.ErrNoRows) {
			return 0, errors.New("invalid or expired code")
		}
		return 0, fmt.Errorf("failed to get one-time code: %w", err)
	}

	if time.Now().UTC().After(oneTimeCode.ExpiresAt) {
		return 0, errors.New("code has expired")
	}

	_, err = db.DB.Exec("DELETE FROM ONE_TIME_CODES WHERE ID = ?", oneTimeCode.ID)
	if err != nil {
		db.logger.With("error", err).Error("Failed to delete one-time code after successful check")
		return 0, fmt.Errorf("failed to delete one-time code after successful check: %w", err)
	}

	return oneTimeCode.UserID, nil
}

func (db *DatabaseManager) _createOneTimeCode(userID int64, expiresIn time.Duration) (string, error) {
	code, err := ggu.Generate6DigitNumber()
	if err != nil {
		db.logger.With("error", err).Error("Failed to generate one-time code")
		return "", fmt.Errorf("failed to generate one-time code")
	}

	codeHash, err := db.hashToken(code)
	if err != nil {
		return "", fmt.Errorf("failed to hash code: %w", err)
	}

	expiresAt := time.Now().UTC().Add(expiresIn)

	_, err = db.DB.Exec(`
		INSERT INTO ONE_TIME_CODES (user_id, code_hash, expires_at)
		VALUES (?, ?, ?)
	`, userID, codeHash, expiresAt)
	if err != nil {
		db.logger.With("error", err).Error("Failed to insert one-time code")
		return "", fmt.Errorf("failed to insert one-time code: %w", err)
	}

	return code, nil
}

func (db *DatabaseManager) CreateOneTimeCode(userID int64) (string, error) {
	if userID <= 0 {
		return "", errors.New("invalid user ID")
	}

	code, err := db._createOneTimeCode(userID, OneTimeCodeValidityDuration)
	if err != nil {
		db.logger.With("error", err).Error("Failed to create one-time code")
		return "", fmt.Errorf("failed to create one-time code")
	}

	return code, nil
}

func (db *DatabaseManager) hashToken(token string) (string, error) {
	hasher := sha256.New()
	_, err := hasher.Write([]byte(token))
	if err != nil {
		db.logger.With("error", err).Error("Failed to hash token")
		return "", err
	}
	return hex.EncodeToString(hasher.Sum(nil)), nil

}

func NullStringValue(s string) sql.NullString {
	if s == "" {
		return sql.NullString{}
	}
	return sql.NullString{
		String: s,
		Valid:  true,
	}
}
