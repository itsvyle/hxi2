CREATE TABLE IF NOT EXISTS MAIN (
    user_id INTEGER PRIMARY KEY,
    hide INTEGER NOT NULL DEFAULT 0, -- Indique si l'utilisateur est caché (1) ou visible (0)
    display_name TEXT NOT NULL,
    pronouns TEXT NOT NULL,
    surnom TEXT NOT NULL, 
    origine TEXT NOT NULL, -- Lycée d'origine
    voeu TEXT NOT NULL, -- Vœu d'orientation post-prépa
    couleur TEXT NOT NULL, -- Couleur de l'utilisateur
    c_or_ocaml TEXT NOT NULL,
    fun_fact TEXT NOT NULL,
    conseil TEXT NOT NULL, -- Conseil pour les filleuls
    algebre_or_analyse TEXT NOT NULL, -- Préférence entre algèbre et analyse
    linux_distro TEXT NOT NULL,
    discord_username TEXT NOT NULL, -- Nom d'utilisateur Discord
    edit_restrictions INTEGER NOT NULL DEFAULT 0,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);