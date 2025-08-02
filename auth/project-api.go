package main

import (
	"encoding/json"
	"log/slog"
	"net/http"

	ggu "github.com/itsvyle/hxi2/global-go/utils"
)

func checkProjectApiAuth(w http.ResponseWriter, r *http.Request, necessaryPerms int) bool {
	if IsLocalDebugInstance {
		return true
	}
	authHeader := r.Header.Get("Authorization")
	if authHeader == "" {
		http.Error(w, "Unauthorized", http.StatusUnauthorized)
		return false
	}

	if len(authHeader) < 7 || authHeader[:7] != "Bearer " {
		http.Error(w, "Unauthorized", http.StatusUnauthorized)
		return false
	}

	token := authHeader[7:]
	apiUsers, err := apiUsersCacher.Get()
	if err != nil {
		http.Error(w, "Internal Server Error", http.StatusInternalServerError)
		return false
	}
	apiUser, ok := (*apiUsers)[token]
	if !ok {
		http.Error(w, "Unauthorized", http.StatusUnauthorized)
		return false
	}

	if !apiUser.HasPermission(necessaryPerms) {
		slog.With("user", apiUser.Username, "url", r.URL.Path, "needsPerms", necessaryPerms).Warn("Forbidden access to project API, but token is valid")
		http.Error(w, "Forbidden", http.StatusForbidden)
		return false
	}

	return true
}

func ProjectHandleListUsers(w http.ResponseWriter, r *http.Request) {
	if !checkProjectApiAuth(w, r, ggu.APIRoleListUsers) {
		return
	}

	users, err := DB.ListUsers()
	if err != nil {
		slog.With("error", err).Error("Error listing all users")
		http.Error(w, "Internal Server Error", http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	err = json.NewEncoder(w).Encode(users)
	if err != nil {
		slog.With("error", err).Error("Error encoding users")
		http.Error(w, "Internal Server Error", http.StatusInternalServerError)
		return
	}
}
