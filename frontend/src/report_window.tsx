import React from "react";
import { useState } from "react";
import './report_window.css'

interface ReportInfo {
    ann_song_id: number | null,
    spotify_song_id: string,
    hide: () => void,
}

const ReportButton: React.FC<ReportInfo> = ({ ann_song_id, spotify_song_id, hide }) => {
    const [message, setReason] = useState(""); // Stores the report reason

    const handleSubmit = () => {
        const params = {
            spotify_id: spotify_song_id,
            ann_song_id: ann_song_id,
            message: message,
        };
        fetch("/api/report", {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify(params)
        })
            .then(response => response.text())
            .then(data => {
                console.log(data);
            })
        console.log("Report Submitted:", message, ann_song_id, spotify_song_id);
        hide(); // Close the popup after submitting
        setReason(""); // Reset the input field
    };

    return (
        <div>
            <div className="report-window-overlay">
                <div className="popup-container">
                    <h2 className="report-header">Report Issue</h2>
                    <textarea
                        value={message}
                        onChange={(e) => setReason(e.target.value)}
                        placeholder="What is the reason for the report?"
                        className="report-textarea"
                    />
                    <div className="popup-buttons">
                        <button
                            onClick={() => hide()}
                            className="popup-cancel"
                        >
                            Cancel
                        </button>
                        <button
                            onClick={handleSubmit}
                            className="popup-submit"
                        >
                            Submit
                        </button>
                    </div>
                </div>
            </div>
        </div>
    );
};

export default ReportButton;