use crate::storage::storage;
use sqlx::{FromRow, QueryBuilder, Sqlite};

pub fn normalize_str(s: &str) -> String {
    let mut norm = String::from(s);
    norm = norm.to_lowercase();
    norm = norm.replace(",", "");
    norm = norm.replace(".", "");
    norm = norm.replace("-", "");
    norm = norm.replace(":", "");
    norm = norm.replace(";", "");
    norm = norm.replace("/", "");
    norm = norm.replace("#", "");
    norm = norm.replace("\\", "");
    norm = norm.replace("`", "");
    norm = norm.replace("(", "");
    norm = norm.replace(")", "");
    norm = norm.replace("\n", " ");
    norm = norm.replace("\r", " ");
    norm = norm.replace("\t", " ");
    return norm;
}

pub fn normalize_title(s: &str) -> String {
    let mut norm = normalize_str(s);
    norm = norm.replace("feat", "");
    norm = norm.replace("remastered", "");
    norm = norm.replace("remaster", "");
    norm = norm.replace("and", "&");
    return norm;
}

pub fn tokenize(s: &String) -> Vec<String> {
    let mut ret = Vec::new();

    for word in s.split(" ") {
        if word.len() < 1 {
            continue;
        }

        if word.len() <= 2 {
            let remainder = String::from(&word[0..word.len()]);
            ret.push(remainder);
            continue;
        }

        // push a sliding window of string
        let mut i = 1;
        while i + 1 < word.len() {
            let tok = String::from(&word[i - 1..i + 1]); // +1 because upper bound is excluded
            ret.push(tok);
            i += 1;
        }
    }
    return ret;
}

pub struct FindMetaCriteria {
    pub title: String,
    pub artists: String,
    pub duration_ms: u32,
    pub title_tokens: Vec<String>,
    pub artist_tokens: Vec<String>,
    pub album_tokens: Vec<String>,
}

impl FindMetaCriteria {
    pub fn new(title: String, duration_ms: u32, artists: String, album: String) -> Self {
        let title_norm = normalize_title(title.as_str());
        let title_tokens = tokenize(&title_norm); // cannot initialize inline, title_norm would be
                                                  // borrowed by other field

        let artist_norm = normalize_str(artists.as_str());
        let artist_tokens = tokenize(&artist_norm);

        let album_norm = normalize_str(album.as_str());
        let album_tokens = tokenize(&album_norm);

        return FindMetaCriteria {
            title: title_norm,
            duration_ms: duration_ms,
            artists: artists,
            title_tokens: title_tokens,
            artist_tokens: artist_tokens,
            album_tokens: album_tokens,
        };
    }
}

#[derive(Debug, FromRow)]
pub struct MetaMatch {
    pub id: Vec<u8>,
    pub title: String,
    pub weight: u8,
}

impl storage::Storage {
    pub async fn search_meta(&self, mut search: String) -> Result<Vec<MetaMatch>, sqlx::Error> {
        if search.len() < 3 {
            log::debug!("search_meta input too small");
            return Ok(vec![]);
        }

        const MAX_INPUT_LEN: usize = 1024;
        search.truncate(MAX_INPUT_LEN);
        let tokens = tokenize(&search);

        // with t(tok) AS (values('a'), ('b')) select * from t
        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"
            WITH input(token) AS (
                VALUES
            "#,
        );

        let mut qsep = qb.separated(",");
        for t in tokens.iter() {
            qsep.push_unseparated("(");
            qsep.push_bind_unseparated(t);
            qsep.push(")");
        }
        qb.push(")");

        qb.push(
            r#"
            SELECT id, title FROM song 
            WHERE id IN (
                SELECT song_id FROM song_token WHERE token IN (SELECT * FROM input)
            ) OR artist_id in (
                SELECT artist_id FROM artist_token WHERE token IN (SELECT * FROM input)
            ) OR album_id in (
                SELECT album_id FROM album_token WHERE token IN (SELECT * FROM input)
            )
            "#,
        );

        let res: Result<Vec<MetaMatch>, sqlx::error::Error> =
            qb.build_query_as::<MetaMatch>().fetch_all(&self.pool).await;

        return res;
    }

    // finds MetaMatch based on an additive set of criteria (AND...)
    pub async fn find_meta(
        &self,
        criteria: FindMetaCriteria,
    ) -> Result<Vec<MetaMatch>, sqlx::Error> {
        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"
            SELECT id, title FROM song 
            WHERE 1 
        "#,
        );

        // song_token filter
        if criteria.title_tokens.len() > 0 {
            qb.push(" AND id IN (SELECT song_id FROM song_token WHERE token IN (");
            let mut qsep = qb.separated(",");
            for t in criteria.title_tokens.iter() {
                qsep.push_bind(t.as_str());
            }
            qsep.push_unseparated("))");
        }

        // artist_token filter
        if criteria.artist_tokens.len() > 0 {
            qb.push(" AND artist_id IN (SELECT artist_id FROM artist_token WHERE token IN (");
            let mut qsep = qb.separated(",");
            for at in criteria.artist_tokens.iter() {
                qsep.push_bind(at.as_str());
            }
            qsep.push_unseparated("))");
        }

        // album_token filter
        if criteria.artist_tokens.len() > 0 {
            qb.push(" AND album_id IN (SELECT album_id FROM album_token WHERE token IN (");
            let mut qsep = qb.separated(",");
            for at in criteria.album_tokens.iter() {
                qsep.push_bind(at.as_str());
            }
            qsep.push_unseparated("))");
        }

        // duration filter
        if criteria.duration_ms > 0 {
            qb.push(" AND duration_ms BETWEEN ");
            let delta_duration = 5000;
            qb.push_bind(criteria.duration_ms - delta_duration);
            qb.push(" AND ");
            qb.push_bind(criteria.duration_ms + delta_duration);
        }

        let mut matches = qb
            .build_query_as::<MetaMatch>()
            .fetch_all(&self.pool)
            .await?;

        prune_by_weight(criteria, &mut matches);

        return Ok(matches);
    }
}

fn prune_by_weight(criteria: FindMetaCriteria, candidates: &mut Vec<MetaMatch>) {
    // Weights is a number that gives an idea of how much metadata
    // was actually matched depending on the request input
    // It is not used to give a match score, but rahter to compare candidates within
    // themselves to keep candidates that match the most entries based on how much metadata
    // collided
    const TITLE_WEIGHT: u8 = 10;
    const DURATION_WEIGHT: u8 = 15;
    const ARTIST_WEIGHT: u8 = 8;
    const ALBUM_WEIGHT: u8 = 2;
    const TOTAL_WEIGHT: u8 = TITLE_WEIGHT + DURATION_WEIGHT + ARTIST_WEIGHT + ALBUM_WEIGHT;

    for c in candidates.iter_mut() {
        let mut score = TITLE_WEIGHT;
        if criteria.duration_ms > 0 {
            score += DURATION_WEIGHT;
        }
        if criteria.artist_tokens.len() > 0 {
            // we matched an artist token, augment the score
            score += ARTIST_WEIGHT
        }
        if criteria.album_tokens.len() > 0 {
            score += ALBUM_WEIGHT
        }
        c.weight = score / TOTAL_WEIGHT;
    }

    // Prune, keep the best 10
    candidates.sort_by(|a, b| return a.weight.cmp(&b.weight));
    candidates.truncate(10);
}
