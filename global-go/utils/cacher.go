package globalgoutils

import (
	"log/slog"
	"sync"
	"time"
)

// should be sync safe
type Cacher[T any] struct {
	CachedValue *T
	getter      func() (T, error)
	fetchMutex  sync.Mutex
	cacherName  string
	RefreshRate int64
	LastUpdated int64
	// The minimum time to wait before allowing cache refresh requests
	// Seconds, 0 to disable
	MinimumWait         int64
	lastRefreshAsked    int64
	isWaitingForRefresh bool
	logger              *slog.Logger
}

func NewCacher[T any](cacheName string, getter func() (T, error), refreshRate time.Duration, minimumWait int64) *Cacher[T] {
	// get the name of the T type and put it in cacherName
	c := &Cacher[T]{
		getter:              getter,
		RefreshRate:         int64(refreshRate.Seconds()),
		fetchMutex:          sync.Mutex{},
		cacherName:          cacheName,
		MinimumWait:         minimumWait,
		LastUpdated:         1,
		lastRefreshAsked:    0,
		isWaitingForRefresh: false,
		logger:              slog.With("cacheName", cacheName),
	}

	return c
}

// WARNING: This function isn't thread safe, and it bypasses the refresh rate limit
func (c *Cacher[T]) refreshCache() error {
	v, err := c.getter()
	if err != nil {
		return err
	}
	c.CachedValue = &v
	c.LastUpdated = time.Now().Unix()
	c.logger.Debug("[cacher] Updated value")
	return nil
}

func (c *Cacher[T]) Get() (*T, error) {
	if c.CachedValue == nil || time.Now().Unix()-c.LastUpdated > c.RefreshRate {
		c.fetchMutex.Lock()
		defer c.fetchMutex.Unlock()
		// Check again in case another goroutine updated the value
		if c.CachedValue == nil || time.Now().Unix()-c.LastUpdated > c.RefreshRate {
			err := c.refreshCache()
			if err != nil {
				c.logger.With("error", err).Debug("[cacher] Failed to get value")
				return nil, err
			}
		}
	}

	return c.CachedValue, nil
}

// GetNow returns the cached value without checking if it's outdated
// If it's outdated, it will start a fetch in the background
// If it's nil, it will start a fetch in the foreground
func (c *Cacher[T]) GetNow() (*T, error) {
	if c.CachedValue == nil {
		return c.Get()
	}
	if time.Now().Unix()-c.LastUpdated > c.RefreshRate {
		go func() {
			_, _ = c.Get()
		}()
	}
	return c.CachedValue, nil
}

func (c *Cacher[T]) ForceInvalidate() {
	c.LastUpdated = 0
}

func (c *Cacher[T]) AskCacheRefresh() {
	if c.MinimumWait <= 0 {
		c.fetchMutex.Lock()
		defer c.fetchMutex.Unlock()
		_ = c.refreshCache()
		return
	}
	if c.isWaitingForRefresh {
		c.lastRefreshAsked = time.Now().Unix()
		return
	}
	// im on cooldown, but not yet waiting for refresh, so start a request ASAP
	if time.Now().Unix()-c.lastRefreshAsked < c.MinimumWait {
		c.lastRefreshAsked = time.Now().Unix()
		c.isWaitingForRefresh = true
		go func() {
			time.Sleep(time.Duration(c.MinimumWait) * time.Second)
			c.fetchMutex.Lock()
			defer c.fetchMutex.Unlock()
			if !c.isWaitingForRefresh {
				return
			}
			err := c.refreshCache()
			if err != nil {
				c.logger.With("error", err).Debug("[cacher] Failed to get value on refresh request")
			}
			c.isWaitingForRefresh = false
		}()
		return
	}
	c.lastRefreshAsked = time.Now().Unix()
	go func() {
		c.fetchMutex.Lock()
		defer c.fetchMutex.Unlock()
		err := c.refreshCache()
		if err != nil {
			c.logger.With("error", err).Debug("[cacher] Failed to get value on refresh request")
		}
	}()
}
