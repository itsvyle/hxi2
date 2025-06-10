package main

import (
	"crypto/ecdsa"
	"crypto/elliptic"
	"crypto/rand"
	"crypto/x509"
	"encoding/pem"
	"fmt"
)

func GenerateECDSAKeys() (*ecdsa.PrivateKey, *ecdsa.PublicKey, error) {
	privateKey, err := ecdsa.GenerateKey(elliptic.P256(), rand.Reader)
	if err != nil {
		return nil, nil, err
	}
	return privateKey, &privateKey.PublicKey, nil
}

// ExportKeyAsPEM encodes an ECDSA key (private or public) to PEM format for storage or sharing.
func ExportKeyAsPEM(key interface{}) ([]byte, error) {
	var keyBytes []byte
	var err error
	if privateKey, ok := key.(*ecdsa.PrivateKey); ok {
		keyBytes, err = x509.MarshalECPrivateKey(privateKey)
	} else if publicKey, ok := key.(*ecdsa.PublicKey); ok {
		keyBytes, err = x509.MarshalPKIXPublicKey(publicKey)
	} else {
		return nil, fmt.Errorf("unsupported key type")
	}
	if err != nil {
		return nil, err
	}

	keyType := "PUBLIC KEY"
	if _, ok := key.(*ecdsa.PrivateKey); ok {
		keyType = "PRIVATE KEY"
	}

	block := &pem.Block{
		Type:  keyType,
		Bytes: keyBytes,
	}
	return pem.EncodeToMemory(block), nil
}

func LoadECDSAPrivateKeyFromPEM(pemBytes []byte) (*ecdsa.PrivateKey, error) {
	block, _ := pem.Decode(pemBytes)
	if block == nil {
		return nil, fmt.Errorf("failed to decode PEM block")
	}
	return x509.ParseECPrivateKey(block.Bytes)
}
