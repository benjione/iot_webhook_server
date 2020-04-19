CREATE TABLE IF NOT EXISTS users (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  email TEXT NOT NULL,
  password TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS devices (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  registration_id TEXT NOT NULL,
  name TEXT NOT NULL,
  user INTEGER NOT NULL,
  online BOOLEAN NOT NULL,
  registration_date TEXT NOT NULL,
  last_login TEXT NOT NULL
);
