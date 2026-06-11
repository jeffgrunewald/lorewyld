CREATE TABLE users (
    id            TEXT     PRIMARY KEY NOT NULL,
    username      TEXT                 NOT NULL  UNIQUE,
    email         TEXT                 NOT NULL,
    password_hash TEXT                 NOT NULL,
    admin         INTEGER              NOT NULL  DEFAULT 0,
    created_at    TEXT                 NOT NULL  DEFAULT (datetime('now')),
    updated_at    TEXT                 NOT NULL  DEFAULT (datetime('now')),
    CONSTRAINT uq_users_email UNIQUE (email)
);

CREATE INDEX idx_users_username ON users(username);

-- Bootstrap admin account, password 'admin' (argon2id) — change it
-- after first login.
INSERT INTO users (id, username, email, password_hash, admin)
VALUES (
    '00000000-0000-0000-0000-000000000001',
    'admin',
    'admin@localhost',
    '$argon2id$v=19$m=19456,t=2,p=1$s7Wo2pI9jPyxjy50Bmu7cQ$NkjZ6md9i2L0pN/QS0Tmkci6vqymgJ4wbTHspypQK2g',
    1
);

CREATE TABLE game_server (
    id         TEXT PRIMARY KEY NOT NULL,
    name       TEXT             NOT NULL,
    version    TEXT             NOT NULL,
    join_code  TEXT             NOT NULL,
    created_at TEXT             NOT NULL  DEFAULT (datetime('now')),
    updated_at TEXT             NOT NULL  DEFAULT (datetime('now'))
);
