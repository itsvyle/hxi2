package globalgoutils

import (
	"bytes"
	"compress/gzip"
	"errors"
	"fmt"
	"hash"
	"hash/fnv"
	"io"
	"log/slog"
	"net/http"
	"strings"
	"time"
)

type OverwriteCookieOptions struct {
	Name        *string
	Value       *string
	Quoted      *bool
	Path        *string    // optional
	Domain      *string    // optional
	Expires     *time.Time // optional
	RawExpires  *string    // for reading cookies only
	MaxAge      *int
	Secure      *bool
	HttpOnly    *bool
	SameSite    *http.SameSite
	Partitioned *bool
	Raw         *string
	Unparsed    *[]string // Raw text of unparsed attribute-value pairs
}

// Generates a new cookie object with the given parameters
// if extra options is not nil, it will override the default options
func GenerateCookieObject(name string, value string, validity time.Duration, extraOptions *OverwriteCookieOptions) *http.Cookie {
	h := &http.Cookie{
		Name:     name,
		Value:    value,
		HttpOnly: true,
		SameSite: http.SameSiteLaxMode,
		Secure:   true,
		Expires:  time.Now().UTC().Add(validity),
	}
	if extraOptions != nil {
		if extraOptions.Name != nil {
			h.Name = *extraOptions.Name
		}
		if extraOptions.Value != nil {
			h.Value = *extraOptions.Value
		}
		if extraOptions.HttpOnly != nil {
			h.HttpOnly = *extraOptions.HttpOnly
		}
		if extraOptions.SameSite != nil {
			h.SameSite = *extraOptions.SameSite
		}
		if extraOptions.Secure != nil {
			h.Secure = *extraOptions.Secure
		}
		if extraOptions.Domain != nil {
			h.Domain = *extraOptions.Domain
		}
		if extraOptions.Path != nil {
			h.Path = *extraOptions.Path
		}
		if extraOptions.MaxAge != nil {
			h.MaxAge = *extraOptions.MaxAge
		}
		if extraOptions.Raw != nil {
			h.Raw = *extraOptions.Raw
		}
		if extraOptions.Unparsed != nil {
			h.Unparsed = *extraOptions.Unparsed
		}
		if extraOptions.Partitioned != nil {
			h.Partitioned = *extraOptions.Partitioned
		}
	}
	return h
}

func GetWithAuthorizationHeader(url string, authorization string) (*http.Response, string, error) {
	req, err := http.NewRequest(http.MethodGet, url, nil)
	if err != nil {
		return nil, "", err
	}
	req.Header.Add("Authorization", authorization)
	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		return nil, "", err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return resp, "", errors.New("status code not OK")
	}

	var body string
	if body, err = ReadResponseBody(resp); err != nil {
		return nil, "", err
	}

	return resp, body, nil
}

func ReadResponseBody(resp *http.Response) (string, error) {
	bodyBytes, err := io.ReadAll(resp.Body)
	if err != nil {
		return "", err
	}
	return string(bodyBytes), nil
}

func SlogHTTPInfo(r *http.Request) slog.Attr {
	if r == nil {
		return slog.Attr{} // Empty value
	}

	return slog.Attr{
		Key: "req",
		Value: slog.GroupValue(
			slog.String("path", r.URL.RequestURI()),
			slog.String("method", r.Method),
			slog.Int64("content_length", r.ContentLength),
		),
	}

}

// gzipResponseWriter wraps http.ResponseWriter to support gzip compression
type gzipResponseWriter struct {
	http.ResponseWriter
	Writer *gzip.Writer
}

func (w gzipResponseWriter) Write(b []byte) (int, error) {
	return w.Writer.Write(b)
}

func GzipMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// Check if the client supports gzip
		if !strings.Contains(r.Header.Get("Accept-Encoding"), "gzip") {
			next.ServeHTTP(w, r)
			return
		}

		// Set the response header to indicate gzip encoding
		w.Header().Set("Content-Encoding", "gzip")

		// Create a gzip writer
		gz := gzip.NewWriter(w)
		defer gz.Close()

		// Wrap the response writer
		gzw := gzipResponseWriter{ResponseWriter: w, Writer: gz}
		next.ServeHTTP(gzw, r)
	})
}

// etagResponseRecorder wraps http.ResponseWriter to capture response data
type etagResponseRecorder struct {
	http.ResponseWriter
	body       *bytes.Buffer
	hash       hash.Hash64
	status     int
	headerSent bool
}

func NewResponseRecorder(w http.ResponseWriter) *etagResponseRecorder { //nolint:revive
	return &etagResponseRecorder{
		ResponseWriter: w,
		body:           &bytes.Buffer{},
		hash:           fnv.New64a(),
		status:         http.StatusOK,
	}
}

func (r *etagResponseRecorder) WriteHeader(statusCode int) {
	if !r.headerSent {
		r.status = statusCode
		r.headerSent = true
	}
}

func (r *etagResponseRecorder) Write(b []byte) (int, error) {
	r.hash.Write(b)
	return r.body.Write(b)
}

// Only use on medium to small responses, as it buffers the entire response
// This doesn't work with gzip middleware, so not recommended to use for now
func ETagMiddleware(cacheControl StaticFileCacheControlHeader, next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		rec := NewResponseRecorder(w)
		next.ServeHTTP(rec, r)

		etag := fmt.Sprintf(`"%x"`, rec.hash.Sum64())

		if match := r.Header.Get("If-None-Match"); match == etag {
			w.WriteHeader(http.StatusNotModified)
			return
		}

		for key, values := range rec.Header() {
			for _, value := range values {
				w.Header().Add(key, value)
			}
		}

		w.Header().Set("Cache-Control", cacheControl.Chose(r.URL.RawQuery))

		w.Header().Set("ETag", etag)
		w.WriteHeader(rec.status)

		_, _ = io.Copy(w, rec.body)
	})
}
