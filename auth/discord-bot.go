package main

import (
	"log/slog"
	"net/url"

	"github.com/bwmarrin/discordgo"
	dbw "github.com/itsvyle/hxi2/global-go/discord-bot-wrapper"
)

type DiscordBot struct {
	*dbw.DiscordBot
	session         *discordgo.Session
	logger          *slog.Logger
	commandHandlers map[string]func(session *discordgo.Session, interaction *discordgo.InteractionCreate)
}

func NewDiscordBot(token string) (*DiscordBot, error) {
	n, err := dbw.NewDiscordBot(token)
	if err != nil {
		return nil, err
	}
	b := &DiscordBot{
		DiscordBot: n,
	}
	b.addCommandLogin()
	b.addCommandUpdateUser()
	b.addCommandParrainsup()
	return b, nil
}

func (discordBot *DiscordBot) checkAdmin(interaction *discordgo.InteractionCreate) bool {
	if interaction.Member == nil {
		return false
	}
	panic("checkAdmin is TODO")

	// userID := interaction.Member.User.ID

	// auth logic, for now ill just put authenticated := false

	authenticated := true

	if !authenticated {
		// answer with ephemeral message, you are not admin
		err := discordBot.session.InteractionRespond(interaction.Interaction, discordBot.ErrorInterractionResponse("You are not an admin"))
		if err != nil {
			discordBot.logger.With("err", err).Error("Failed to respond to interaction")
		}
	}
	return authenticated
}

func (discordBot *DiscordBot) addCommandUpdateUser() {
	const cmdName = "update"

	var command = &discordgo.ApplicationCommand{
		Name:        cmdName,
		Description: "Update a user's data",
		Options: []*discordgo.ApplicationCommandOption{
			{
				Type:        discordgo.ApplicationCommandOptionUser,
				Name:        "user",
				Description: "The Discord user to update",
				Required:    false,
			},
			{
				Type:        discordgo.ApplicationCommandOptionString,
				Name:        "userid",
				Description: "The ID of the user to update, if the user isn't in the server",
				Required:    false,
			},
		},
	}

	hand := func(session *discordgo.Session, interaction *discordgo.InteractionCreate) {
		if !discordBot.checkAdmin(interaction) {
			return
		}

		data := interaction.ApplicationCommandData()

		optionIndexes := make(map[string]int, len(data.Options))
		for i, option := range data.Options {
			optionIndexes[option.Name] = i
		}

		userID := data.Options[0].UserValue(session).ID
		if userID == "" {
			userID = data.Options[1].StringValue()
		} else if data.Options[1].StringValue() != "" {
			discordBot.RespondWithError(interaction, "Both user and userid options were provided, using user option; only one should be provided")
			return
		}
		if userID == "" {
			discordBot.RespondWithError(interaction, "No user or userid option provided, at least one should be provided")
			return
		}
	}

	discordBot.AddCommand(cmdName, command, hand)
}

func (discordBot *DiscordBot) addCommandLogin() {
	const cmdName = "login"
	var command = &discordgo.ApplicationCommand{
		Name:        cmdName,
		Description: "Login to hxi2.fr",
	}

	hand := func(session *discordgo.Session, interaction *discordgo.InteractionCreate) {
		if interaction.Member == nil {
			return
		}
		discordUserID := interaction.Member.User.ID
		user, err := DB.GetDBUserByDiscordID(discordUserID)
		if err != nil || user == nil {
			discordBot.logger.With("err", err, "discordUserID", discordUserID).Error("Failed to get user by discord ID")
			discordBot.RespondWithError(interaction, "Failed to get hxi2.fr user - maybe you are not registered on hxi2.fr?")
			return
		}
		code, err := DB.CreateOneTimeCode(user.ID)
		if err != nil {
			discordBot.RespondWithError(interaction, "Failed to create one-time code")
			return
		}
		codeURL := authManager.LoginPageURL + "?code=" + code

		err = session.InteractionRespond(interaction.Interaction, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseChannelMessageWithSource,
			Data: &discordgo.InteractionResponseData{
				Flags: discordgo.MessageFlagsEphemeral,
				Embeds: []*discordgo.MessageEmbed{
					{
						Title:       "Your one-time code",
						Description: "Use this code to login to [hxi2.fr](" + authManager.LoginPageURL + "), or click the button below: \n\n" + code + "\n\nThis code is valid for 10 minutes.",
						Color:       0x00FF00,
					},
				},
				Components: []discordgo.MessageComponent{
					discordgo.ActionsRow{
						Components: []discordgo.MessageComponent{
							discordgo.Button{
								Label:    "Login to hxi2.fr",
								Style:    discordgo.LinkButton,
								URL:      codeURL,
								Disabled: false,
							},
						},
					},
				},
			},
		})
		if err != nil {
			discordBot.logger.With("err", err, "discordUserID", discordUserID).Error("Failed to respond to interaction")
			return
		}
	}

	discordBot.AddCommand(cmdName, command, hand)
}

func (discordBot *DiscordBot) addCommandParrainsup() {
	const cmdName = "parrainsup"
	var command = &discordgo.ApplicationCommand{
		Name:        cmdName,
		Description: "Ouvre parrainsup",
	}

	hand := func(session *discordgo.Session, interaction *discordgo.InteractionCreate) {
		if interaction.Member == nil {
			return
		}
		discordUserID := interaction.Member.User.ID
		user, err := DB.GetDBUserByDiscordID(discordUserID)
		if err != nil || user == nil {
			discordBot.logger.With("err", err, "discordUserID", discordUserID).Error("Failed to get user by discord ID")
			discordBot.RespondWithError(interaction, "Failed to get hxi2.fr user - maybe you are not registered on hxi2.fr?")
			return
		}
		code, err := DB.CreateOneTimeCode(user.ID)
		if err != nil {
			discordBot.RespondWithError(interaction, "Failed to create one-time code")
			return
		}
		codeURL := authManager.LoginPageURL + "?code=" + code + "&redirectTo=" + url.QueryEscape("https://parrainsup.hxi2.fr/")

		err = session.InteractionRespond(interaction.Interaction, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseChannelMessageWithSource,
			Data: &discordgo.InteractionResponseData{
				Flags:   discordgo.MessageFlagsEphemeral,
				Content: "Cliquer sur le bouton ci-dessous pour ouvrir Parrainsup\n**Attention: Ce lien vous est strictement personnel, car il vous identifie sur hxi2.fr.**",
				Components: []discordgo.MessageComponent{
					discordgo.ActionsRow{
						Components: []discordgo.MessageComponent{
							discordgo.Button{
								Label:    "Ouvrir Parrainsup",
								Style:    discordgo.LinkButton,
								URL:      codeURL,
								Disabled: false,
							},
						},
					},
				},
			},
		})
		if err != nil {
			discordBot.logger.With("err", err, "discordUserID", discordUserID).Error("Failed to respond to interaction")
			return
		}
	}

	discordBot.AddCommand(cmdName, command, hand)
}
