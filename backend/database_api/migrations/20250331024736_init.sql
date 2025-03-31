-- Add migration script here
CREATE OR REPLACE FUNCTION sort_int_array(_arr INTEGER[])
RETURNS INTEGER[] AS $$
SELECT array_agg(elem ORDER BY elem)
FROM unnest(_arr) AS elem;
$$ LANGUAGE SQL IMMUTABLE;

CREATE TYPE song_category AS ENUM (
    'standard',
    'instrumental',
    'character',
    'chanting',
    'no_category'
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
    vintage_release_season release_season,  -- Specify type for new column
    vintage_release_year INTEGER,

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

CREATE VIEW anisong_view AS SELECT
    s.id AS song_id,
    s.name AS song_name,
    s.artist_name,
    s.composer_name,
    s.arranger_name,
    s.category AS song_category,
    s.length AS song_length,
    s.is_dub AS song_is_dub,
    s.hq,
    s.mq,
    s.audio,
    s.artists AS artist_ids,
    s.composers AS composer_ids,
    s.arrangers AS arranger_ids,

    a.ann_id AS anime_ann_id,
    a.eng_name AS anime_eng_name,
    a.jpn_name AS anime_jpn_name,
    a.alt_names AS anime_alt_names,
    a.vintage_release_season AS anime_vintage_season,
    a.vintage_release_year AS anime_vintage_year,
    a.myanimelist_id,
    a.anidb_id,
    a.anilist_id,
    a.kitsu_id,
    a.anime_type,
    a.index_type AS anime_index_type,
    a.index_number AS anime_index_number,
    a.index_part AS anime_index_part,
    a.mean_score AS anime_mean_score,
    a.banner_image AS anime_banner_image,
    a.cover_image_color AS anime_cover_image_color,
    a.cover_image_medium AS anime_cover_image_medium,
    a.cover_image_large AS anime_cover_image_large,
    a.cover_image_extra_large AS anime_cover_image_extra_large,
    a.format AS anime_format,
    a.genres AS anime_genres,
    a.source AS anime_source,
    a.studios_id AS anime_studios_id,
    a.studios_name AS anime_studios_name,
    a.studios_url AS anime_studios_url,
    a.tags_id AS anime_tags_id,
    a.tags_name AS anime_tags_name,
    a.trailer_id AS anime_trailer_id,
    a.trailer_site AS anime_trailer_site,
    a.trailer_thumbnail AS anime_trailer_thumbnail,
    a.episodes AS anime_episodes,
    a.season AS anime_season,
    a.season_year AS anime_season_year,    

    asl.difficulty,
    asl.song_ann_id,
    asl.song_index_type,
    asl.song_index_number,
    asl.is_rebroadcast,
	    -- Aggregate artists into an array of JSON objects
    COALESCE((
        SELECT jsonb_agg(jsonb_build_object('id', ar.id, 'names', ar.names, 'line_up_id', ar.line_up_id, 'group_ids', ar.group_ids, 'member_ids', ar.member_ids))
        FROM artists ar
        WHERE ar.id = ANY(s.artists)
    ), '[]') AS artists,

    -- Aggregate composers into an array of JSON objects
    COALESCE((
        SELECT jsonb_agg(jsonb_build_object('id', co.id, 'names', co.names, 'line_up_id', co.line_up_id, 'group_ids', co.group_ids, 'member_ids', co.member_ids))
        FROM artists co
        WHERE co.id = ANY(s.composers)
    ), '[]') AS composers,

    -- Aggregate arrangers into an array of JSON objects
    COALESCE((
        SELECT jsonb_agg(jsonb_build_object('id', ae.id, 'names', ae.names, 'line_up_id', ae.line_up_id, 'group_ids', ae.group_ids, 'member_ids', ae.member_ids))
        FROM artists ae
        WHERE ae.id = ANY(s.arrangers)
    ), '[]') AS arrangers

FROM anime_song_links asl
JOIN animes a ON asl.anime_ann_id = a.ann_id
JOIN songs s ON asl.song_id = s.id;
