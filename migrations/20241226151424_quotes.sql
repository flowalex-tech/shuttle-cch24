CREATE TABLE IF NOT EXISTS quotes (
    id uuid PRIMARY KEY,
    author text NOT NULL,
    quote text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    version int NOT NULL DEFAULT 1
);
