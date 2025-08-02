package main

import (
	"log/slog"
	"time"

	_ "embed"

	"github.com/jmoiron/sqlx"
	_ "github.com/mattn/go-sqlite3"
)

var DB *DatabaseManager

type DatabaseManager struct {
	DB *sqlx.DB
}

//go:embed schema.sql
var sqlSchema string

// panics on fail
func NewDatabaseManager(ConfigDBPath string) *DatabaseManager {
	sqlDB, err := sqlx.Open("sqlite3", ConfigDBPath)
	if err != nil {
		slog.With("error", err).Error("Failed to open database")
		panic(err)
	}
	err = sqlDB.Ping()
	if err != nil {
		slog.With("error", err).Error("Failed to ping database")
		panic(err)
	}

	sqlDB.MustExec(sqlSchema)

	slog.With("file", ConfigDBPath).Info("Connected to database")

	return &DatabaseManager{
		DB: sqlDB,
	}
}

type Main struct {
	UserID           int       `db:"user_id" json:"user_id"`
	DisplayName      string    `db:"display_name" json:"display_name"`
	Surnom           string    `db:"surnom" json:"surnom"`
	Origine          string    `db:"origine" json:"origine"`
	Voeu             string    `db:"voeu" json:"voeu"`
	Couleur          string    `db:"couleur" json:"couleur"`
	EditRestrictions int       `db:"edit_restrictions" json:"edit_restrictions"`
	UpdatedAt        time.Time `db:"updated_at" json:"updated_at"`
}
