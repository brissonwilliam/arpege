pub struct FullSongContext<'a> {
    song: &'a Song,
    artist: &'a Artist,
    album: &'a Album,
    // Maybe add other things like album art ?
    // TODO: add metadata info (liked, seconds played, skipped, found in which playlist)
}

pub struct Song {
    id: Vec<u8>,
    title: String,
    duration_ms: u32,
    artist_id: Vec<u8>,
    album_id: Vec<u8>,
    track_number: i8,
}

pub struct Artist {
    id: Vec<u8>,
    name: String,
}

pub struct Album {
    id: Vec<u8>,
    artist_id: Vec<u8>,
    name: String,
    release_date: std::time::Instant,
    track_count: u8,
    duration_ms: u32,
}
