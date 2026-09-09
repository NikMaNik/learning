use rusqlite::Connection;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Word {
    pub id: i64,
    pub english: String,
    pub russian: String,
    pub ease_factor: f64,
    pub interval: i64,
    pub repetitions: i64,
    pub next_review: String,
    pub created_at: String,
    pub last_quality: Option<i64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActiveScreen {
    Main,
    Review,
    Forgot,
    Upcoming,
    Add,
    Update,
    UpdateSearch,
    UpdateEdit,
    Delete,
    DeleteConfirm,
    Import,
    ImportResult,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputField {
    English,
    Russian,
    Path,
    ReviewTranslation,
}

pub struct App {
    pub words: Vec<Word>,
    pub review_queue: Vec<Word>,
    pub forgotten_queue: Vec<Word>,
    pub upcoming_words: Vec<Word>,
    pub upcoming_days: i64,
    pub upcoming_input: String,
    pub current_word_index: usize,
    pub current_screen: ActiveScreen,
    pub selected: usize,
    pub input_english: String,
    pub input_russian: String,
    pub input_path: String,
    pub input_review: String,
    pub search_query: String,
    pub search_results: Vec<Word>,
    pub active_field: InputField,
    pub status_message: String,
    pub status_is_error: bool,
    pub show_review_answer: bool,
    pub last_quality: u8,
    pub last_distance: usize,
    pub last_next_interval: i64,
    pub import_count: usize,
    pub should_quit: bool,
    pub db: Connection,
}

impl App {
    pub fn new(db: Connection) -> Self {
        Self {
            words: Vec::new(),
            review_queue: Vec::new(),
            forgotten_queue: Vec::new(),
            upcoming_words: Vec::new(),
            upcoming_days: 2,
            upcoming_input: String::new(),
            current_word_index: 0,
            current_screen: ActiveScreen::Main,
            selected: 0,
            input_english: String::new(),
            input_russian: String::new(),
            input_path: String::new(),
            input_review: String::new(),
            search_query: String::new(),
            search_results: Vec::new(),
            active_field: InputField::English,
            status_message: String::new(),
            status_is_error: false,
            show_review_answer: false,
            last_quality: 0,
            last_distance: 0,
            last_next_interval: 0,
            import_count: 0,
            should_quit: false,
            db,
        }
    }

    pub fn refresh_words(&mut self) {
        if let Ok(words) = crate::db::get_all_words(&self.db) {
            self.words = words;
        }
    }

    pub fn refresh_review_queue(&mut self) {
        if let Ok(queue) = crate::db::get_words_for_review(&self.db) {
            self.review_queue = queue;
            self.current_word_index = 0;
            self.show_review_answer = false;
        }
    }

    pub fn refresh_forgotten_queue(&mut self) {
        if let Ok(queue) = crate::db::get_forgotten_words(&self.db) {
            self.forgotten_queue = queue;
        }
    }

    pub fn refresh_upcoming(&mut self) {
        if let Ok(words) = crate::db::get_words_upcoming(&self.db, self.upcoming_days) {
            self.upcoming_words = words;
        }
    }

    pub fn current_review_word(&self) -> Option<&Word> {
        self.review_queue.get(self.current_word_index)
    }

    pub fn clear_inputs(&mut self) {
        self.input_english.clear();
        self.input_russian.clear();
        self.input_path.clear();
        self.input_review.clear();
        self.search_query.clear();
        self.search_results.clear();
        self.upcoming_input.clear();
        self.status_message.clear();
        self.status_is_error = false;
        self.show_review_answer = false;
        self.selected = 0;
    }

    pub fn total_words(&self) -> usize {
        self.words.len()
    }

    pub fn words_for_review_count(&self) -> usize {
        self.review_queue.len()
    }

    pub fn forgotten_count(&self) -> usize {
        self.forgotten_queue.len()
    }

    pub fn upcoming_count(&self) -> usize {
        self.upcoming_words.len()
    }

    pub fn search_words(&mut self, query: &str) {
        self.search_results.clear();
        let query_lower = query.to_lowercase();

        // Exact match first
        for word in &self.words {
            if word.english.to_lowercase() == query_lower
                || word.russian.to_lowercase() == query_lower
            {
                self.search_results.push(word.clone());
            }
        }

        // If no exact match, find similar words
        if self.search_results.is_empty() {
            let mut scored: Vec<(i64, &Word)> = self
                .words
                .iter()
                .map(|w| (similarity_score(&query_lower, &w.english.to_lowercase()), w))
                .filter(|(score, _)| *score > 0)
                .collect();
            scored.sort_by_key(|a| std::cmp::Reverse(a.0));
            for (_, word) in scored.into_iter().take(5) {
                self.search_results.push(word.clone());
            }
        }
    }
}

fn similarity_score(query: &str, target: &str) -> i64 {
    if query.is_empty() || target.is_empty() {
        return 0;
    }

    let mut score = 0i64;

    // Prefix match
    if target.starts_with(query) {
        score += 100;
    }

    // Contains match
    if target.contains(query) {
        score += 50;
    }

    // Levenshtein-like: count matching characters in order
    let mut qi = query.chars();
    let mut matches = 0i64;
    let mut last_pos = 0;
    for (i, tc) in target.chars().enumerate() {
        if let Some(qc) = qi.clone().next() {
            if qc == tc {
                matches += 1;
                if i == last_pos {
                    matches += 2; // bonus for consecutive
                }
                last_pos = i + 1;
                qi.next();
            }
        }
    }
    score += matches;

    // Length similarity
    let len_diff = (query.len() as i64 - target.len() as i64).abs();
    score += 10 - len_diff.min(10);

    score
}

/// Levenshtein distance between two strings
pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();
    let len1 = s1_chars.len();
    let len2 = s2_chars.len();

    let mut matrix = vec![vec![0usize; len2 + 1]; len1 + 1];

    for (i, row) in matrix.iter_mut().enumerate().take(len1 + 1) {
        row[0] = i;
    }
    for (j, cell) in matrix[0].iter_mut().enumerate().take(len2 + 1) {
        *cell = j;
    }

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] {
                0
            } else {
                1
            };
            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }

    matrix[len1][len2]
}

/// Map Levenshtein distance to SM-2 quality (0-5)
/// 0 = exact match, higher = more different
pub fn distance_to_quality(distance: usize, answer_len: usize) -> u8 {
    if answer_len == 0 {
        return 0;
    }

    // Normalized distance (0.0 = exact, 1.0 = completely different)
    let normalized = distance as f64 / answer_len as f64;

    if normalized == 0.0 {
        5 // Perfect match
    } else if normalized < 0.2 {
        4 // Very close (1-2 chars off)
    } else if normalized < 0.4 {
        3 // Close (some mistakes)
    } else if normalized < 0.6 {
        2 // Far (many mistakes)
    } else if normalized < 0.8 {
        1 // Very far
    } else {
        0 // Almost completely wrong
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_identical() {
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
    }

    #[test]
    fn test_levenshtein_one_edit() {
        assert_eq!(levenshtein_distance("hello", "hallo"), 1);
    }

    #[test]
    fn test_levenshtein_completely_different() {
        assert_eq!(levenshtein_distance("abc", "xyz"), 3);
    }

    #[test]
    fn test_distance_to_quality_exact() {
        assert_eq!(distance_to_quality(0, 5), 5);
    }

    #[test]
    fn test_distance_to_quality_close() {
        // distance=1, answer_len=5, normalized=0.2 → quality 3
        assert_eq!(distance_to_quality(1, 5), 3);
    }

    #[test]
    fn test_distance_to_quality_far() {
        assert!(distance_to_quality(4, 5) <= 2);
    }
}
