CREATE TABLE IF NOT EXISTS todo (
    id          INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    content     TEXT    NOT NULL,
    expire_time DATETIME DEFAULT NULL
);
