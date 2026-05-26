DROP TABLE IF EXISTS game_server;

DELETE FROM users WHERE username = 'admin';

DROP INDEX IF EXISTS idx_users_username;
DROP TABLE IF EXISTS users;
