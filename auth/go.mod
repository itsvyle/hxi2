module github.com/itsvyle/hxi2/auth

go 1.23.2

require (
	github.com/cristalhq/jwt/v5 v5.4.0
	github.com/itsvyle/hxi2/global-go-utils v0.0.0
	github.com/jmoiron/sqlx v1.4.0
	github.com/lmittmann/tint v1.0.6
	github.com/mattn/go-sqlite3 v1.14.24
	golang.org/x/oauth2 v0.24.0
)

require (
	github.com/bwmarrin/discordgo v0.29.0 // indirect
	github.com/gorilla/websocket v1.5.3 // indirect
	golang.org/x/crypto v0.38.0 // indirect
	golang.org/x/net v0.39.0 // indirect
	golang.org/x/sys v0.33.0 // indirect
)

replace github.com/itsvyle/hxi2/global-go-utils => ../global-go-utils
