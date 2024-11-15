-- Add migration script here
CREATE TABLE IF NOT EXISTS users (
    id bigserial PRIMARY KEY,
    username text UNIQUE NOT NULL,
    created_at timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
);
