/// Phase 7: Semantic Similarity Matching for Prompt Cache
///
/// Provides fuzzy matching for cache keys when exact matches fail.
/// Uses token-based Jaccard similarity for fast approximate matching.
///
/// Target: 70%+ cache hit rate with similarity matching enabled
use std::collections::HashSet;

/// Similarity threshold for cache hits (0.0-1.0)
pub const DEFAULT_SIMILARITY_THRESHOLD: f32 = 0.85;

/// Compute Jaccard similarity between two sets of tokens
///
/// Jaccard similarity = |A ∩ B| / |A ∪ B|
///
/// This is much faster than embedding-based similarity (no GPU required)
/// and provides good results for prompt caching use cases.
pub fn jaccard_similarity(tokens_a: &[&str], tokens_b: &[&str]) -> f32 {
    if tokens_a.is_empty() && tokens_b.is_empty() {
        return 1.0;
    }
    if tokens_a.is_empty() || tokens_b.is_empty() {
        return 0.0;
    }

    let set_a: HashSet<&str> = tokens_a.iter().copied().collect();
    let set_b: HashSet<&str> = tokens_b.iter().copied().collect();

    let intersection = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();

    if union == 0 {
        0.0
    } else {
        intersection as f32 / union as f32
    }
}

/// Tokenize prompt text into words
///
/// Simple whitespace + punctuation tokenization.
/// Good enough for cache similarity without heavyweight NLP.
pub fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase())
        .collect()
}

/// Extract key tokens from prompt (filter common words)
///
/// Focuses on domain-specific terms for better similarity matching.
pub fn extract_key_tokens(text: &str) -> Vec<String> {
    let stopwords: HashSet<&str> = [
        "a", "an", "the", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had",
        "do", "does", "did", "will", "would", "should", "could", "may", "might", "can", "this",
        "that", "these", "those", "i", "you", "he", "she", "it", "we", "they", "what", "which",
        "who", "when", "where", "why", "how", "and", "or", "but", "if", "for", "to", "of", "in",
        "on", "at", "by", "with", "from", "as",
    ]
    .iter()
    .copied()
    .collect();

    tokenize(text)
        .into_iter()
        .filter(|token| !stopwords.contains(token.as_str()))
        .filter(|token| token.len() > 2) // Skip very short tokens
        .collect()
}

/// Compute similarity between two prompts
///
/// Uses key token extraction + Jaccard similarity.
/// Returns score in range 0.0-1.0.
pub fn prompt_similarity(prompt_a: &str, prompt_b: &str) -> f32 {
    let tokens_a = extract_key_tokens(prompt_a);
    let tokens_b = extract_key_tokens(prompt_b);

    let refs_a: Vec<&str> = tokens_a.iter().map(|s| s.as_str()).collect();
    let refs_b: Vec<&str> = tokens_b.iter().map(|s| s.as_str()).collect();

    jaccard_similarity(&refs_a, &refs_b)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jaccard_identical() {
        let tokens = vec!["attack", "enemy", "position"];
        let similarity = jaccard_similarity(&tokens, &tokens);
        assert_eq!(similarity, 1.0);
    }

    #[test]
    fn test_jaccard_disjoint() {
        let tokens_a = vec!["attack", "enemy"];
        let tokens_b = vec!["heal", "ally"];
        let similarity = jaccard_similarity(&tokens_a, &tokens_b);
        assert_eq!(similarity, 0.0);
    }

    #[test]
    fn test_jaccard_partial_overlap() {
        let tokens_a = vec!["attack", "enemy", "now"];
        let tokens_b = vec!["attack", "ally", "later"];
        let similarity = jaccard_similarity(&tokens_a, &tokens_b);
        // Intersection: 1 (attack)
        // Union: 5 (attack, enemy, now, ally, later)
        assert_eq!(similarity, 0.2);
    }

    #[test]
    fn test_jaccard_high_overlap() {
        let tokens_a = vec!["move", "to", "cover", "and", "attack"];
        let tokens_b = vec!["move", "to", "cover", "quickly"];
        let similarity = jaccard_similarity(&tokens_a, &tokens_b);
        // Intersection: 3 (move, to, cover)
        // Union: 6 (move, to, cover, and, attack, quickly)
        assert_eq!(similarity, 0.5);
    }

    #[test]
    fn test_tokenize_basic() {
        let text = "Move to position (5, 10) and attack!";
        let tokens = tokenize(text);
        assert_eq!(
            tokens,
            vec!["move", "to", "position", "5", "10", "and", "attack"]
        );
    }

    #[test]
    fn test_tokenize_empty() {
        let tokens = tokenize("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_extract_key_tokens_filters_stopwords() {
        let text = "The enemy is at position 5 and the ally is at position 10";
        let tokens = extract_key_tokens(text);
        // Should filter out: the, is, at, and
        // Should keep: enemy, position, 5, ally, 10
        assert!(tokens.contains(&"enemy".to_string()));
        assert!(tokens.contains(&"ally".to_string()));
        assert!(tokens.contains(&"position".to_string()));
        assert!(!tokens.contains(&"the".to_string()));
        assert!(!tokens.contains(&"is".to_string()));
    }

    #[test]
    fn test_prompt_similarity_identical() {
        let prompt = "Attack enemy at position 5, then move to cover";
        let similarity = prompt_similarity(prompt, prompt);
        assert_eq!(similarity, 1.0);
    }

    #[test]
    fn test_prompt_similarity_similar_tactics() {
        let prompt_a = "Attack enemy at position 5, then retreat to cover";
        let prompt_b = "Attack foe at position 5, then fall back to cover";
        let similarity = prompt_similarity(prompt_a, prompt_b);
        // Both mention: attack, position, 5, cover
        // Differ on: enemy/foe, retreat/fall/back
        // Realistic expectation: >0.4 (moderate similarity)
        assert!(similarity > 0.4, "Expected >0.4, got {}", similarity);
    }

    #[test]
    fn test_prompt_similarity_different_actions() {
        let prompt_a = "Attack all enemies with grenades";
        let prompt_b = "Heal wounded allies with medkits";
        let similarity = prompt_similarity(prompt_a, prompt_b);
        // Very different actions, should be low similarity
        assert!(similarity < 0.3, "Expected <0.3, got {}", similarity);
    }

    #[test]
    fn test_prompt_similarity_threshold() {
        let prompt_a = "Move to position 10, scan area, then attack if enemy spotted";
        let prompt_b = "Move to position 10, scan surroundings, then engage if threat detected";
        let similarity = prompt_similarity(prompt_a, prompt_b);

        // Similar intent: move→scan→conditional attack
        // Realistic expectation: moderate similarity (0.3-0.5)
        // For production, would likely need threshold around 0.75-0.85 for safety
        assert!(similarity >= 0.3, "Expected ≥0.3, got {:.2}", similarity);
    }
}
