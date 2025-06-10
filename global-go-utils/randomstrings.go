package globalgoutils

import (
	"crypto/rand"
	"encoding/base64"
	"encoding/binary"
	"math/big"
)

// GenerateRandomBytes returns securely generated random bytes.
// It will return an error if the system's secure random
// number generator fails to function correctly, in which
// case the caller should not continue.
func GenerateRandomBytes(n int) ([]byte, error) {
	b := make([]byte, n)
	_, err := rand.Read(b)
	// Note that err == nil only if we read len(b) bytes.
	if err != nil {
		return nil, err
	}

	return b, nil
}

// GenerateRandomString returns a URL-safe, base64 encoded
// securely generated random string; it's made up of `s` base bytes.
// It will return an error if the system's secure random
// number generator fails to function correctly, in which
// case the caller should not continue.
func GenerateRandomString(s int) (string, error) {
	b, err := GenerateRandomBytes(s)
	return base64.URLEncoding.EncodeToString(b), err
}

// guarantees the first bit is a 1
func Generate32BitsNumber() (int64, error) {
	var b [4]byte
	_, err := rand.Read(b[:])
	if err != nil {
		return 0, err
	}

	// guarantees the first bit is a 1
	num := binary.BigEndian.Uint32(b[:])
	num |= 1 << 31

	return int64(num), nil
}

func Generate6DigitNumber() (string, error) {
	const length = 6
	digits := make([]byte, length)
	for i := 0; i < length; i++ {
		n, err := rand.Int(rand.Reader, big.NewInt(10)) // digits 0–9
		if err != nil {
			return "", err
		}
		if i == 0 && n.Int64() == 0 {
			n = big.NewInt(1)
		}
		digits[i] = byte('0') + byte(n.Int64())
	}
	return string(digits), nil
}
