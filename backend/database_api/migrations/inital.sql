CREATE OR REPLACE FUNCTION sort_int_array(_arr INTEGER[])
RETURNS INTEGER[] AS $$
SELECT array_agg(elem ORDER BY elem)
FROM unnest(_arr) AS elem;
$$ LANGUAGE SQL IMMUTABLE;

CREATE TYPE song_category AS ENUM (
    'standard',
    'instrumental',
    'character',
    'chanting'
);

CREATE TABLE IF NOT EXISTS songs (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    artist_name TEXT NOT NULL,
    composer_name TEXT NOT NULL,
    arranger_name TEXT NOT NULL,
    category song_category,
    length DOUBLE PRECISION,
    is_dub BOOLEAN,
    hq TEXT,
    mq TEXT,
    audio TEXT,
    artists INTEGER[] NOT NULL,
    composers INTEGER[] NOT NULL,
    arrangers INTEGER[] NOT NULL
);

CREATE UNIQUE INDEX unique_song_artists_name
ON songs(name, sort_int_array(artists));
CREATE INDEX idx_song_name ON songs(name);
CREATE INDEX idx_song_artists ON songs USING GIN(artists);
CREATE INDEX idx_song_composers ON songs USING GIN(composers);

CREATE TABLE IF NOT EXISTS artists (
    id INTEGER PRIMARY KEY,
    names TEXT[] NOT NULL,
    line_up_id INTEGER,
    group_ids INTEGER[] NOT NULL,
    member_ids INTEGER[] NOT NULL
);

CREATE INDEX idx_artist_names ON artists(names);
CREATE INDEX idx_artist_group_ids ON artists USING GIN(group_ids);
CREATE INDEX idx_artist_member_ids ON artists USING GIN(member_ids);

CREATE TYPE anime_type AS ENUM (
    'tv',
    'movie',
    'ova',
    'ona',
    'special',
    'unknown'
);

CREATE TYPE anime_index_type AS ENUM (
    'season',
    'movie',
    'ona',
    'ova',
    'tv_special',
    'special',
    'music_video',
    'unknown'
);

CREATE TYPE media_format AS ENUM (
    'tv',
    'tv_short',
    'movie',
    'special',
    'ova',
    'ona',
    'music',
    'manga',
    'novel',
    'one_shot'
);

CREATE TYPE media_source AS ENUM (
    'original',
    'manga',
    'light_novel',
    'visual_novel',
    'video_game',
    'other',
    'novel',
    'doujinshi',
    'anime',
    'web_novel',
    'live_action',
    'game',
    'comic',
    'multi_media_project',
    'picture_book'
);

CREATE TYPE release_season AS ENUM (
    'winter',
    'spring',
    'summer',
    'fall'
);

CREATE TABLE IF NOT EXISTS animes (
    -- From Anisongdb --
    ann_id INTEGER PRIMARY KEY,
    eng_name TEXT NOT NULL,
    jpn_name TEXT NOT NULL,
    alt_names TEXT[] NOT NULL,
    vintage TEXT,

    -- linked ids
    myanimelist_id INTEGER,
    anidb_id INTEGER,
    anilist_id INTEGER,
    kitsu_id INTEGER,
    
    anime_type anime_type,

    -- anime index --
    index_type anime_index_type,
    index_number INTEGER NOT NULL,
    index_part SMALLINT NOT NULL,

    -- From Anilist --
    mean_score INTEGER,
    banner_image TEXT,
    -- Cover Image --
    cover_image_color VARCHAR(8),
    cover_image_medium TEXT,
    cover_image_large TEXT,
    cover_image_extra_large TEXT,
    format media_format,

    genres TEXT[],
    source media_source,
    -- studio array --
    studios_id INTEGER[],
    studios_name TEXT[],
    studios_url TEXT[],

    -- tag array
    tags_id INTEGER[],
    tags_name TEXT[],

    -- trailer
    trailer_id TEXT,
    trailer_site TEXT,
    trailer_thumbnail TEXT,

    episodes INTEGER,
    season release_season,
    season_year INTEGER
);

CREATE TYPE song_index_type AS ENUM (
    'opening',
    'insert',
    'ending'
);

CREATE TABLE IF NOT EXISTS anime_song_links (
    -- Bind -- 
    song_id INTEGER,
    anime_ann_id INTEGER,
    song_ann_id INTEGER PRIMARY KEY,

    -- Bind Info --
    difficulty DOUBLE PRECISION,
    song_index_type song_index_type,
    song_index_number INTEGER,
    is_rebroadcast BOOLEAN
);

CREATE INDEX anime_song_links_song_id ON anime_song_links(song_id);
CREATE INDEX anime_song_links_anime_ann_id ON anime_song_links(anime_ann_id);

CREATE TABLE IF NOT EXISTS spotify_song_links (
    spotify_id VARCHAR(22) NOT NULL,
    song_id INTEGER NOT NULL,
    PRIMARY KEY (spotify_id, song_id) 
);

CREATE TABLE IF NOT EXISTS spotify_artist_links (
    spotify_id VARCHAR(22) NOT NULL,
    artist_id INTEGER NOT NULL,
    PRIMARY KEY (spotify_id, artist_id)
);