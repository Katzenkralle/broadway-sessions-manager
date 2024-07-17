-- Your SQL goes here
CREATE TABLE invite_key (
     inv_key VARCHAR(64) PRIMARY KEY NOT NULL,
     unix_created_at BIGINT NOT NULL
);