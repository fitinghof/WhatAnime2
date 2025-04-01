import { useEffect, useState } from "react";
import { AnimeInfo } from "./AnimeEntry"; // use AnimeInfo and AnimeEntry from AnimeEntry.tsx
import AnimeList, { ListConfig } from "./AnimeList"
import ReportButton from "./report_window";
import SongContainer, { SongInfo } from "./SongInfo";
import OptionsOverlay, { Settings } from "./OptionsOverlay";

const defaultState: SongUpdate = {
    song_info: {
        song_name: "Not Playing Anything",
        romanized_song_name: "Not Playing Anything",
        song_artists: [],
        romanized_artists: [],
        album_image: "/amq_icon_green.svg",
        spotify_song_id: "",
    },
    anisongs: { miss: { possible: [] } }
}

const defaultSettings: Settings = {
    showOpenings: true,
    showInserts: true,
    showEndings: true,
    romanizeSongInfo: false,
    language: "eng",
}

interface ReportData {
    show: boolean,
    song_ann_id: number | null,
}

const Update = () => {
    const [reportOverlay, setReportOverlay] = useState<ReportData>({ show: false, song_ann_id: null });
    const [settings, setSettings] = useState<Settings>(defaultSettings);
    const [showSettings, setShowSettings] = useState<boolean>(false);
    const [info, setInfo] = useState<SongUpdate>(defaultState);

    const [list_config, setListConfig] = useState<ListConfig>({
        openings: true,
        inserts: true,
        endings: true,
        show_confirm_button: false,
        seperator: null,
        spotify_song_id: "",
        language: "eng",
        after_anime_bind: () => fetchUpdate(true, false),
        open_report_window: (song_ann_id: number) => setReportOverlay({ show: true, song_ann_id: song_ann_id }),
    })

    const onSettingsUpdate = (settings: Settings) => {
        setSettings(settings);
        setListConfig((p) => ({
            ...p,
            openings: settings.showOpenings,
            inserts: settings.showInserts,
            endings: settings.showEndings,
            language: settings.language,
        }))
    }

    const fetchUpdate = (refresh: boolean = false, recursive: boolean = true) => {
        const fetch_address = `/api/update${refresh ? "?refresh=true" : ""}`;
        //const fetch_address = "";
        fetch(fetch_address, { credentials: "include" })
            .then((response) => response.json())
            .then((data: Update) => {
                console.log(data)
                if (typeof data === "string") {
                    switch (data) {
                        case "login_required": {
                            window.location.href = "/api/login";
                            break;
                        }
                        case "unauthorized": {
                            setInfo({
                                song_info: {
                                    song_name: "You likely need to ask Simon for User Approval, spotify is annoying that way.",
                                    song_artists: ["You will have to provide your spotify mail and full name for approval"],
                                    album_image: "/amq_icon_green.svg",
                                    spotify_song_id: "",
                                    romanized_song_name: "You likely need to ask Simon for User Approval, spotify is annoying that way.",
                                    romanized_artists: ["You will have to provide your spotify mail and full name for approval"],
                                },
                                anisongs: { miss: { possible: [] } }
                            });
                            break;
                        }
                        case "no_updates":
                        case "not_playing":
                    }
                } else {
                    setInfo(data.new_song);
                    const anisongs = data.new_song.anisongs;
                    let show_button = true;
                    if ("hit" in anisongs) {
                        show_button = !(anisongs.hit.certainty === 100);
                    }
                    setListConfig((p) => ({
                        ...p,
                        show_confirm_button: show_button,
                        spotify_song_id: data.new_song.song_info.spotify_song_id,
                    }))
                }
            })
            .catch((err) => console.error(err));
        if (recursive) {
            setTimeout(fetchUpdate, 5000);
        }
    };

    useEffect(() => {
        // Run immediately, then every 5 seconds (5000ms)
        return () => fetchUpdate(true);
    }, []);

    return (
        <>
            {reportOverlay.show && (
                <ReportButton
                    track_id={info.song_info.spotify_song_id}
                    hide={() => setReportOverlay((p) => ({ ...p, show: false, }))}
                    ann_song_id={reportOverlay.song_ann_id}>
                </ReportButton>
            )}

            {showSettings && (
                <OptionsOverlay settings={settings} hide={() => setShowSettings(false)}
                    onSettingsChange={onSettingsUpdate}></OptionsOverlay>
            )}

            <SongContainer song_info={info.song_info} showSettingsOverlay={() => setShowSettings(true)} romanizeContent={settings.romanizeSongInfo}>
            </SongContainer>

            {"hit" in info.anisongs ? (
                <div>
                    <AnimeList animes={info.anisongs.hit.hits}
                        list_config={{ ...list_config, seperator: `Match ${info.anisongs.hit.certainty} %` }}></AnimeList>
                    <AnimeList animes={info.anisongs.hit.more_by_artists}
                        list_config={{ ...list_config, seperator: "More by artists" }}></AnimeList>
                </div>
            ) : "miss" in info.anisongs ? (
                <div>
                    <AnimeList animes={info.anisongs.miss.possible}
                        list_config={{ ...list_config, seperator: "Possible matches" }}></AnimeList>
                </div>
            ) : null // null case should be impossible
            }
        </>
    );

};

export default Update;

export type Update =
    | "no_updates"
    | "login_required"
    | "unauthorized"
    | "not_playing"
    | { new_song: SongUpdate };

export interface SongUpdate {
    song_info: SongInfo; // Information about the current song
    anisongs: Anisongs; // Anisongs data (hit or miss)
}

export type Anisongs =
    | { hit: NewSongHit }
    | { miss: NewSongMiss };

export interface NewSongHit {
    hits: AnimeInfo[]; // List of matching songs
    more_by_artists: AnimeInfo[]; // Additional songs by the same artists
    certainty: number; // Certainty score for the match
}

export interface NewSongMiss {
    possible: AnimeInfo[]; // List of possible matches
}