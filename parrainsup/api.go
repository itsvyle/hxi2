package main

import (
	"database/sql"
	"encoding/json"
	"errors"
	"log/slog"
	"net/http"
	"strings"

	ggu "github.com/itsvyle/hxi2/global-go/utils"
)

func HandleListUsers(w http.ResponseWriter, r *http.Request) {
	c, err := authManager.AuthenticateHTTPRequest(w, r, true)
	if err != nil || !c.CheckPermHTTP(w, ggu.RoleStudent) {
		return
	}

	users, err := mainUsersCacher.Get()
	if err != nil {
		slog.With("error", err).Error("Failed to get users")
		http.Error(w, "Failed to get users", http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	err = json.NewEncoder(w).Encode(*users)
	if err != nil {
		slog.With("error", err).Error("Failed to encode users")
		http.Error(w, "Failed to encode users", http.StatusInternalServerError)
		return
	}
}

func HandleGetUserMyself(w http.ResponseWriter, r *http.Request) {
	c, err := authManager.AuthenticateHTTPRequest(w, r, true)
	if err != nil || !c.CheckPermHTTP(w, ggu.RoleStudent) {
		return
	}
	if c.Promotion != promoActive {
		http.Error(w, "Promotion not active", http.StatusForbidden)
		return
	}

	user, err := DB.GetMainUserByID(c.IDInt())
	if err != nil {
		if errors.Is(err, sql.ErrNoRows) {
			user = &MainUser{
				UserID:          c.IDInt(),
				DiscordUsername: c.Username,
			}

		} else {
			slog.With("error", err).Error("Failed to get user by ID")
			http.Error(w, "Failed to get user", http.StatusInternalServerError)
			return
		}
	}

	w.Header().Set("Content-Type", "application/json")
	err = json.NewEncoder(w).Encode(user)
	if err != nil {
		slog.With("error", err).Error("Failed to encode user")
		http.Error(w, "Failed to encode user", http.StatusInternalServerError)
		return
	}
}

func HandleUpdateUserMyself(w http.ResponseWriter, r *http.Request) {
	c, err := authManager.AuthenticateHTTPRequest(w, r, true)
	if err != nil || !c.CheckPermHTTP(w, ggu.RoleStudent) {
		return
	}
	if c.Promotion != promoActive {
		http.Error(w, "Promotion not active", http.StatusForbidden)
		return
	}

	var newUserRaw MainUser
	err = json.NewDecoder(r.Body).Decode(&newUserRaw)
	if err != nil {
		slog.With("error", err).Error("Failed to decode user data")
		http.Error(w, "Invalid user data", http.StatusBadRequest)
		return
	}
	if newUserRaw.Couleur != "" {
		newUserRaw.Couleur = strings.ToLower(newUserRaw.Couleur)
	}
	newUserRaw.DisplayName = strings.TrimSpace(newUserRaw.DisplayName)
	if newUserRaw.DisplayName == "" {
		http.Error(w, "Display name cannot be empty", http.StatusBadRequest)
		return
	}

	if !c.IsAdmin() || newUserRaw.UserID != 0 {
		newUserRaw.UserID = c.IDInt()
		newUserRaw.DiscordUsername = c.Username
	}

	oldUser, err := DB.GetMainUserByID(newUserRaw.UserID)
	if err != nil {
		if errors.Is(err, sql.ErrNoRows) {
			oldUser = &MainUser{
				UserID:           newUserRaw.UserID,
				DiscordUsername:  newUserRaw.DiscordUsername,
				EditRestrictions: 0,
			}
			err = DB.InsertNewMainUser(oldUser)
			if err != nil {
				slog.With("error", err).Error("Failed to insert new user")
				http.Error(w, "Failed to insert new user", http.StatusInternalServerError)
				return
			}
		} else {
			slog.With("error", err).Error("Failed to get user by ID")
			http.Error(w, "Failed to get user", http.StatusInternalServerError)
			return
		}
	}

	newUser, mergeErr := MergeUserWithRestrictions(oldUser, &newUserRaw)

	err = DB.UpdateMainUser(newUser)
	if err != nil {
		slog.With("error", err).Error("Failed to update user")
		http.Error(w, "Failed to update user", http.StatusInternalServerError)
		return
	}

	if mergeErr != nil {
		http.Error(w, "Some fields are restricted and were not updated: "+mergeErr.Error(), http.StatusBadRequest)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	err = json.NewEncoder(w).Encode(newUser)
	if err != nil {
		slog.With("error", err).Error("Failed to encode updated user")
		http.Error(w, "Failed to encode updated user", http.StatusInternalServerError)
		return
	}
	mainUsersCacher.AskCacheRefresh()
}
