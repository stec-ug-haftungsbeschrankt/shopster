-- Your SQL goes here


CREATE TABLE "settings" (      
    id SERIAL PRIMARY KEY,
    title TEXT UNIQUE NOT NULL,
    datatype TEXT NOT NULL,
    value TEXT NOT NULL
);
