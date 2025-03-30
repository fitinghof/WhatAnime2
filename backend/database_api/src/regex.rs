use fuzzywuzzy::fuzz;
use kakasi::{self, IsJapanese};
use regex::Regex;

pub fn process_possible_japanese(japanese: &str) -> String {
    if kakasi::is_japanese(japanese) == IsJapanese::False {
        japanese.to_string()
    } else {
        kakasi::convert(japanese).romaji
    }
}

pub fn normalize_text(text: &str) -> String {
    let new_text = text
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != ' ', "");
    return deunicode::deunicode(&new_text);
}

#[allow(dead_code)]
fn remove_vowels(word: &str) -> String {
    word.chars()
        .filter(|&c| !"aeiouAEIOU".contains(c))
        .collect()
}
#[allow(dead_code)]
fn remove_consonants(word: &str) -> String {
    word.chars().filter(|&c| "aeiouAEIOU".contains(c)).collect()
}

pub fn process_similarity(japanese_text: &str, romaji_text: &str) -> f32 {
    if kakasi::is_japanese(japanese_text) != IsJapanese::False {
        let romanized_japanese = process_possible_japanese(japanese_text);
        let normalized_japanese = normalize_text(&romanized_japanese);
        let normalized_romaji = normalize_text(romaji_text);
        let fuzz_value_full = fuzz::ratio(&normalized_japanese, &normalized_romaji);

        // let normalized_japanese_consonants = remove_vowels(&normalized_japanese)
        //     .replace("r", "l")
        //     .replace("b", "v");
        // let normalized_romaji_consonants = remove_vowels(&normalized_romaji)
        //     .replace("r", "l")
        //     .replace("b", "v");

        let fuzz_value_consonants = fuzz::ratio(&normalized_japanese, &normalized_romaji);
        let consonant_weight = 0.9;
        let full_weight = 1.0 - consonant_weight;

        let value = (fuzz_value_consonants as f32 * consonant_weight
            + fuzz_value_full as f32 * full_weight) as f32;
        value
    } else {
        let value =
            fuzz::ratio(&normalize_text(japanese_text), &normalize_text(romaji_text)) as f32;
        value
    }
}

use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Captures;
use std::collections::HashMap;

lazy_static! {

    static ref REPLACEMENT_RULES: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();
        map.insert("ļ", "[ļĻ]");
        map.insert("l", "[l˥ļĻΛ]");
        map.insert("ź", "[źŹ]");
        map.insert("z", "[zźŹ]");
        map.insert("ou", "(ou|ō|o)");
        map.insert("oo", "(oo|ō|o)");
        map.insert("oh", "(oh|ō|o)");
        map.insert("wo", "(wo|o)");
        map.insert("ō", "[Ōō]");
        map.insert("o", "([oōŌóòöôøӨΦο]|ou|oo|oh|wo)");
        map.insert("uu", "(uu|u|ū)");
        map.insert("ū", "[ūŪ]");
        map.insert("u", "([uūŪûúùüǖμ]|uu)");
        map.insert("aa", "(aa|a)");
        map.insert("ae", "(ae|æ)");
        map.insert("λ", "[λΛ]");
        map.insert("a", "([aäãά@âàáạåæā∀Λ]|aa)");
        map.insert("c", "[cςč℃Ↄ]");
        map.insert("é", "[éÉ]");
        map.insert("e", "[eəéÉêёëèæē]");
        map.insert("'", "['’ˈ]");
        map.insert("n", "[nñ]");
        map.insert("0", "[0Ө]");
        map.insert("2", "[2²₂]");
        map.insert("3", "[3³]");
        map.insert("5", "[5⁵]");
        map.insert("*", "[*✻＊✳︎]");
        map.insert(" ", "([^\\w]+|_+)");
        map.insert("i", "([iíίɪ]|ii)");
        map.insert("x", "[x×]");
        map.insert("b", "[bßβ]");
        map.insert("r", "[rЯ]");
        map.insert("s", "[sς]");
        map
    };

    // Build a single regex that matches all keys in REPLACEMENT_RULES
    static ref REPLACEMENT_REGEX: Regex = {
        let pattern = REPLACEMENT_RULES.keys()
            .map(|key| regex::escape(key))
            .collect::<Vec<String>>()
            .join("|"); // Join with `|` to create an "OR" regex
        Regex::new(&pattern).unwrap()
    };

    static ref ARTIST_REGEX: Regex = {
        Regex::new(&r".*?\((CV|Vo)(:|\.)\s*(?P<a>.*?)\)").unwrap()
    };
}

/// Takes the actual artist name from 'Perhaps a character (CV: Actual Artist)' or returns original string
pub fn process_artist_name(name: &str) -> String {
    ARTIST_REGEX.replace_all(name, "$a").trim().to_string()
}
/// simply unwraps possible (CV:artistname) before calling create_regex
pub fn create_artist_regex(input: Vec<&String>, whole_word_match: bool) -> String {
    input
        .iter()
        .map(|a| {
            let parsed_artist = ARTIST_REGEX.replace_all(a, "$a");
            create_regex(&parsed_artist, whole_word_match)
        })
        .join("|")
}

/// Replaces using a precompiled regex
pub fn create_regex(input: &str, whole_word_match: bool) -> String {
    if whole_word_match {
        format!(
            "^{}$",
            REPLACEMENT_REGEX.replace_all(input, |caps: &Captures| {
                let matched = caps.get(0).unwrap().as_str();

                REPLACEMENT_RULES.get(matched).map_or_else(
                    || matched.to_string(),
                    |&replacement| replacement.to_string(),
                )
            })
        )
    } else {
        REPLACEMENT_REGEX
            .replace_all(input, |caps: &Captures| {
                let matched = caps.get(0).unwrap().as_str();

                REPLACEMENT_RULES.get(matched).map_or_else(
                    || matched.to_string(),
                    |&replacement| replacement.to_string(),
                )
            })
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TESTING_LIST: &[(&str, &str)] = &[
        ("デート ・ ア ・ ライブ", "Date A Live"),
        ("モンスター ハンター", "Monster Hunter"),
        ("ファイナル ファンタジー", "Final Fantasy"),
        ("オンライン ゲーム", "Online Game"),
        ("レジェンド オブ ゼルダ", "Legend of Zelda"),
        ("ポケット モンスター", "Pocket Monster"),
        ("ドラゴン クエスト", "Dragon Quest"),
        ("キングダム ハーツ", "Kingdom Hearts"),
        ("ストリート ファイター", "Street Fighter"),
        ("スーパーマリオ", "Super Mario"),
    ];

    const TEST_LIST_FAIL: &[(&str, &str)] = &[
        ("又三郎", "Shayou"),
        ("こんにちは", "Hello"),
        ("ありがとう", "Thank You"),
        ("バナナ", "Bandana"),
        ("コーヒー", "Cough"),
        ("ホテル", "Hostel"),
        ("スピーカー", "Spiker"),
        ("マイク", "Mice"),
        ("バイク", "Back"),
        ("チェック", "Chick"),
    ];

    fn test_similarity(test: (&str, &str), success_function: impl Fn(f32) -> bool) -> f32 {
        let score: f32 = process_similarity(test.0, test.1);
        if !success_function(score) {
            println!("Failed Test: {:?}, Score: {}", test, score);
        }
        println!("{}", score);
        score
    }

    fn test_all(tests: &[(&str, &str)], success_function: impl Fn(f32) -> bool) -> f64 {
        let mut total_score = 0.0;
        for test in tests {
            total_score += test_similarity(*test, &success_function);
        }
        total_score as f64 / tests.len() as f64
    }

    #[test]
    fn test_deltas() {
        let fail_limit = 60.0;

        println!("--------------- Doing match tests ---------------");
        let average_success_score = test_all(&TESTING_LIST, |a| a > fail_limit);

        println!("--------------- Doing False Match tests ---------------");
        let average_fail_score = test_all(&TEST_LIST_FAIL, |a| a < fail_limit);

        println!("Average Success Score: {}", average_success_score);
        println!("Average Fail Score: {}", average_fail_score);
        let delta = average_success_score - average_fail_score;
        println!("Delta: {}", delta);
        assert!(delta > 10.0);
    }
}
