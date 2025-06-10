package main

import (
	"log/slog"

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

type Parrainage struct {
	ID        int64  `db:"ID" json:"ID"`
	ParrainID int64  `db:"parrain_id" json:"parrainID"`
	FilleulID int64  `db:"filleul_id" json:"filleulID"`
	DateAdded string `db:"date_added" json:"dateAdded"`
}

// if forID=-1, list all
func (db *DatabaseManager) ListParrainage(forID int64) ([]Parrainage, error) {
	var parrainages []Parrainage
	var err error
	if forID == -1 {
		err = db.DB.Select(&parrainages, "SELECT * FROM parrainages")
	} else {
		err = db.DB.Select(&parrainages, "SELECT * FROM parrainages WHERE parrain_id = ? OR filleul_id = ?", forID, forID)
	}
	return parrainages, err

}

func (db *DatabaseManager) AddParrainage(parrainID int64, filleulID int64) (int64, error) {
	res, err := db.DB.Exec("INSERT INTO parrainages (parrain_id, filleul_id, date_added) VALUES (?, ?, datetime('now'))", parrainID, filleulID)
	if err != nil {
		return 0, err
	}
	newID, err := res.LastInsertId()
	if err != nil {
		return 0, err
	}

	return newID, err
}

func (db *DatabaseManager) GetParrainage(id int64) (*Parrainage, error) {
	var parrainage Parrainage
	err := db.DB.Get(&parrainage, "SELECT * FROM parrainages WHERE ID = ?", id)
	return &parrainage, err
}

func (db *DatabaseManager) DeleteParrainage(id int64) error {
	_, err := db.DB.Exec("DELETE FROM parrainages WHERE ID = ?", id)
	return err
}
