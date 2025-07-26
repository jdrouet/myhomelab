CREATE TABLE events (
    source JSON NOT NULL,
    timestamp INTEGER NOT NULL,
    level TEXT NOT NULL,
    message TEXT NOT NULL,
    attributes JSON NOT NULL DEFAULT '{}'
);
CREATE INDEX events_header ON events (source, level, timestamp);
