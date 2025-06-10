package main

// Code taken from: https://github.com/abhinav/goldmark-mermaid/tree/main/mermaidcdp
// Refactored to make a general chrome compiler
import (
	"context"
	_ "embed" // for go:embed
	"encoding/json"
	"fmt"
	"runtime"
	"strings"
	"sync"

	cdruntime "github.com/chromedp/cdproto/runtime"
	"github.com/chromedp/chromedp"
)

// ChromeCompilerConfig specifies the configuration for a Compiler.
type ChromeCompilerConfig[_ any, Output any] struct {
	// JSSource is JavaScript code that will be injected into the headless browser as a base, before init.
	JSSource string
	// JSExtra is additional JavaScript code that will be injected into the headless browser.
	JSExtra string
	// JSInit is JavaScript code that will be executed when the headless browser is started.
	JSInit string

	// OutputProcessor is a function that processes the output of the compiler. It's optional.
	OutputProcessor func(*string) (*Output, error)

	// NoSandbox disables the sandbox for the headless browser.
	//
	// Use this with care.
	NoSandbox bool
}

type ChromeCompiler[Input any, Output any] struct {
	mu              sync.RWMutex // guards ctx
	OutputProcessor func(*string) (*Output, error)

	// While standard practice is to not hold a context in a struct,
	// we do so here because that's where chromedp puts information
	// about the headless browser it's using.
	//
	// ctx is the context scoped to the headless browser.
	ctx context.Context
}

// CreateChromeCompiler builds a new Compiler with the provided configuration.
//
// The returned Compiler must be closed with [Close] when it is no longer needed.
func CreateChromeCompiler[Input any, Output any](cfg *ChromeCompilerConfig[Input, Output]) (_ *ChromeCompiler[Input, Output], err error) {
	if cfg.JSSource == "" {
		return nil, fmt.Errorf("source code wasn't provided; please provide a non-empty JSSource")
	}

	var ctxOpts []chromedp.ContextOption

	ctx := context.Background()
	if cfg.NoSandbox {
		execOpts := make([]chromedp.ExecAllocatorOption, 0, len(chromedp.DefaultExecAllocatorOptions)+1)
		execOpts = append(execOpts, chromedp.DefaultExecAllocatorOptions[:]...)
		execOpts = append(execOpts, chromedp.NoSandbox)

		var cancel context.CancelFunc
		ctx, cancel = chromedp.NewExecAllocator(ctx, execOpts...)
		defer func(cancel context.CancelFunc) {
			if err != nil {
				cancel() // kill it if this function fails
			}
		}(cancel)
	}

	// The cdp context should NOT be bound to a context with a limited lifetime
	// because that'll kill the headless browser when the context finishes.
	// Instead, we'll use the background context.
	ctx, cancel := chromedp.NewContext(ctx, ctxOpts...)
	defer func(cancel context.CancelFunc) {
		if err != nil {
			cancel() // kill it if this function fails
		}
	}(cancel)

	var ready *cdruntime.RemoteObject
	if err := chromedp.Run(ctx, chromedp.Evaluate(cfg.JSSource, &ready)); err != nil {
		return nil, fmt.Errorf("set up headless browser: %w", err)
	}

	ready = nil
	if cfg.JSExtra != "" {
		if err := chromedp.Run(ctx, chromedp.Evaluate(cfg.JSExtra, &ready)); err != nil {
			return nil, fmt.Errorf("inject additional JavaScript: %w", err)
		}
	}

	ready = nil
	if err := chromedp.Run(ctx, chromedp.Evaluate(cfg.JSInit, &ready)); err != nil {
		return nil, fmt.Errorf("initialize: %w", err)
	}

	c := &ChromeCompiler[Input, Output]{ctx: ctx, OutputProcessor: cfg.OutputProcessor}
	runtime.SetFinalizer(c, func(c *ChromeCompiler[Input, Output]) {
		// If the engine is garbage collected and not closed, close it.
		_ = c.Close()
	})
	return c, nil
}

// CompileToString executes function `functionToCall` with `req` as the argument, json-encoded, and returns the result as a string.
// The context controls how long the rendering is allowed to take.
//
// Panics if the Compiler has already been closed.
func (c *ChromeCompiler[Input, Output]) CompileToString(ctx context.Context, functionToCall string, req *Input) (*string, error) {
	var script strings.Builder
	script.WriteString(functionToCall)
	script.WriteString("(")
	if err := json.NewEncoder(&script).Encode(req); err != nil {
		return nil, fmt.Errorf("encode source: %w", err)
	}
	script.WriteString(")")

	// TODO: Can we use chromedp.CallFunctionOn instead?
	var result string
	render := chromedp.Evaluate(
		script.String(),
		&result,
		func(p *cdruntime.EvaluateParams) *cdruntime.EvaluateParams {
			return p.WithAwaitPromise(true)
		},
	)

	c.mu.RLock()
	defer c.mu.RUnlock()
	if c.ctx == nil {
		panic("Compiler is closed")
	}
	ctx, cancel := mergeCtxLifetime(c.ctx, ctx)
	defer cancel()

	err := chromedp.Run(ctx, render)
	return &result, err
}

func (c *ChromeCompiler[Input, Output]) Compile(ctx context.Context, functionToCall string, req *Input) (*Output, error) {
	if c.OutputProcessor == nil {
		panic("OutputProcessor must be set")
	}
	result, err := c.CompileToString(ctx, functionToCall, req)
	if err != nil {
		return nil, err
	}

	return c.OutputProcessor(result)
}

// Close stops the compiler and releases any resources it holds.
// This method must be called when the compiler is no longer needed.
func (c *ChromeCompiler[Input, Output]) Close() error {
	c.mu.Lock()
	defer c.mu.Unlock()

	if ctx := c.ctx; ctx != nil {
		c.ctx = nil
		return chromedp.Cancel(ctx)
	}

	return nil
}

func mergeCtxLifetimeInner(parentCtx, timeCtx context.Context) (context.Context, context.CancelFunc) {
	ctx, cancel := context.WithCancelCause(parentCtx)
	stop := context.AfterFunc(timeCtx, func() {
		cancel(context.Cause(timeCtx))
	})
	return ctx, func() {
		stop()
		cancel(context.Canceled)
	}
}

var _backgroundCtx = context.Background()

func mergeCtxLifetime(parentCtx, timeCtx context.Context) (context.Context, context.CancelFunc) {
	// Optimization: Avoid the goroutine if either
	// is a background context.
	if parentCtx == _backgroundCtx {
		return context.WithCancel(timeCtx)
	} else if timeCtx == _backgroundCtx {
		return context.WithCancel(parentCtx)
	}

	return mergeCtxLifetimeInner(parentCtx, timeCtx)
}
