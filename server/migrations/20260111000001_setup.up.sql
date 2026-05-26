CREATE TABLE users (
    id            TEXT     PRIMARY KEY NOT NULL,
    username      TEXT                 NOT NULL  UNIQUE,
    password_hash TEXT                 NOT NULL,
    admin         INTEGER              NOT NULL  DEFAULT 0,
    created_at    TEXT                 NOT NULL  DEFAULT (datetime('now')),
    updated_at    TEXT                 NOT NULL  DEFAULT (datetime('now'))
);

CREATE INDEX idx_users_username ON users(username);

INSERT INTO users (id, username, password_hash, admin)
VALUES (
    '00000000-0000-0000-0000-000000000001',
    'admin',
    '$argon2id$v=19$m=16,t=2,p=1$M3hJelB0VGoxbUZjdlF6aw$2bTX0HqetQor0lrcYkkLzw',
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
