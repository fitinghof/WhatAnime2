import React, { useEffect, useState } from "react";
import "./AnimeEntry.css";
import { Language } from "./OptionsOverlay";

function parseAnimeIndex(animeIndex: AnimeIndex): string {
  switch (animeIndex.index_type) {
    case "Season":
      return `Season ${animeIndex.number || 1}`;
    case "Movie":
      return `Movie ${animeIndex.number || 1}`;
    case "ONA":
      return `ONA ${animeIndex.number || 1}`;
    case "OVA":
      return `OVA ${animeIndex.number || 1}`;
    case "TVSpecial":
      return `TV Special ${animeIndex.number || 1}`;
    case "Special":
      return `Special ${animeIndex.number || 1}`;
    case "MusicVideo":
      return `Music Video ${animeIndex.number || 1}`;
    default:
      return "Unknown season";
  }
}

function parseTrackIndex(track: AnimeTrackIndex): string {
  if (!track) return "";

  switch (track.index_type) {
    case "Opening":
      return `Opening ${track.index || "1"}`;
    case "Insert":
      return `Insert Song ${track.index || ""}`;
    case "Ending":
      return `Ending ${track.index || "1"}`;
    default:
      return "";
  }
}

interface AnimeEntryProps {
  anime: AnimeInfo;
  config: AnimeEntryConfig;
}

export interface AnimeEntryConfig {
  show_confirm_button: boolean;
  spotify_song_id: string;
  language: Language;
  after_anime_bind: () => void;
  open_report_window: (anisong_ann_id: number) => void;
}

function linked_ids(anime_ids: AnimeListLinks) {
  if (anime_ids === undefined) return null;
  return (
    <div className="anime-links">
      {anime_ids.myanimelist && (
        <a
          href={`https://myanimelist.net/anime/${anime_ids.myanimelist}`}
          target="_blank"
          rel="noopener noreferrer"
        >
          MAL
        </a>
      )}
      {anime_ids.anilist && (
        <a
          href={`https://anilist.co/anime/${anime_ids.anilist}`}
          target="_blank"
          rel="noopener noreferrer"
        >
          Anilist
        </a>
      )}
      {anime_ids.anidb && (
        <a
          href={`https://anidb.net/anime/${anime_ids.anidb}`}
          target="_blank"
          rel="noopener noreferrer"
        >
          AniDB
        </a>
      )}
      {anime_ids.kitsu && (
        <a
          href={`https://kitsu.io/anime/${anime_ids.kitsu}`}
          target="_blank"
          rel="noopener noreferrer"
        >
          Kitsu
        </a>
      )}
    </div>
  );
}

const AnimeEntry: React.FC<AnimeEntryProps> = ({ anime, config }) => {
  const [showMoreInfo, setShowMoreInfo] = useState(false);

  useEffect(() => {
    setShowMoreInfo(false);
  }, [anime]);

  const handleConfirmClick = () => {
    const params = {
      song_id: anime.song.id,
      spotify_song_id: config.spotify_song_id,
    };
    fetch("/api/confirm_anime", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(params),
    })
      .then((response) => response.text())
      .then((data) => {
        console.log(data);
        config.after_anime_bind();
      });
  };

  let animeSongNumber = parseTrackIndex(anime.bind.song_index);
  let animeIndex = parseAnimeIndex(anime.anime.anime_index);
  let source = formatSource(anime.anime.source);
  let release_season = formatReleaseSeason(anime.anime.vintage?.season);
  let title =
    config.language === "eng" ? anime.anime.eng_name : anime.anime.jpn_name;
  return (
    <div
      className="anime-item"
      style={{
        backgroundImage: `linear-gradient(rgba(0, 0, 0, 0.5), rgba(0, 0, 0, 0.5)), url(${anime.anime.banner_image ?? "/amq_icon_green.svg"
          })`,
      }}
      onClick={() => setShowMoreInfo(!showMoreInfo)}
    >
      <div className="left-info-container">
        <img
          src={anime.anime.cover_image.medium ?? "/amq_icon_green.svg"}
          alt="Anime art"
          className="anime-art"
          onError={(e) => {
            e.currentTarget.src = "/amq_icon_green.svg"; // Fallback to SVG
          }}
        />
        {showMoreInfo && (
          <div className="report-button-container">
            <button
              onClick={(event) => {
                event.stopPropagation();
                config.open_report_window(anime.bind.song_ann_id);
              }}
              className="report-button"
            >
              Report
            </button>
          </div>
        )}
      </div>
      <div className="anisong-info-container">
        <div className="anime-title">{title || "Unknown Anime"}</div>
        {showMoreInfo && (
          <div className="anisong-info">
            <div className="extra-info">
              <div className="anime-song-separator">Song info</div>

              {/* Insert x */}
              <div className="anime-info-text">{animeSongNumber}</div>

              {/* Song name */}
              <div className="anime-info-text">
                {`Song: ${anime.song.name}`}
              </div>

              {/* Artists */}
              <div className="anime-info-text">
                {`Artists: ${anime.song.artists
                  .map((a) => a.names[0])
                  .join(", ")}`}
              </div>

              {/* Composers */}
              <div className="anime-info-text">
                {`Composers: ${anime.song.composers
                  .map((a) => a.names[0])
                  .join(", ")}`}
              </div>

              {/* Arrangers */}
              <div className="anime-info-text">
                {`Arrangers: ${anime.song.arrangers
                  .map((a) => a.names[0])
                  .join(", ")}`}
              </div>
            </div>
            {showMoreInfo && (
              <div
                className="extra-info"
                onClick={() => setShowMoreInfo(!showMoreInfo)}
              >
                <div className="anime-song-separator">Anime info</div>

                {/* Season */}
                <div className="anime-info-text">{`${animeIndex}`}</div>

                {/* Episodes */}
                {anime.anime.episodes && (
                  <div className="anime-info-text">
                    {`Episodes: ${anime.anime.episodes}`}
                  </div>
                )}

                {/* Release date */}
                {anime.anime.vintage && (
                  <div className="anime-info-text">
                    {`Release: ${release_season} ${anime.anime.vintage?.year}`}
                  </div>
                )}

                {/* Source */}
                {source && (
                  <div className="anime-info-text">{`Source: ${source}`}</div>
                )}

                {/* Anime Type */}
                <div className="anime-info-text">
                  {`Type: ${anime.anime.anime_type || "Unknown"}`}
                </div>

                {/* Anime Type */}
                {anime.anime.genres.length !== 0 && (
                  <div className="anime-info-text">
                    {`Genres: ${anime.anime.genres.join(", ")}`}
                  </div>
                )}

                {/* Studios */}
                {anime.anime.studios.nodes.length !== 0 && false && (
                  <div className="anime-info-text">
                    {`Studios: ${anime.anime.studios.nodes.map((a) => a.name).join(", ")}`}
                  </div>
                )}
              </div>
            )}
          </div>
        )}
        {showMoreInfo && linked_ids(anime.anime.linked_ids)}
      </div>
      <div className="right-info-container">
        <div className="anime-score">
          <div className="score-text">{anime.anime.mean_score ?? ""}</div>
        </div>
        {config.show_confirm_button && showMoreInfo && (
          <button
            className="bind-anime-button"
            onClick={(event) => {
              event.stopPropagation();
              handleConfirmClick();
            }}
          >
            <p>
              Confirm<br></br> Anime
            </p>
          </button>
        )}
      </div>
    </div>
  );
};

export default AnimeEntry;

export interface AnimeInfo {
  anime: DBAnime; // Anime information
  song: SimplifiedAnisongSong; // Song information
  bind: DBAnisongBind; // Binding information between song and anime
}

export interface DBAnime {
  ann_id: number; // AnisongDB anime ID
  eng_name: string; // English name of the anime
  jpn_name: string; // Japanese name of the anime
  alt_name: string[]; // Alternative names for the anime
  vintage?: Release; // Release season and year
  linked_ids: AnimeListLinks; // Linked IDs for external databases
  anime_type?: AnimeType; // Type of the anime (e.g., TV, Movie, OVA)
  anime_index: AnimeIndex; // Index information for the anime
  mean_score?: number; // Mean score of the anime
  banner_image?: string; // URL for the banner image
  cover_image: CoverImage; // Cover image information
  format?: MediaFormat; // Format of the anime (e.g., TV, Movie)
  genres: string[]; // List of genres
  source?: MediaSource; // Source material (e.g., Manga, Light Novel)
  studios: StudioConnection; // Studio information
  tags: MediaTag[]; // Tags associated with the anime
  trailer?: MediaTrailer; // Trailer information
  episodes?: number; // Number of episodes
  season?: ReleaseSeason; // Release season (e.g., Spring, Summer)
  season_year?: number; // Release year
}

export interface AnimeIndex {
  index_type: AnimeIndexType; // Type of the anime index (e.g., Season, Movie, OVA)
  number: number; // Index number (e.g., season number, movie number)
  part: number; // Part of the index (e.g., part 1, part 2)
}

export type AnimeIndexType =
  | "Season"
  | "Movie"
  | "ONA"
  | "OVA"
  | "TVSpecial"
  | "Special"
  | "MusicVideo"
  | "Unknown";

export interface SimplifiedAnisongSong {
  id?: number; // Song ID
  name: string; // Name of the song
  artist_name: string; // Name of the main artist
  composer_name: string; // Name of the composer
  arranger_name: string; // Name of the arranger
  category: SongCategory; // Category of the song (e.g., Opening, Ending)
  length?: number; // Length of the song in seconds
  is_dub: boolean; // Whether the song is a dub
  hq?: string; // High-quality audio URL
  mq?: string; // Medium-quality audio URL
  audio?: string; // Audio URL
  artists: SimplifiedArtist[]; // List of artists
  composers: SimplifiedArtist[]; // List of composers
  arrangers: SimplifiedArtist[]; // List of arrangers
}

export interface DBAnisongBind {
  song_id?: number; // Song ID
  anime_ann_id?: number; // AnisongDB anime ID
  song_ann_id: number; // AnisongDB song ID
  difficulty?: number; // Difficulty rating
  song_index: AnimeTrackIndex; // Song index information
  is_rebroadcast: boolean; // Whether the song is for a rebroadcast
}

export interface SimplifiedArtist {
  names: string[]; // List of names for the artist
  id: number; // Artist ID
  line_up_id?: number; // Line-up ID
  group_ids: number[]; // IDs of groups the artist belongs to
  member_ids: number[]; // IDs of members in the group
}

export interface Release {
  season: ReleaseSeason; // Release season (e.g., Spring, Summer)
  year: number; // Release year
}

export interface AnimeListLinks {
  myanimelist?: number; // MyAnimeList ID
  anidb?: number; // AniDB ID
  anilist?: number; // AniList ID
  kitsu?: number; // Kitsu ID
}

export interface CoverImage {
  large: string; // URL for the large cover image
  medium: string; // URL for the medium cover image
}

export interface StudioConnection {
  nodes: Studio[]; // List of studio edges
}

// export interface StudioEdge {
//   node: Studio; // Studio information
// }

export interface Studio {
  id: number; // Studio ID
  name: string; // Studio name
}

export interface MediaTag {
  id: number; // Tag ID
  name: string; // Tag name
}

export interface MediaTrailer {
  id: string; // Trailer ID
  site: string; // Site hosting the trailer (e.g., YouTube)
}

export interface AnimeTrackIndex {
  index: number; // Index of the song (e.g., Opening 1, Ending 2)
  index_type: SongIndexType;
}

export type SongIndexType = "Opening" | "Insert" | "Ending";

export type AnimeType = "TV" | "Movie" | "OVA" | "ONA" | "Special";

export type ReleaseSeason = "SPRING" | "SUMMER" | "FALL" | "WINTER";

export type MediaFormat =
  | "TV"
  | "TV_SHORT"
  | "MOVIE"
  | "OVA"
  | "ONA"
  | "SPECIAL";

export type MediaSource =
  | "MANGA"
  | "LIGHT_NOVEL"
  | "ORIGINAL"
  | "GAME"
  | "OTHER";

export type SongCategory = "Opening" | "Ending" | "Insert";

function formatReleaseSeason(
  release_season: ReleaseSeason | undefined
): string | null {
  switch (release_season) {
    case "SPRING":
      return "Spring";
    case "SUMMER":
      return "Summer";
    case "FALL":
      return "Fall";
    case "WINTER":
      return "Winter";
  }
  return null;
}

function formatSource(source: MediaSource | undefined): string | null {
  switch (source) {
    case "GAME":
      return "Game";
    case "LIGHT_NOVEL":
      return "Light Novel";
    case "MANGA":
      return "Manga";
    case "ORIGINAL":
      return "Original";
    case "OTHER":
      return null;
  }
  return null;
}
