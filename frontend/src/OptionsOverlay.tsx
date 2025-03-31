import React from "react";
import { useState } from "react";
import './OptionsOverlay.css'

interface SettingsProp {
    settings: Settings,
    hide: () => void,
    onSettingsChange: (settings: Settings) => void,
}

export interface Settings {
    showOpenings: boolean,
    showInserts: boolean,
    showEndings: boolean,

    language: Language,
}

const OptionsOverlay: React.FC<SettingsProp> = ({ settings, hide, onSettingsChange }) => {
    return (
        <div className="settings-window-overlay">
            <div className="settings-popup-container">
                <div className="header">
                    <div className="dummy"></div>
                    <button className="close-button" onClick={hide}>X</button>
                    <h2 className="settings-header">Settings</h2>
                </div>
                <div className="setting-header">Filters</div>
                <div className="setting-buttons-container">
                    <button className={`setting-buttons ${settings.showOpenings ? "on-color" : "off-color"}`}
                        onClick={() => onSettingsChange({ ...settings, showOpenings: !settings.showOpenings })}>
                        Openings</button>
                    <button className={`setting-buttons ${settings.showInserts ? "on-color" : "off-color"}`}
                        onClick={() => onSettingsChange({ ...settings, showInserts: !settings.showInserts })}>
                        Inserts</button>
                    <button className={`setting-buttons ${settings.showEndings ? "on-color" : "off-color"}`}
                        onClick={() => onSettingsChange({ ...settings, showEndings: !settings.showEndings })}>
                        Endings</button>
                </div>
            </div>
        </div>
    );
};

export default OptionsOverlay;

export type Language = "eng" | "jpn";