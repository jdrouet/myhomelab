CREATE TABLE counter_metrics (
    name TEXT NOT NULL,
    tags JSON NOT NULL,
    timestamp INTEGER NOT NULL,
    value INTEGER NOT NULL
);
