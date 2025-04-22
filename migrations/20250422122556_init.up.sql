CREATE TABLE IF NOT EXISTS notifications (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    app_name TEXT NOT NULL,
    app_icon TEXT, -- Nullable
    summary TEXT NOT NULL,
    body TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL
);
