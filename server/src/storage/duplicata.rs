use std::time::Duration;

use crate::storage::storage;
use sqlx::{Execute, FromRow, QueryBuilder, Sqlite};

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
    return norm;
}

pub fn normalize_title(s: &str) -> String {
    let mut norm = normalize_str(s);
    norm = norm.replace("feat", "");
    norm = norm.replace("remastered", "");
    norm = norm.replace("remaster", "");
    norm = norm.replace("live", "");
    norm = norm.replace("and", "&");
    return norm;
}

pub fn tokenize(s: String) -> Vec<String> {
    let mut ret = Vec::new();

    for word in s.split(" ") {
        if word.len() < 1 {
            continue;
        }

        if word.len() <= 3 {
            let remainder = String::from(&word[0..word.len()]);
            ret.push(remainder);
            continue;
        }

        // push a sliding window of string
        let mut i = 1;
        while i + 1 < word.len() {
            let tok = String::from(&word[i - 1..i + 2]); // +2 because upper bound is excluded
            ret.push(tok);
            i += 1;
        }
    }
    return ret;
}

pub struct PruneDuplicataCriteria {
    title_tokens: Vec<String>,
    duration_ms: Option<u32>,
    artist_tokens: Vec<String>,
}

impl PruneDuplicataCriteria {
    pub fn new(title: String, duration_ms: Option<u32>, artists: Vec<String>) -> Self {
        let title_norm = normalize_title(title.as_str());
        let title_tokens = tokenize(title_norm);

        let mut artist_tokens = Vec::new();
        for a in artists {
            let norm = normalize_str(a.as_str());
            let toks = tokenize(norm);
            artist_tokens.extend(toks);
        }

        return PruneDuplicataCriteria {
            title_tokens,
            duration_ms,
            artist_tokens: artist_tokens,
        };
    }
}

#[derive(Debug, FromRow)]
pub struct Duplicata {}

impl storage::Storage {
    pub async fn get_duplicates(
        &self,
        criteria: PruneDuplicataCriteria,
    ) -> Result<Vec<Duplicata>, sqlx::Error> {
        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"
            SELECT id FROM song 
            WHERE id IN (SELECT song_id FROM song_token WHERE token IN (
        "#,
        );
        let mut qsep = qb.separated(",");
        for t in criteria.title_tokens.iter() {
            qsep.push_bind(t.as_str());
        }
        qsep.push_unseparated("))");

        if let Some(duration_ms) = criteria.duration_ms {
            qb.push(" AND duration_ms BETWEEN ");
            let delta_duration = 3000;
            qb.push_bind(duration_ms - delta_duration);
            qb.push(" AND ");
            qb.push_bind(duration_ms + delta_duration);
        }

        if criteria.artist_tokens.len() > 0 {
            qb.push(" AND artist_id IN ( ");
            let mut qsep = qb.separated(",");
            for at in criteria.artist_tokens.iter() {
                qsep.push_bind(at.as_str());
            }
            qsep.push_unseparated(" )");
        }

        let res = qb.build_query_as::<Duplicata>().fetch_all(&self.pool).await;

        return res;
    }
}
