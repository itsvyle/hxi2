module github.com/itsvyle/hxi2/global-go/discord-bot-wrapper

go 1.24.5

replace github.com/itsvyle/hxi2/global-go/utils => ../utils

require (
	github.com/bwmarrin/discordgo v0.29.0
	github.com/itsvyle/hxi2/global-go/utils v0.0.0-20250802091751-5994c90da70c
)

require (
	github.com/cristalhq/jwt/v5 v5.4.0 // indirect
	github.com/gorilla/websocket v1.5.3 // indirect
	github.com/lmittmann/tint v1.0.6 // indirect
	golang.org/x/crypto v0.40.0 // indirect
	golang.org/x/sys v0.34.0 // indirect
)
