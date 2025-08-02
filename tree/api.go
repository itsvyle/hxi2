package main

import (
	"encoding/json"
	"log/slog"
	"net/http"
	"strconv"
	"strings"

	ggu "github.com/itsvyle/hxi2/global-go/utils"
)

func HandleListUsers(w http.ResponseWriter, r *http.Request) {
	c, err := authManager.AuthenticateHTTPRequest(w, r, true)
	if err != nil || !c.CheckPermHTTP(w, ggu.RoleStudent) {
		return
	}

	users, err := usersCacher.Get()
	if err != nil {
		slog.With("error", err).Error("Failed to get users")
		http.Error(w, "Failed to get users", http.StatusInternalServerError)
		return
	}

	etag := strconv.FormatInt(usersCacher.LastUpdated, 10)
	w.Header().Set("ETag", etag)

	if match := r.Header.Get("If-None-Match"); match == etag {
		http.Error(w, http.StatusText(http.StatusNotModified), http.StatusNotModified)
		return
	}

	if users == nil || len(*users) == 0 {
		users = &[]ggu.ProjectUser{}
	}

	w.Header().Set("Cache-Control", "private, max-age=100, must-revalidate")
	w.Header().Set("Content-Type", "application/json")
	err = json.NewEncoder(w).Encode(*users)
	if err != nil {
		slog.With("error", err).Error("Failed to encode users")
		http.Error(w, "Failed to encode users", http.StatusInternalServerError)
		return
	}
}

func HandleListRelations(w http.ResponseWriter, r *http.Request) {
	c, err := authManager.AuthenticateHTTPRequest(w, r, true)
	if err != nil || !c.CheckPermHTTP(w, ggu.RoleStudent) {
		return
	}

	cachedRelations, err := relationsCacher.Get()
	if err != nil {
		slog.With("error", err).Error("Failed to get parrainages")
		http.Error(w, "Failed to get parrainages", http.StatusInternalServerError)
		return
	}

	userID := r.URL.Query().Get("userID")
	if userID != "" && userID != "-1" {
		userIDInt, err := ggu.ParseInt64(userID)
		if err != nil {
			http.Error(w, "Invalid userID", http.StatusBadRequest)
			return
		}

		userGraph, err := ExtractUserGraph(userIDInt, cachedRelations.Graph)
		if err != nil {
			slog.With("error", err).Error("Failed to get user graph")
			http.Error(w, "Failed to get user graph", http.StatusInternalServerError)
			return
		}

		mermaidCode, _, err := GenerateMermaidCodeFromGraph(userGraph)
		if err != nil {
			slog.With("error", err).Error("Failed to generate mermaid code")
			http.Error(w, "Failed to generate mermaid code", http.StatusInternalServerError)
			return
		}
		w.Header().Set("Content-Type", "text/plain")

		_, err = w.Write([]byte(mermaidCode))
		if err != nil {
			slog.With("error", err).Error("Failed to write mermaid code")
			http.Error(w, "Failed to write mermaid code", http.StatusInternalServerError)
			return
		}
		return
	}

	parrainages := &cachedRelations.Parrainages

	if len(*parrainages) == 0 {
		parrainages = &[]Parrainage{}
	}

	w.Header().Set("Content-Type", "application/json")
	err = json.NewEncoder(w).Encode(*parrainages)
	if err != nil {
		slog.With("error", err).Error("Failed to encode parrainages")
		http.Error(w, "Failed to encode parrainages", http.StatusInternalServerError)
		return
	}
}

func HandlePostRelation(w http.ResponseWriter, r *http.Request) {
	c, err := authManager.AuthenticateHTTPRequest(w, r, true)
	if err != nil || !c.CheckPermHTTP(w, ggu.RoleStudent) {
		return
	}

	parrainID := r.FormValue("parrainID")
	filleulID := r.FormValue("filleulID")
	if parrainID == "" || filleulID == "" {
		http.Error(w, "Missing parrainID or filleulID", http.StatusBadRequest)
		return
	}

	parrainIDInt, err := ggu.ParseInt64(parrainID)
	if err != nil {
		http.Error(w, "Invalid parrainID", http.StatusBadRequest)
		return
	}

	filleulIDInt, err := ggu.ParseInt64(filleulID)
	if err != nil {
		http.Error(w, "Invalid filleulID", http.StatusBadRequest)
		return
	}

	if parrainIDInt == filleulIDInt {
		http.Error(w, "parrainID and filleulID cannot be the same", http.StatusBadRequest)
		return
	}

	if !c.HasPermission(ggu.RoleAdmin) {
		_, err := usersCacher.GetNow()
		if err != nil {
			slog.With("error", err).Error("Failed to get users for parrainage check")
			http.Error(w, "Failed to get users", http.StatusInternalServerError)
			return
		}
		parrainPromo := -1
		filleulPromo := -1
		if parrainIDInt == c.IDInt() {
			parrainPromo = c.Promotion
			filleul, ok := allUsersMap[filleulIDInt]
			if !ok {
				http.Error(w, "Filleul not found", http.StatusBadRequest)
				return
			}
			filleulPromo = filleul.Promotion
		} else if filleulIDInt == c.IDInt() {
			filleulPromo = c.Promotion
			parrain, ok := allUsersMap[parrainIDInt]
			if !ok {
				http.Error(w, "Parrain not found", http.StatusBadRequest)
				return
			}
			parrainPromo = parrain.Promotion
		} else {
			http.Error(w, "You can only add relations which include yourself", http.StatusForbidden)
			return
		}

		if parrainPromo != filleulPromo-1 {
			http.Error(w, "You can only add relations between a generation and the one after!", http.StatusBadRequest)
			return
		}
	}

	newID, err := DB.AddParrainage(parrainIDInt, filleulIDInt)
	if err != nil {
		if strings.HasPrefix(err.Error(), "UNIQUE constraint failed:") {
			http.Error(w, "Relation already exists", http.StatusBadRequest)
			return
		}

		slog.With("error", err).Error("Failed to add parrainage")
		http.Error(w, "Failed to add parrainage", http.StatusInternalServerError)
		return
	}

	relationsCacher.AskCacheRefresh()
	globalTreeCacher.AskCacheRefresh()

	w.Header().Set("Content-Type", "application/json")
	err = json.NewEncoder(w).Encode(Parrainage{
		ID:        newID,
		ParrainID: parrainIDInt,
		FilleulID: filleulIDInt,
	})
	if err != nil {
		slog.With("error", err).Error("Failed to encode parrainage")
		http.Error(w, "Failed to encode parrainage", http.StatusInternalServerError)
		return
	}
}

func HandleDeleteRelation(w http.ResponseWriter, r *http.Request) {
	c, err := authManager.AuthenticateHTTPRequest(w, r, true)
	if err != nil || !c.CheckPermHTTP(w, ggu.RoleStudent) {
		return
	}

	relationID := r.FormValue("id")
	if relationID == "" {
		http.Error(w, "Missing id", http.StatusBadRequest)
		return
	}

	relationIDInt, err := ggu.ParseInt64(relationID)
	if err != nil {
		http.Error(w, "Invalid relationID", http.StatusBadRequest)
		return
	}

	parrainage, err := DB.GetParrainage(relationIDInt)
	if err != nil {
		slog.With("error", err, "id", relationIDInt).Error("Failed to get parrainage")
		http.Error(w, "Failed to get parrainage", http.StatusInternalServerError)
		return
	}

	if parrainage == nil {
		http.Error(w, "Relation not found", http.StatusBadRequest)
		return
	}

	if !c.HasPermission(ggu.RoleAdmin) {
		cid := c.IDInt()
		if cid != parrainage.ParrainID && cid != parrainage.FilleulID {
			http.Error(w, "You can only delete relations which include yourself", http.StatusForbidden)
			return
		}
	}

	err = DB.DeleteParrainage(relationIDInt)
	if err != nil {
		slog.With("error", err).Error("Failed to delete parrainage")
		http.Error(w, "Failed to delete parrainage", http.StatusInternalServerError)
		return
	}
	relationsCacher.AskCacheRefresh()
	globalTreeCacher.AskCacheRefresh()

	_, err = w.Write([]byte(`{"success":true}`))
	if err != nil {
		slog.With("error", err).Error("Failed to write success")
	}
}

func HandleGetGlobalTree(w http.ResponseWriter, r *http.Request) {
	c, err := authManager.AuthenticateHTTPRequest(w, r, true)
	if err != nil || !c.CheckPermHTTP(w, ggu.RoleStudent) {
		return
	}

	globalTree, err := globalTreeCacher.GetNow()
	if err != nil {
		slog.With(ggu.SlogHTTPInfo(r), "error", err).Error("Failed to get global tree")
		http.Error(w, "Failed to get global tree", http.StatusInternalServerError)
		return
	}

	w.Header().Set("ETag", globalTree.SVGHash)
	if match := r.Header.Get("If-None-Match"); match == globalTree.SVGHash {
		// If ETag matches, send Not Modified status
		http.Error(w, http.StatusText(http.StatusNotModified), http.StatusNotModified)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	err = json.NewEncoder(w).Encode(globalTree)
	if err != nil {
		slog.With(ggu.SlogHTTPInfo(r), "error", err).Error("Failed to encode global tree")
		http.Error(w, "Failed to encode global tree", http.StatusInternalServerError)
		return
	}
}
