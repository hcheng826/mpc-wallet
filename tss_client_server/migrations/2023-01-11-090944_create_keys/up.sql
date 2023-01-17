-- Your SQL goes here
CREATE TABLE keys (
    id SERIAL PRIMARY KEY,
    address VARCHAR NOT NULL,
    local_share TEXT NOT NULL
)
