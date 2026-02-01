use std::{char, str::FromStr};

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
    duartion_ms: Option<u32>,
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
            duartion_ms: duration_ms,
            artist_tokens: artist_tokens,
        };
    }
}
