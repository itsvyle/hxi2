package globalgoutils

import (
	"log/slog"
	"os"
	"strconv"
	"time"

	"github.com/lmittmann/tint"
)

func InitGlobalSlog() {
	slog.SetDefault(
		GetServiceSpecificLogger("GLOBAL", ""),
	)
}

// Good practice to use a 6 letter long CAPITALIZED name for the service
func GetServiceSpecificLogger(name string, color string) *slog.Logger {
	const ansiFaint = "\033[2m"
	const ansiReset = "\033[0m"
	if os.Getenv("JSON_LOG") == "1" {
		return slog.New(slog.NewJSONHandler(os.Stdout, &slog.HandlerOptions{
			Level: slog.LevelDebug,
		}).WithAttrs([]slog.Attr{slog.String("logger", name)}))
	}
	if color != "" {
		name = color + name + ansiReset
	} else {
		name = ansiFaint + name + ansiReset
	}
	return slog.New(
		tint.NewHandler(os.Stdout, &tint.Options{
			Level:      slog.LevelDebug,
			TimeFormat: time.Kitchen,
			ReplaceAttr: func(groups []string, a slog.Attr) slog.Attr {
				if a.Key == slog.TimeKey {
					a.Value = slog.StringValue(ansiFaint + time.Now().Format("15:04") + ansiReset + " " + name)
				}
				return a
			},
		}),
	)
}

func StringPtr(s string) *string {
	return &s
}

func IntPtr(i int) *int {
	return &i
}

func BoolPtr(b bool) *bool {
	return &b
}

func ParseInt64(s string) (int64, error) {
	id, err := strconv.ParseInt(s, 10, 64)
	if err != nil {
		return 0, err
	}
	return id, nil
}

func Filter[T any](ss []T, test func(T) bool) (ret []T) {
	for _, s := range ss {
		if test(s) {
			ret = append(ret, s)
		}
	}
	return
}
