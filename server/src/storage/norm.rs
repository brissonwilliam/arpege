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
