
import "./SongInfo.css";

export interface SongInfo {
    song_name: string; // Name of the song
    song_artists: string[]; // List of song artists
    album_image: string; // URL for the album image
    spotify_song_id: string; // Spotify track ID
}

interface SongConProps {
    song_info: SongInfo;
    showSettingsOverlay: () => void;
}

const SongContainer: React.FC<SongConProps> = ({ song_info, showSettingsOverlay }) => {
    return (
        <div className="now-playing-container">
            <div className="now-playing">
                <img
                    className="album-art"
                    src={song_info.album_image ? song_info.album_image : "/amq_icon_green.svg"}
                    alt="Album cover"
                    onError={(e) => {
                        e.currentTarget.src = "/amq_icon_green.svg"; // Fallback to SVG
                    }}
                />
                <div className="song-info">
                    <h1 className="song-title">
                        {song_info ? song_info.song_name : "No song info"}
                    </h1>
                    <p className="artist-name">
                        {song_info ? song_info.song_artists.join(", ") : ""}
                    </p>
                </div>
                <button className="settings-button" onClick={showSettingsOverlay}></button>
            </div>
        </div>
    );
}

export default SongContainer;