package main

import (
	"log/slog"
	"time"

	"github.com/cristalhq/jwt/v5"
	ggu "github.com/itsvyle/hxi2/global-go-utils"
)

type JWTManager struct {
	secretKey                    string
	PublicKeyPEM                 string
	signer                       jwt.Signer
	builder                      *jwt.Builder
	verifier                     jwt.Verifier
	JWTValidityDuration          time.Duration
	RefreshTokenValidityDuration time.Duration
}

func NewJWTManager(secretKey string, validityDuration time.Duration, refreshTokenValidityDuration time.Duration) (*JWTManager, error) {
	var err error

	j := &JWTManager{
		secretKey:                    secretKey,
		JWTValidityDuration:          validityDuration,
		RefreshTokenValidityDuration: refreshTokenValidityDuration,
	}

	privateKey, err := LoadECDSAPrivateKeyFromPEM([]byte(secretKey))
	if err != nil {
		return nil, err
	}

	publicKey := &privateKey.PublicKey

	k, _ := ExportKeyAsPEM(publicKey)
	j.PublicKeyPEM = string(k)
	slog.With("publicKey", j.PublicKeyPEM).Debug("Loaded public key")

	j.signer, err = jwt.NewSignerES(jwt.ES256, privateKey)
	if err != nil {
		return nil, err
	}

	j.verifier, err = jwt.NewVerifierES(jwt.ES256, publicKey)
	if err != nil {
		return nil, err
	}

	j.builder = jwt.NewBuilder(j.signer)

	return j, nil
}

// do note that the claims are passed by reference, and therefore correctly filled with the final info, including the JIT
func (j *JWTManager) GenerateToken(claims *ggu.HXI2JWTClaims) (string, error) {
	jit, err := JWTGenerateJITToken()
	if err != nil {
		return "", err
	}
	if claims.ID == "" {
		claims.ID = jit
	}
	claims.IssuedAt = jwt.NewNumericDate(time.Now().UTC())
	claims.ExpiresAt = jwt.NewNumericDate(time.Now().UTC().Add(j.JWTValidityDuration))
	claims.NotBefore = jwt.NewNumericDate(time.Now().UTC())

	token, err := j.builder.Build(claims)
	if err != nil {
		slog.With("error", err).Error("Failed to build token")
		return "", err
	}
	return token.String(), nil
}

func (j *JWTManager) VerifyToken(token string) (*ggu.HXI2JWTClaims, error) {
	t, err := jwt.Parse([]byte(token), j.verifier)
	if err != nil {
		return nil, err
	}

	claims := &ggu.HXI2JWTClaims{}
	err = t.DecodeClaims(&claims)
	if err != nil {
		return nil, err
	}

	return claims, nil
}

func JWTGenerateJITToken() (string, error) {
	jit, err := ggu.GenerateRandomString(32)
	if err != nil {
		return "", err
	}
	return jit[:32], nil
}

func JWTGenerateRefreshToken() (string, error) {
	rt, err := ggu.GenerateRandomString(32)
	if err != nil {
		return "", err
	}
	return rt[:32], nil
}

// THE TOKEN VALIDITY HAS TO BE CHECKED BEFORE CALLING THIS FUNCTION
func RenewTokenActionner(oldToken, refreshToken string) (res *ggu.AuthRenewalResponse, err error) {
	var oldCL *ggu.HXI2JWTClaims
	t, err := jwt.ParseNoVerify([]byte(oldToken))
	if err != nil {
		slog.With("error", err).Error("Renewal: Failed to parse old token")
		return nil, err
	}
	err = t.DecodeClaims(&oldCL)
	if err != nil {
		return nil, err
	}

	jti := oldCL.ID

	userID, newRefreshToken, newJTI, err := DB.RenewRefreshToken(refreshToken, jti)
	if err != nil {
		slog.With("error", err).Error("Renewal: Failed to renew refresh token for valid input JWT")
		return nil, err
	}
	dbUser, err := DB.GetDBUserByID(userID)
	if err != nil {
		slog.With("error", err).Error("Renewal: Failed to get user by ID for a valid token renewal")
		return nil, err
	}

	cl := dbUser.GetNewJWTClaims()
	cl.ID = newJTI
	token, err := jwtManager.GenerateToken(cl)
	if err != nil {
		slog.With("error", err).Error("Failed to generate token")
		return nil, err
	}

	return &ggu.AuthRenewalResponse{
		Token:                 token,
		RefreshToken:          newRefreshToken,
		RefreshTokenExpiresAt: time.Now().UTC().Add(JWTRefreshTokenValidityDuration),
		SmallData:             dbUser.GetSmallData(),
	}, nil
}
