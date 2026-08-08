CREATE TABLE IF NOT EXISTS USERS (
    ID INTEGER PRIMARY KEY,
    username TEXT NOT NULL,
    first_name TEXT NOT NULL,
    last_name TEXT,
    discord_id TEXT NOT NULL UNIQUE,
    account_created_date DATETIME DEFAULT CURRENT_TIMESTAMP,
    account_modified_date DATETIME DEFAULT CURRENT_TIMESTAMP,
    promotion SMALLINT DEFAULT 0, --first year of mp2i,
    permissions INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS REFRESH_TOKENS (
    associated_user_id INTEGER NOT NULL,
    refresh_token_hash TEXT NOT NULL UNIQUE, -- SHA-256 hash (32 bytes)
    jti_hash TEXT NOT NULL UNIQUE, -- SHA-256 hash (32 bytes)
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS API_TOKENS (
    id INTEGER PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    token TEXT NOT NULL UNIQUE,
    permissions INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS ONE_TIME_CODES (
    ID INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    code_hash TEXT NOT NULL UNIQUE, -- SHA-256 hash (32 bytes)
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES USERS(ID) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS TEMPORARY_CODES (
    ID INTEGER PRIMARY KEY,
    username TEXT NOT NULL UNIQUE, -- a service name
    code_hash TEXT NOT NULL UNIQUE, -- SHA-256 hash (32 bytes)
    recheck_after INTEGER NOT NULL DEFAULT 0, -- seconds before rechecking with auth server if service is still valid - 0 means no recheck
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME DEFAULT CURRENT_TIMESTAMP
)