-- Add migration script here
CREATE TYPE report_status AS ENUM (
    'pending',
    'in_progress',
    'resolved',
    'dismissed'
);

CREATE TABLE IF NOT EXISTS reports (
    report_id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    track_id VARCHAR(22),
    ann_song_id INTEGER,
    message TEXT NOT NULL,
    user_name TEXT,
    user_mail TEXT,
    user_id VARCHAR(22),
    created_by TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    status report_status DEFAULT 'pending',
    handled_by VARCHAR(22) DEFAULT NULL
);

CREATE TABLE IF NOT EXISTS users (
    name TEXT,
    mail TEXT,
    id VARCHAR(22),
    binds INTEGER NOT NULL DEFAULT 0,
    flags BIGINT NOT NULL DEFAULT 0
);

ALTER TABLE animes
    DROP COLUMN vintage,
    ADD COLUMN vintage_release_season release_season,  -- Specify type for new column
    ADD COLUMN vintage_release_year INTEGER;