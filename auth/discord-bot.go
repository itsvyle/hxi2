package main

import (
	"log/slog"

	"github.com/bwmarrin/discordgo"
	ggu "github.com/itsvyle/hxi2/global-go/utils"
)

type DiscordBot struct {
	session         *discordgo.Session
	logger          *slog.Logger
	commandHandlers map[string]func(session *discordgo.Session, interaction *discordgo.InteractionCreate)
}

func NewDiscordBot(token string) (*DiscordBot, error) {
	l := ggu.GetServiceSpecificLogger("DISBOT", "\033[38;2;88;101;242m")

	discordSession, err := discordgo.New("Bot " + token)
	if err != nil {
		l.With("err", err).Error("Error logging in to the discord session because of invalid bot parameters. Check that token is valid.")
		return nil, err
	}
	discordSession.ShouldReconnectOnError = true
	discordSession.Identify.Intents = discordgo.MakeIntent(discordgo.PermissionCreateInstantInvite)

	d := &DiscordBot{
		session:         discordSession,
		logger:          l,
		commandHandlers: map[string]func(session *discordgo.Session, interaction *discordgo.InteractionCreate){},
	}

	discordSession.AddHandler(d.onReady)

	return d, nil
}

func (discordBot *DiscordBot) Start() error {
	err := discordBot.session.Open()
	if err != nil {
		discordBot.logger.With("err", err).Error("Error logging in to the discord session. Check that token is valid.")
		return err
	}
	discordBot.logger.Debug("Discord bot started")
	discordBot.logger.Debug("Registering commands")
	err = discordBot.RegisterCommands(discordBot.session)
	if err != nil {
		discordBot.logger.With("err", err).Error("Failed to register commands")
		return err
	}
	discordBot.logger.Debug("Commands registered")
	discordBot.logger.Debug("Waiting for events")
	return nil
}

func (discordBot *DiscordBot) onReady(session *discordgo.Session, _ *discordgo.Ready) {
	discordBot.logger.With("username", session.State.User.Username).Info("Bot is ready")
	session.AddHandler(discordBot.onCommandInteraction)
	session.AddHandler(discordBot.onMessageButtonPressed)
	session.AddHandler(discordBot.onModalSubmit)
}

func (discordBot *DiscordBot) RegisterCommands(session *discordgo.Session) (err error) {
	commandsPayload := []*discordgo.ApplicationCommand{
		// discordBot.commandUpdateUser(),
		discordBot.commandLogin(),
	}

	_, err = session.ApplicationCommandBulkOverwrite(session.State.User.ID, "", commandsPayload)
	if err != nil {
		discordBot.logger.With("err", err).Error("Failed to register commands")
		return err
	}

	return
}

func (discordBot *DiscordBot) onCommandInteraction(session *discordgo.Session, interaction *discordgo.InteractionCreate) {
	if interaction.Type != discordgo.InteractionApplicationCommand {
		return
	}
	interractionID := interaction.ApplicationCommandData().Name
	if handler, ok := discordBot.commandHandlers[interractionID]; ok {
		handler(session, interaction)
	} else {
		err := discordBot.session.InteractionRespond(interaction.Interaction, discordBot.errorInterractionResponse("Command not found"))
		if err != nil {
			discordBot.logger.With("err", err, "interractionID", interractionID).Error("Received invalid interraction id")
		}
	}
}

func (discordBot *DiscordBot) onMessageButtonPressed(session *discordgo.Session, interaction *discordgo.InteractionCreate) {
	if interaction.Type != discordgo.InteractionMessageComponent {
		return
	}
}

func (discordBot *DiscordBot) onModalSubmit(session *discordgo.Session, interaction *discordgo.InteractionCreate) {
	if interaction.Type != discordgo.InteractionModalSubmit {
		return
	}
}

func (discordBot *DiscordBot) errorInterractionResponse(text string) *discordgo.InteractionResponse {
	return &discordgo.InteractionResponse{
		Type: discordgo.InteractionResponseChannelMessageWithSource,
		Data: &discordgo.InteractionResponseData{
			Flags: discordgo.MessageFlagsEphemeral,
			Embeds: []*discordgo.MessageEmbed{
				{
					Title:       "Error",
					Description: text,
					Color:       0xFF0000,
				},
			},
		},
	}
}

func (discordBot *DiscordBot) respondWithError(interaction *discordgo.InteractionCreate, text string) {
	err := discordBot.session.InteractionRespond(interaction.Interaction, discordBot.errorInterractionResponse(text))
	if err != nil {
		discordBot.logger.With("err", err).Error("Failed to respond to interaction with error")
	}
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
		err := discordBot.session.InteractionRespond(interaction.Interaction, discordBot.errorInterractionResponse("You are not an admin"))
		if err != nil {
			discordBot.logger.With("err", err).Error("Failed to respond to interaction")
		}
	}
	return authenticated
}

func (discordBot *DiscordBot) commandUpdateUser() *discordgo.ApplicationCommand {
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

	discordBot.commandHandlers[cmdName] = func(session *discordgo.Session, interaction *discordgo.InteractionCreate) {
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
			discordBot.respondWithError(interaction, "Both user and userid options were provided, using user option; only one should be provided")
			return
		}
		if userID == "" {
			discordBot.respondWithError(interaction, "No user or userid option provided, at least one should be provided")
			return
		}
	}

	return command
}

func (discordBot *DiscordBot) commandLogin() *discordgo.ApplicationCommand {
	const cmdName = "login"
	var command = &discordgo.ApplicationCommand{
		Name:        cmdName,
		Description: "Login to hxi2.fr",
	}

	discordBot.commandHandlers[cmdName] = func(session *discordgo.Session, interaction *discordgo.InteractionCreate) {
		if interaction.Member == nil {
			return
		}
		discordUserID := interaction.Member.User.ID
		user, err := DB.GetDBUserByDiscordID(discordUserID)
		if err != nil || user == nil {
			discordBot.logger.With("err", err, "discordUserID", discordUserID).Error("Failed to get user by discord ID")
			discordBot.respondWithError(interaction, "Failed to get hxi2.fr user - maybe you are not registered on hxi2.fr?")
			return
		}
		code, err := DB.CreateOneTimeCode(user.ID)
		if err != nil {
			discordBot.respondWithError(interaction, "Failed to create one-time code")
			return
		}
		codeURL := authManager.LoginPageURL + "?code=" + code

		err = discordBot.session.InteractionRespond(interaction.Interaction, &discordgo.InteractionResponse{
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

	return command
}
