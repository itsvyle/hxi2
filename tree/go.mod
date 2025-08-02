module github.com/itsvyle/hxi2/tree

go 1.23.2

replace github.com/itsvyle/hxi2/global-go/utils => ../global-go/utils

require (
	github.com/chromedp/cdproto v0.0.0-20230220211738-2b1ec77315c9
	github.com/chromedp/chromedp v0.9.1
	github.com/itsvyle/hxi2/global-go/utils v0.0.0-00010101000000-000000000000
	github.com/jmoiron/sqlx v1.4.0
	github.com/mattn/go-sqlite3 v1.14.24
)

require (
	github.com/chromedp/sysutil v1.0.0 // indirect
	github.com/cristalhq/jwt/v5 v5.4.0 // indirect
	github.com/gobwas/httphead v0.1.0 // indirect
	github.com/gobwas/pool v0.2.1 // indirect
	github.com/gobwas/ws v1.1.0 // indirect
	github.com/josharian/intern v1.0.0 // indirect
	github.com/lmittmann/tint v1.0.6 // indirect
	github.com/mailru/easyjson v0.7.7 // indirect
	golang.org/x/sys v0.6.0 // indirect
)
