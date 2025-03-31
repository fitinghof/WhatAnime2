import React from "react";
import AnimeEntry, { AnimeInfo, AnimeEntryConfig } from "./AnimeEntry";
import "./AnimeList.css";


interface AnimeListProps {
    animes: AnimeInfo[];
    list_config: ListConfig;
}

export interface ListConfig {
    openings: boolean,
    inserts: boolean,
    endings: boolean,
    show_confirm_button: boolean,
    seperator: string | null,
    spotify_song_id: string,
    after_anime_bind: () => void,
    open_report_window: (anime_ann_id: number) => void,
}

function visible(anime: AnimeInfo, list_config: ListConfig): boolean {
    const { index_type } = anime.bind.song_index;

    return (
        (index_type === "Ending" && list_config.endings) ||
        (index_type === "Opening" && list_config.openings) ||
        (index_type === "Insert" && list_config.inserts)
    );
}

const AnimeList: React.FC<AnimeListProps> = ({ animes, list_config }) => {
    let anime_config: AnimeEntryConfig =
    {
        show_confirm_button: list_config.show_confirm_button,
        spotify_song_id: list_config.spotify_song_id,
        after_anime_bind: list_config.after_anime_bind,
        open_report_window: list_config.open_report_window
    };

    let animes_filtered = animes.filter(value => visible(value, list_config));
    return (
        <>
            {animes_filtered.length != 0 && (
                <>
                    {
                        list_config.seperator && (
                            <div className="separator">
                                {list_config.seperator}
                            </div>
                        )
                    }

                    < div className="anime-list" id="animes" >
                        {
                            animes.map((anime, index) => (
                                <AnimeEntry key={index} anime={anime} config={anime_config} />
                            ))
                        }
                    </div>
                </>
            )
            }
        </>
    );
};

export default AnimeList;
