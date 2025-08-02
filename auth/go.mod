module github.com/itsvyle/hxi2/auth

go 1.24.5

require (
	github.com/bwmarrin/discordgo v0.29.0
	github.com/cristalhq/jwt/v5 v5.4.0
	github.com/itsvyle/hxi2/global-go/discord-bot-wrapper v0.0.0-00010101000000-000000000000
	github.com/itsvyle/hxi2/global-go/utils v0.0.0
	github.com/jmoiron/sqlx v1.4.0
	github.com/mattn/go-sqlite3 v1.14.24
	golang.org/x/net v0.41.0
	golang.org/x/oauth2 v0.27.0
)

require (
	github.com/gorilla/websocket v1.5.3 // indirect
	github.com/lmittmann/tint v1.0.6 // indirect
	golang.org/x/crypto v0.40.0 // indirect
	golang.org/x/sys v0.34.0 // indirect
)

replace github.com/itsvyle/hxi2/global-go/utils => ../global-go/utils

replace github.com/itsvyle/hxi2/global-go/discord-bot-wrapper => ../global-go/discord-bot-wrapper
