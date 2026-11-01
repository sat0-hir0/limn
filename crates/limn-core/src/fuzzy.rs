//! Fuzzy matching for the command palette and file search.
//!
//! A query matches a candidate when every query character appears in
//! the candidate in order (case-insensitive subsequence). Matches are
//! ranked by a score that rewards consecutive runs and characters that
//! start a word, so `cp` ranks "Command Palette" above "accept".

/// Score `candidate` against `query`. Returns `None` when `query` is
/// not a subsequence of `candidate`. An empty query matches everything
/// with score 0. Higher is better.
#[must_use]
pub fn score(query: &str, candidate: &str) -> Option<u32> {
    let query: Vec<char> = query
        .chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect();
    if query.is_empty() {
        return Some(0);
    }

    let mut total = 0u32;
    let mut qi = 0;
    let mut prev_matched = false;
    let mut prev: Option<char> = None;
    for c in candidate.chars() {
        if qi == query.len() {
            break;
        }
        let matched = c.to_lowercase().eq(std::iter::once(query[qi]));
        if matched {
            total += 1;
            if prev_matched {
                total += 4;
            }
            if prev.is_none_or(is_separator)
                || (c.is_uppercase() && prev.is_some_and(char::is_lowercase))
            {
                total += 6;
            }
            qi += 1;
        }
        prev_matched = matched;
        prev = Some(c);
    }

    (qi == query.len()).then_some(total)
}

/// Filter `items` by `query` and return the indices of the matches,
/// best first. Ties keep the original order, so callers control the
/// fallback ordering (e.g. alphabetical paths).
pub fn filter<T>(query: &str, items: &[T], key: impl Fn(&T) -> &str) -> Vec<usize> {
    let mut scored: Vec<(usize, u32)> = items
        .iter()
        .enumerate()
        .filter_map(|(i, item)| score(query, key(item)).map(|s| (i, s)))
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    scored.into_iter().map(|(i, _)| i).collect()
}

fn is_separator(c: char) -> bool {
    matches!(c, ' ' | '/' | '\\' | '-' | '_' | '.')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_query_matches_everything() {
        assert_eq!(score("", "anything"), Some(0));
    }

    #[test]
    fn subsequence_is_required() {
        assert!(score("opn", "Open File").is_some());
        assert!(score("xyz", "Open File").is_none());
        assert!(score("fo", "Open File").is_none());
    }

    #[test]
    fn matching_is_case_insensitive() {
        assert!(score("OPEN", "open file").is_some());
    }

    #[test]
    fn word_starts_and_runs_rank_higher() {
        let word_start = score("of", "Open File").unwrap();
        let scattered = score("of", "proof").unwrap();
        assert!(word_start > scattered);
    }

    #[test]
    fn filter_orders_best_first_and_keeps_ties_stable() {
        let items = ["notes/todo.md", "daily/2026-11-01.md", "todo.md"];
        let hits = filter("todo", &items, |s| s);
        assert_eq!(hits.len(), 2);
        assert!(hits.contains(&0) && hits.contains(&2));
        assert_eq!(filter("", &items, |s| s), vec![0, 1, 2]);
    }
}
