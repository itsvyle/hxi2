package main

import (
	"database/sql"
	"errors"
	"net/url"
	"strconv"
	"strings"

	"github.com/bwmarrin/discordgo"
	dbw "github.com/itsvyle/hxi2/global-go/discord-bot-wrapper"
	ggu "github.com/itsvyle/hxi2/global-go/utils"
)

type DiscordBot struct {
	*dbw.DiscordBot
	session         *discordgo.Session
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

func (discordBot *DiscordBot) GetMemberClaims(memberID string) (*ggu.HXI2JWTClaims, error) {
	user, err := DB.GetDBUserByDiscordID(memberID)
	if err != nil || user == nil {
		if errors.Is(err, sql.ErrNoRows) {
			return nil, nil
		}
		discordBot.Logger.With("err", err, "discordUserID", memberID).Error("Failed to get user by discord ID")
		return nil, errors.New("failed to get user by discord ID")
	}
	return user.GetNewJWTClaims(), nil
}

func (discordBot *DiscordBot) GetInteractionClaims(interaction *discordgo.InteractionCreate) (*ggu.HXI2JWTClaims, error) {
	if interaction.Member == nil {
		return nil, errors.New("interaction member is nil")
	}
	return discordBot.GetMemberClaims(interaction.Member.User.ID)
}

func (discordBot *DiscordBot) addCommandUpdateUser() {
	const cmdName = "create"
	const newUserPermissions = ggu.RoleStudent
	minp, maxp := ggu.GetPromotionsRange()
	var command = &discordgo.ApplicationCommand{
		Name:        cmdName,
		Description: "Create a student's data",
		Options: []*discordgo.ApplicationCommandOption{
			{
				Type:        discordgo.ApplicationCommandOptionString,
				Name:        "first_name",
				Description: "The first name of the user",
				Required:    true,
			},
			{
				Type:        discordgo.ApplicationCommandOptionInteger,
				Name:        "promo",
				Description: "The promo of the user",
				Required:    true,
				MinValue:    ggu.F64Ptr(float64(minp)),
				MaxValue:    float64(maxp),
			},
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
			{
				Type:        discordgo.ApplicationCommandOptionString,
				Name:        "last_name",
				Description: "The last name of the user",
				Required:    false,
			},
		},
	}

	hand := func(session *discordgo.Session, interaction *discordgo.InteractionCreate) {
		user, err := discordBot.GetInteractionClaims(interaction)
		if err != nil || user == nil {
			discordBot.RespondWithError(interaction, "Failed to get user claims")
			return
		}
		if !user.IsAdmin() {
			discordBot.RespondWithError(interaction, "You do not have permission to use this command")
			return
		}

		data := interaction.ApplicationCommandData()

		optionIndexes := make(map[string]int, len(data.Options))
		for i, option := range data.Options {
			optionIndexes[option.Name] = i
		}

		var discordUsername string = ""

		var userID string = ""
		idx1, ok1 := optionIndexes["user"]
		idx2, ok2 := optionIndexes["userid"]
		if ok1 && ok2 {
			discordBot.RespondWithError(interaction, "You cannot provide both `user` and `userid` options, choose one")
			return
		}
		if ok1 {
			if data.Options[idx1].Type != discordgo.ApplicationCommandOptionUser {
				discordBot.RespondWithError(interaction, "Invalid option type for user, expected user")
				return
			}
			u := data.Options[idx1].UserValue(session)
			userID = u.ID
			discordUsername = u.Username
		} else if ok2 {
			if data.Options[idx2].Type != discordgo.ApplicationCommandOptionString {
				discordBot.RespondWithError(interaction, "Invalid option type for userid, expected string")
				return
			}
			userID = data.Options[idx2].StringValue()
			if _, err := strconv.Atoi(userID); err != nil {
				discordBot.RespondWithError(interaction, "Provided userid is not a valid number - this should be a discord user ID")
				return
			}
		} else {
			discordBot.RespondWithError(interaction, "No `user` or `userid` option provided, at least one should be provided")
		}
		if userID == "" {
			discordBot.RespondWithError(interaction, "No user ID provided")
			return
		}
		var promo int
		if promoIndex, ok := optionIndexes["promo"]; !ok {
			discordBot.RespondWithError(interaction, "Invalid or missing promo option")
			return
		} else {
			promoI64 := data.Options[promoIndex].IntValue()
			promo = int(promoI64)
			if promo < minp || promo > maxp {
				discordBot.RespondWithError(interaction, "Invalid promo value, must be between "+strconv.Itoa(minp)+" and "+strconv.Itoa(maxp))
				return
			}
		}

		var firstName string
		if firstNameIndex, ok := optionIndexes["first_name"]; !ok || data.Options[firstNameIndex].Type != discordgo.ApplicationCommandOptionString {
			discordBot.RespondWithError(interaction, "Invalid or missing first_name option")
			return
		} else {
			firstName = data.Options[firstNameIndex].StringValue()
			if firstName == "" {
				discordBot.RespondWithError(interaction, "First name cannot be empty")
				return
			}
		}

		if discordUsername == "" {
			discordUsername = strings.ToLower(firstName)
		}

		var lastName string = ""
		if lastNameIndex, ok := optionIndexes["last_name"]; ok && data.Options[lastNameIndex].Type == discordgo.ApplicationCommandOptionString {
			lastName = data.Options[lastNameIndex].StringValue()
			if lastName == "" {
				discordBot.RespondWithError(interaction, "Last name cannot be empty if provided")
				return
			}
		}

		discordBot.Logger.Debug("Creating user", "discordUserID", userID, "firstName", firstName, "lastName", lastName, "promo", promo, "discordUsername", discordUsername)

		// Now, create the user in the database
		_, err = DB.GetDBUserByDiscordID(userID)
		if err != nil && !errors.Is(err, sql.ErrNoRows) {
			discordBot.Logger.With("err", err, "discordUserID", userID).Error("Failed to get user by discord ID")
			discordBot.RespondWithError(interaction, "Failed to get user by discord ID")
			return
		}

		var ln sql.NullString
		if lastName != "" {
			ln = sql.NullString{String: lastName, Valid: true}
		} else {
			ln = sql.NullString{Valid: false}
		}

		newID, err := ggu.Generate32BitsNumber()
		if err != nil {
			discordBot.Logger.With("err", err, "discordUserID", userID).Error("Failed to generate new user ID")
			discordBot.RespondWithError(interaction, "Failed to generate new user ID")
			return
		}

		newUser := &DBUser{
			ID:          newID,
			DiscordID:   userID,
			Username:    discordUsername,
			FirstName:   firstName,
			LastName:    ln,
			Promotion:   promo,
			Permissions: newUserPermissions,
		}

		err = DB.CreateNewUser(newUser)
		if err != nil {
			discordBot.Logger.With("err", err, "discordUserID", userID).Error("Failed to create new user")
			discordBot.RespondWithError(interaction, "Failed to create new user")
			return
		}
		embed := &discordgo.MessageEmbed{
			Title:       "User created successfully",
			Description: "The user has been created and can now log in to hxi2.fr.",
			Color:       0x00FF00,
			Fields: []*discordgo.MessageEmbedField{
				{
					Name:   "Discord ID",
					Value:  newUser.DiscordID,
					Inline: true,
				},
				{
					Name:   "Username",
					Value:  newUser.Username,
					Inline: true,
				},
				{
					Name:   "First Name",
					Value:  newUser.FirstName,
					Inline: true,
				},
				{
					Name: "Last Name",
					Value: func() string {
						if newUser.LastName.Valid {
							return newUser.LastName.String
						}
						return "(not set)"
					}(),
					Inline: true,
				},
				{
					Name:   "Promotion",
					Value:  strconv.Itoa(newUser.Promotion),
					Inline: true,
				},
				{
					Name:   "Permissions",
					Value:  strconv.Itoa(newUser.Permissions),
					Inline: true,
				},
				{
					Name:   "Internal ID",
					Value:  strconv.Itoa(int(newUser.ID)),
					Inline: true,
				},
			},
		}

		err = session.InteractionRespond(interaction.Interaction, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseChannelMessageWithSource,
			Data: &discordgo.InteractionResponseData{
				Flags:  discordgo.MessageFlagsEphemeral,
				Embeds: []*discordgo.MessageEmbed{embed},
			},
		})
		if err != nil {
			discordBot.Logger.With("err", err, "discordUserID", userID).Error("Failed to respond to interaction")
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
			discordBot.Logger.With("err", err, "discordUserID", discordUserID).Error("Failed to get user by discord ID")
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
			discordBot.Logger.With("err", err, "discordUserID", discordUserID).Error("Failed to respond to interaction")
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
			discordBot.Logger.With("err", err, "discordUserID", discordUserID).Error("Failed to get user by discord ID")
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
			discordBot.Logger.With("err", err, "discordUserID", discordUserID).Error("Failed to respond to interaction")
			return
		}
	}

	discordBot.AddCommand(cmdName, command, hand)
}
