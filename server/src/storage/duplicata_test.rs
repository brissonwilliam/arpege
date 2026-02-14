#[cfg(test)]
mod tests {
    use crate::storage::duplicata::{normalize_str, normalize_title, tokenize};

    #[test]
    fn tokenize_input_empty() {
        let out = tokenize(String::from(""));
        let expected = vec![];
        assert_cmp(expected, out);
    }

    #[test]
    fn tokenize_input_less_than_tokensize() {
        let out = tokenize(String::from("a bb"));
        let expected = vec!["a", "bb"];
        assert_cmp(expected, out);
    }

    #[test]
    fn tokenize_input_eq_tokensize() {
        let out = tokenize(String::from("123 abc"));
        let expected = vec!["123", "abc"];
        assert_cmp(expected, out);
    }

    #[test]
    fn tokenize_input_bigger_tokensize_with_remainder() {
        let out = tokenize(String::from("123a 45678"));
        let expected = vec!["123", "23a", "456", "567", "678"];
        assert_cmp(expected, out);
    }

    #[test]
    fn tokenize_input_realistic() {
        let out = tokenize(String::from("call me maybe"));
        let expected = vec!["cal", "all", "me", "may", "ayb", "ybe"];
        assert_cmp(expected, out);
    }

    fn assert_cmp(expected: Vec<&str>, actual: Vec<String>) {
        assert_eq!(expected.len(), actual.len());

        let mut i = 0;
        for a in actual {
            assert_eq!(expected[i], a.as_str());
            i += 1;
        }
    }

    #[test]
    fn normalize_str_basic_lowercase() {
        assert_eq!(normalize_str("Hello World"), "hello world");
    }

    #[test]
    fn normalize_str_removes_punctuation() {
        let input = "A,B.C-D:E;F/G#H\\I`J(K)L";
        let expected = "abcdefghijkl";
        assert_eq!(normalize_str(input), expected);
    }

    #[test]
    fn normalize_str_preserves_spaces() {
        assert_eq!(normalize_str("Hello, World!"), "hello world!");
    }

    #[test]
    fn normalize_str_idempotent() {
        let s = "test-string";
        assert_eq!(normalize_str(&normalize_str(s)), "teststring");
    }

    #[test]
    fn normalize_title_removes_common_suffixes() {
        let input = "Song Title (Remastered)";
        let expected = "song title ";
        assert_eq!(normalize_title(input), expected);
    }

    #[test]
    fn normalize_title_removes_feat_and_live() {
        let input = "My Song feat Artist Live";
        let expected = "my song  artist ";
        assert_eq!(normalize_title(input), expected);
    }

    #[test]
    fn normalize_title_replaces_and_with_ampersand() {
        let input = "Rock and Roll";
        let expected = "rock & roll";
        assert_eq!(normalize_title(input), expected);
    }

    #[test]
    fn normalize_title_order_of_operations() {
        // ensure normalize_str runs before title-specific logic
        let input = "LIVE-AND-LOUD";
        let expected = "&loud";
        assert_eq!(normalize_title(input), expected);
    }
}
