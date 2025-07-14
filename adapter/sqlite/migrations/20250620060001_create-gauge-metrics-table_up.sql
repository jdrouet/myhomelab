CREATE TABLE gauge_metrics (
    name TEXT NOT NULL,
    tags JSON NOT NULL,
    timestamp INTEGER NOT NULL,
    value REAL NOT NULL
);
