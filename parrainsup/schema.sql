CREATE TABLE IF NOT EXISTS MAIN (
    user_id INTEGER PRIMARY KEY,
    display_name TEXT NOT NULL,
    pronouns TEXT NOT NULL,
    surnom TEXT NOT NULL, 
    origine TEXT NOT NULL, -- Lycée d'origine
    voeu TEXT NOT NULL, -- Vœu d'orientation post-prépa
    couleur TEXT NOT NULL, -- Couleur de l'utilisateur
    c_or_ocaml TEXT NOT NULL,
    fun_fact TEXT NOT NULL,
    conseil TEXT NOT NULL, -- Conseil pour les filleuls
    edit_restrictions INTEGER NOT NULL DEFAULT 0,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);