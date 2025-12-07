-- Your SQL goes here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE "customers" (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email TEXT UNIQUE NOT NULL,
    email_verified BOOLEAN NOT NULL DEFAULT FALSE,
    algorithm TEXT NOT NULL,
    password TEXT NOT NULL,
    full_name TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP
);
