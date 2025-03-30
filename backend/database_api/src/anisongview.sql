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