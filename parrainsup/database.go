package main

import (
	"fmt"
	"log/slog"
	"reflect"
	"strings"
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

type MainUser struct {
	UserID           int64     `db:"user_id" json:"user_id"`
	Hide             bool      `db:"hide" json:"hide"`
	DisplayName      string    `db:"display_name" json:"display_name"`
	Surnom           string    `db:"surnom" json:"surnom"`
	Origine          string    `db:"origine" json:"origine"`
	Voeu             string    `db:"voeu" json:"voeu"`
	Couleur          string    `db:"couleur" json:"couleur"`
	COrOcaml         string    `db:"c_or_ocaml" json:"c_or_ocaml"`
	FunFact          string    `db:"fun_fact" json:"fun_fact"`
	Conseil          string    `db:"conseil" json:"conseil"`
	AlgebreOrAnalyse string    `db:"algebre_or_analyse" json:"algebre_or_analyse"`
	Pronouns         string    `db:"pronouns" json:"pronouns"`
	DiscordUsername  string    `db:"discord_username" json:"discord_username"`
	EditRestrictions int       `db:"edit_restrictions" json:"edit_restrictions"`
	UpdatedAt        time.Time `db:"updated_at" json:"updated_at"`
}

var EditRestrictionKeys = map[string]int{
	"user_id":            0, // never editable
	"updated_at":         0, // never editable
	"edit_restrictions":  0, // never editable
	"display_name":       1,
	"surnom":             2,
	"origine":            4,
	"voeu":               8,
	"couleur":            16,
	"c_or_ocaml":         32,
	"fun_fact":           64,
	"conseil":            128,
	"algebre_or_analyse": 256,
	"pronouns":           512,
	"hide":               -1,
}

// MergeUserWithRestrictions merges the wanted user data into the returned user, following the edit restrictions.
// Returns an error with the restricted fields if the wanted user tries to edit them - it will still return the merged user.
func MergeUserWithRestrictions(oldUser *MainUser, wanted *MainUser) (*MainUser, error) {
	newUser := *oldUser

	var restrictedFields []string

	oldVal := reflect.ValueOf(oldUser).Elem()
	wantedVal := reflect.ValueOf(wanted).Elem()
	newVal := reflect.ValueOf(&newUser).Elem()
	typ := oldVal.Type()

	for i := range typ.NumField() {
		field := typ.Field(i)
		jsonTag := field.Tag.Get("json")
		if jsonTag == "" || jsonTag == "-" {
			continue
		}

		restrictionBit, ok := EditRestrictionKeys[jsonTag]
		if !ok || restrictionBit == 0 {
			continue
		}

		oldField := oldVal.Field(i)
		wantedField := wantedVal.Field(i)
		if restrictionBit == -1 {
			newVal.Field(i).Set(wantedField)
			continue
		}

		if !reflect.DeepEqual(oldField.Interface(), wantedField.Interface()) {
			if wanted.EditRestrictions&restrictionBit != 0 {
				restrictedFields = append(restrictedFields, jsonTag)
			} else {
				newVal.Field(i).Set(wantedField)
			}
		}
	}

	// check if the newVal for couleur is a valid hex color
	if newUser.Couleur != "" {
		if !strings.HasPrefix(newUser.Couleur, "#") || len(newUser.Couleur) != 7 {
			newUser.Couleur = oldUser.Couleur // revert to old value
			return &newUser, fmt.Errorf("couleur must be a valid hex color (e.g. #RRGGBB)")
		}
		if _, err := fmt.Sscanf(newUser.Couleur, "#%02x%02x%02x", newUser.Couleur[1:3], newUser.Couleur[3:5], newUser.Couleur[5:7]); err != nil {
			newUser.Couleur = oldUser.Couleur // revert to old value
			return &newUser, fmt.Errorf("couleur must be a valid hex color (e.g. #RRGGBB)")
		}
	}

	if len(restrictedFields) > 0 {
		return &newUser, fmt.Errorf("you cannot edit the following fields: %s", strings.Join(restrictedFields, ", "))
	}

	return &newUser, nil
}

func (db *DatabaseManager) ListVisibleMainUsers() (map[int64]*MainUser, error) {
	var users []MainUser
	err := db.DB.Select(&users, "SELECT * FROM MAIN WHERE hide = 0")
	if err != nil {
		return nil, err
	}

	userMap := make(map[int64]*MainUser, len(users))
	for _, u := range users {
		userMap[u.UserID] = &u
	}

	return userMap, nil
}
