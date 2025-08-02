package main

import (
	"encoding/json"
	"log/slog"
	"net/http"

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
