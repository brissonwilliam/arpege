export interface ChunkMeta {
    id: string;
    start_ms: number;
    end_ms: number;
    /** Present on server JSON; optional on the client. */
    duration_ms?: number;
}

export interface SongMeta {
    id: string;
    title: string;
    artist: string;
    duration_ms: number;
    mime_type: string; // e.g. "audio/mpeg", "audio/mp4; codecs=\"mp4a.40.2\"", "audio/ogg; codecs=opus"
    chunks: ChunkMeta[];
}

export enum PlayerState {
    PLAYING = "play",
    PAUSED = "pause",
    ERROR = "err",
};
