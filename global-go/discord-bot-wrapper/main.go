package discordbotwrapper

import (
	"log/slog"

	"github.com/bwmarrin/discordgo"
	ggu "github.com/itsvyle/hxi2/global-go/utils"
)

type DiscordBotCommandHandler func(session *discordgo.Session, interaction *discordgo.InteractionCreate)

type DiscordBot struct {
	session         *discordgo.Session
	Logger          *slog.Logger
	commandHandlers map[string]DiscordBotCommandHandler
	commands        []*discordgo.ApplicationCommand
}

// Returns a new DiscordBot instance with the provided token.
// Once created, you can register commands using AddCommand.
// Use .Start() to start the bot.
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
		Logger:          l,
		commandHandlers: map[string]DiscordBotCommandHandler{},
	}

	discordSession.AddHandler(d.onReady)

	return d, nil
}

func (discordBot *DiscordBot) Start() error {
	err := discordBot.session.Open()
	if err != nil {
		discordBot.Logger.With("err", err).Error("Error logging in to the discord session. Check that token is valid.")
		return err
	}
	discordBot.Logger.Debug("Discord bot started")
	discordBot.Logger.Debug("Registering commands")
	err = discordBot.RegisterCommands(discordBot.session)
	if err != nil {
		discordBot.Logger.With("err", err).Error("Failed to register commands")
		return err
	}
	discordBot.Logger.Debug("Commands registered")
	discordBot.Logger.Debug("Waiting for events")
	return nil
}

func (discordBot *DiscordBot) onReady(session *discordgo.Session, _ *discordgo.Ready) {
	discordBot.Logger.With("username", session.State.User.Username).Info("Bot is ready")
	session.AddHandler(discordBot.onCommandInteraction)
	session.AddHandler(discordBot.onMessageButtonPressed)
	session.AddHandler(discordBot.onModalSubmit)
}

func (discordBot *DiscordBot) RegisterCommands(session *discordgo.Session) (err error) {

	_, err = session.ApplicationCommandBulkOverwrite(session.State.User.ID, "", discordBot.commands)
	if err != nil {
		discordBot.Logger.With("err", err).Error("Failed to register commands")
		return err
	}

	return
}

func (discordBot *DiscordBot) AddCommand(commandName string, command *discordgo.ApplicationCommand, handler DiscordBotCommandHandler) {
	discordBot.commands = append(discordBot.commands, command)
	discordBot.commandHandlers[commandName] = handler
	discordBot.Logger.With("command", commandName).Debug("Command added")
}

func (discordBot *DiscordBot) onCommandInteraction(session *discordgo.Session, interaction *discordgo.InteractionCreate) {
	if interaction.Type != discordgo.InteractionApplicationCommand {
		return
	}
	interractionID := interaction.ApplicationCommandData().Name
	if handler, ok := discordBot.commandHandlers[interractionID]; ok {
		handler(session, interaction)
	} else {
		err := discordBot.session.InteractionRespond(interaction.Interaction, discordBot.ErrorInterractionResponse("Command not found"))
		if err != nil {
			discordBot.Logger.With("err", err, "interractionID", interractionID).Error("Received invalid interraction id")
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

func (discordBot *DiscordBot) ErrorInterractionResponse(text string) *discordgo.InteractionResponse {
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

func (discordBot *DiscordBot) RespondWithError(interaction *discordgo.InteractionCreate, text string) {
	err := discordBot.session.InteractionRespond(interaction.Interaction, discordBot.ErrorInterractionResponse(text))
	if err != nil {
		discordBot.Logger.With("err", err).Error("Failed to respond to interaction with error")
	}
}
