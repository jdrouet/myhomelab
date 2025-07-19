CREATE TABLE metrics (
    name TEXT NOT NULL,
    tags JSON NOT NULL,
    timestamp INTEGER NOT NULL,
    value JSON NOT NULL
);
