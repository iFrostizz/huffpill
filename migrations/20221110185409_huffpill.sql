-- Add migration script here
 
CREATE TABLE IF NOT EXISTS challenges
(
    name        TEXT    PRIMARY KEY NOT NULL,
    difficulty  INTEGER             NOT NULL,
    solves      BOOLEAN             NOT NULL DEFAULT 0,
    kind        VARCHAR(3)          NOT NULL
); 

CREATE TABLE IF NOT EXISTS users
(
    name        TEXT    PRIMARY KEY NOT NULL,
    port_in     INTEGER             NOT NULL,
    port_out    INTEGER             NOT NULL DEFAULT 0
);