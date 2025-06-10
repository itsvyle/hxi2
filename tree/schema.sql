CREATE TABLE IF NOT EXISTS parrainages (
    ID INTEGER PRIMARY KEY,
    parrain_id INTEGER NOT NULL,
    filleul_id INTEGER NOT NULL,
    date_added DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(parrain_id, filleul_id)
);