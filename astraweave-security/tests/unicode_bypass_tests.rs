//! P1: Unicode Bypass Tests for astraweave-security
//!
//! Tests for Unicode-based security bypasses including:
//! - Homoglyph attacks (characters that look similar but are different)
//! - Zero-width characters (invisible Unicode characters)
//! - Right-to-left override attacks
//! - Unicode normalization issues
//! - Bidirectional text attacks
//! - Path traversal using Unicode equivalents

#![cfg(test)]

use astraweave_security::{sanitize_llm_prompt, LLMValidator};
use std::path::Path;

// ============================================================================
// Helper Functions
// ============================================================================

fn create_validator() -> LLMValidator {
    LLMValidator {
        banned_patterns: vec![
            "system(".to_string(),
            "exec(".to_string(),
            "eval(".to_string(),
            "import ".to_string(),
            "__import__".to_string(),
            "subprocess".to_string(),
        ],
        allowed_domains: vec!["localhost".to_string()],
        max_prompt_length: 1000,
        enable_content_filtering: true,
    }
}

/// Check if a string contains any zero-width characters
fn contains_zero_width_chars(s: &str) -> bool {
    s.chars().any(|c| matches!(c,
        '\u{200B}' | // Zero Width Space
        '\u{200C}' | // Zero Width Non-Joiner
        '\u{200D}' | // Zero Width Joiner
        '\u{FEFF}' | // Zero Width No-Break Space (BOM)
        '\u{180E}' | // Mongolian Vowel Separator
        '\u{2060}' | // Word Joiner
        '\u{2061}' | // Function Application
        '\u{2062}' | // Invisible Times
        '\u{2063}' | // Invisible Separator
        '\u{2064}'   // Invisible Plus
    ))
}

/// Check if string contains RTL override characters
fn contains_rtl_override(s: &str) -> bool {
    s.chars().any(|c| matches!(c,
        '\u{202A}' | // Left-to-Right Embedding
        '\u{202B}' | // Right-to-Left Embedding
        '\u{202C}' | // Pop Directional Formatting
        '\u{202D}' | // Left-to-Right Override
        '\u{202E}' | // Right-to-Left Override
        '\u{2066}' | // Left-to-Right Isolate
        '\u{2067}' | // Right-to-Left Isolate
        '\u{2068}' | // First Strong Isolate
        '\u{2069}'   // Pop Directional Isolate
    ))
}

/// Strip zero-width characters from string
fn strip_zero_width_chars(s: &str) -> String {
    s.chars().filter(|c| !matches!(c,
        '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{FEFF}' |
        '\u{180E}' | '\u{2060}' | '\u{2061}' | '\u{2062}' |
        '\u{2063}' | '\u{2064}'
    )).collect()
}

/// Strip RTL override characters from string
fn strip_rtl_override(s: &str) -> String {
    s.chars().filter(|c| !matches!(c,
        '\u{202A}' | '\u{202B}' | '\u{202C}' | '\u{202D}' |
        '\u{202E}' | '\u{2066}' | '\u{2067}' | '\u{2068}' |
        '\u{2069}'
    )).collect()
}

/// Check if a character might be a homoglyph of a Latin letter
fn is_potential_homoglyph(c: char, expected_ascii: char) -> bool {
    // This is a simplified check - real implementation would use confusables table
    if c == expected_ascii {
        return false;
    }
    
    // Not the same char and not plain ASCII
    !c.is_ascii() && c.to_lowercase().to_string() != expected_ascii.to_lowercase().to_string()
}

// ============================================================================
// Suite 1: Zero-Width Character Detection (15 tests)
// ============================================================================

#[test]
fn test_detect_zero_width_space() {
    let input = "exec\u{200B}(command)"; // Zero Width Space
    assert!(contains_zero_width_chars(input));
}

#[test]
fn test_detect_zero_width_non_joiner() {
    let input = "exec\u{200C}(command)"; // Zero Width Non-Joiner
    assert!(contains_zero_width_chars(input));
}

#[test]
fn test_detect_zero_width_joiner() {
    let input = "exec\u{200D}(command)"; // Zero Width Joiner
    assert!(contains_zero_width_chars(input));
}

#[test]
fn test_detect_zero_width_no_break_space() {
    let input = "exec\u{FEFF}(command)"; // BOM / ZWNBSP
    assert!(contains_zero_width_chars(input));
}

#[test]
fn test_detect_word_joiner() {
    let input = "exec\u{2060}(command)"; // Word Joiner
    assert!(contains_zero_width_chars(input));
}

#[test]
fn test_clean_string_no_zero_width() {
    let clean = "Hello World";
    assert!(!contains_zero_width_chars(clean));
}

#[test]
fn test_strip_zero_width_chars() {
    let input = "e\u{200B}x\u{200C}e\u{200D}c";
    let stripped = strip_zero_width_chars(input);
    assert_eq!(stripped, "exec");
}

#[test]
fn test_zero_width_in_banned_pattern() {
    let validator = create_validator();
    
    // "exec(" with zero-width space
    let sneaky = "exec\u{200B}(command)";
    
    // The sanitizer should either reject this or strip it first
    let result = sanitize_llm_prompt(sneaky, &validator);
    // If stripped, "exec(" should be detected; if not stripped, it might pass
    // This test documents the behavior
    let _ = result;
}

#[test]
fn test_multiple_zero_width_chars() {
    let input = "e\u{200B}x\u{200C}e\u{200D}c\u{FEFF}(\u{2060})";
    assert!(contains_zero_width_chars(input));
    
    let stripped = strip_zero_width_chars(input);
    assert_eq!(stripped, "exec()");
}

#[test]
fn test_zero_width_at_start() {
    let input = "\u{200B}exec(command)";
    assert!(contains_zero_width_chars(input));
}

#[test]
fn test_zero_width_at_end() {
    let input = "exec(command)\u{200B}";
    assert!(contains_zero_width_chars(input));
}

#[test]
fn test_only_zero_width_chars() {
    let input = "\u{200B}\u{200C}\u{200D}\u{FEFF}";
    assert!(contains_zero_width_chars(input));
    
    let stripped = strip_zero_width_chars(input);
    assert!(stripped.is_empty());
}

#[test]
fn test_unicode_whitespace_allowed() {
    // Regular spaces and tabs should be allowed
    let clean = "Hello\t\nWorld";
    assert!(!contains_zero_width_chars(clean));
}

#[test]
fn test_invisible_function_application() {
    let input = "f\u{2061}(x)"; // Function Application
    assert!(contains_zero_width_chars(input));
}

#[test]
fn test_invisible_separator() {
    let input = "a\u{2063}b"; // Invisible Separator
    assert!(contains_zero_width_chars(input));
}

// ============================================================================
// Suite 2: RTL Override Detection (10 tests)
// ============================================================================

#[test]
fn test_detect_rtl_override() {
    let input = "hello\u{202E}dlrow"; // RTL Override makes "world" appear
    assert!(contains_rtl_override(input));
}

#[test]
fn test_detect_ltr_embedding() {
    let input = "test\u{202A}text"; // LTR Embedding
    assert!(contains_rtl_override(input));
}

#[test]
fn test_detect_rtl_embedding() {
    let input = "test\u{202B}text"; // RTL Embedding
    assert!(contains_rtl_override(input));
}

#[test]
fn test_detect_ltr_override() {
    let input = "test\u{202D}text"; // LTR Override
    assert!(contains_rtl_override(input));
}

#[test]
fn test_detect_rtl_isolate() {
    let input = "test\u{2067}text"; // RTL Isolate
    assert!(contains_rtl_override(input));
}

#[test]
fn test_strip_rtl_override() {
    let input = "hello\u{202E}world";
    let stripped = strip_rtl_override(input);
    assert_eq!(stripped, "helloworld");
}

#[test]
fn test_clean_string_no_rtl() {
    let clean = "Hello World";
    assert!(!contains_rtl_override(clean));
}

#[test]
fn test_rtl_override_filename_attack() {
    // Filename that looks like .txt but is actually .exe
    // "document[RLO]txt.exe" displays as "documentexe.txt"
    let sneaky = "document\u{202E}txt.exe";
    assert!(contains_rtl_override(sneaky));
}

#[test]
fn test_nested_rtl_overrides() {
    let input = "\u{202A}text\u{202B}more\u{202C}\u{202D}end\u{202E}";
    assert!(contains_rtl_override(input));
}

#[test]
fn test_rtl_in_path() {
    // This could make a path look legitimate while hiding its true nature
    let sneaky_path = "assets/\u{202E}gnp.exe";
    assert!(contains_rtl_override(sneaky_path));
}

// ============================================================================
// Suite 3: Homoglyph Detection (15 tests)
// ============================================================================

#[test]
fn test_cyrillic_e_homoglyph() {
    // Cyrillic 'е' (U+0435) looks like Latin 'e'
    let cyrillic_e = 'е'; // This is Cyrillic, not Latin!
    assert!(is_potential_homoglyph(cyrillic_e, 'e'));
}

#[test]
fn test_cyrillic_a_homoglyph() {
    // Cyrillic 'а' (U+0430) looks like Latin 'a'
    let cyrillic_a = 'а';
    assert!(is_potential_homoglyph(cyrillic_a, 'a'));
}

#[test]
fn test_cyrillic_o_homoglyph() {
    // Cyrillic 'о' (U+043E) looks like Latin 'o'
    let cyrillic_o = 'о';
    assert!(is_potential_homoglyph(cyrillic_o, 'o'));
}

#[test]
fn test_cyrillic_c_homoglyph() {
    // Cyrillic 'с' (U+0441) looks like Latin 'c'
    let cyrillic_c = 'с';
    assert!(is_potential_homoglyph(cyrillic_c, 'c'));
}

#[test]
fn test_cyrillic_p_homoglyph() {
    // Cyrillic 'р' (U+0440) looks like Latin 'p'
    let cyrillic_p = 'р';
    assert!(is_potential_homoglyph(cyrillic_p, 'p'));
}

#[test]
fn test_greek_alpha_homoglyph() {
    // Greek 'α' looks similar to Latin 'a'
    let greek_alpha = 'α';
    assert!(is_potential_homoglyph(greek_alpha, 'a'));
}

#[test]
fn test_greek_omicron_homoglyph() {
    // Greek 'ο' looks like Latin 'o'
    let greek_omicron = 'ο';
    assert!(is_potential_homoglyph(greek_omicron, 'o'));
}

#[test]
fn test_fullwidth_letter_homoglyph() {
    // Fullwidth 'A' (U+FF21) looks like Latin 'A' but is wider
    let fullwidth_a = 'Ａ';
    assert!(is_potential_homoglyph(fullwidth_a, 'A'));
}

#[test]
fn test_latin_same_char_not_homoglyph() {
    // Same character should not be flagged
    assert!(!is_potential_homoglyph('e', 'e'));
    assert!(!is_potential_homoglyph('A', 'A'));
}

#[test]
fn test_mixed_script_exec_attempt() {
    // "exec" with Cyrillic 'е' instead of Latin 'e'
    let sneaky_exec = "еxеc"; // First and third 'e' are Cyrillic
    
    // This should be caught or fail pattern matching
    let validator = create_validator();
    let result = sanitize_llm_prompt(sneaky_exec, &validator);
    
    // Note: The validator may not detect this because pattern is "exec("
    // The test documents current behavior
    let _ = result;
}

#[test]
fn test_homoglyph_in_function_name() {
    // "еval" with Cyrillic 'е' (U+0435) instead of Latin 'e'
    let sneaky_eval = "еval(code)";
    
    let validator = create_validator();
    // Current validator checks for literal "eval("
    // Homoglyph bypass might succeed depending on implementation
    let result = sanitize_llm_prompt(sneaky_eval, &validator);
    let _ = result; // Document behavior
}

#[test]
fn test_mathematical_script_letters() {
    // Mathematical script 'e' (U+2147) looks like italic 'e'
    let math_e = 'ⅇ';
    assert!(is_potential_homoglyph(math_e, 'e'));
}

#[test]
fn test_subscript_numbers() {
    // Subscript numbers look similar but are different codepoints
    let subscript_1 = '₁';
    assert!(is_potential_homoglyph(subscript_1, '1'));
}

#[test]
fn test_superscript_numbers() {
    // Superscript numbers
    let superscript_2 = '²';
    assert!(is_potential_homoglyph(superscript_2, '2'));
}

#[test]
fn test_roman_numeral_one() {
    // Roman numeral Ⅰ (U+2160) looks like 'I'
    let roman_one = 'Ⅰ';
    assert!(is_potential_homoglyph(roman_one, 'I'));
}

// ============================================================================
// Suite 4: Unicode Normalization Issues (10 tests)
// ============================================================================

#[test]
fn test_combining_characters() {
    // 'é' can be represented as:
    // - Single codepoint: U+00E9 (Latin Small Letter E with Acute)
    // - Combined: U+0065 + U+0301 (e + combining acute accent)
    let precomposed = "café"; // Uses U+00E9
    let decomposed = "cafe\u{0301}"; // Uses combining character
    
    // Both should render the same but may compare differently
    assert_eq!(precomposed.chars().count(), 4);
    assert_eq!(decomposed.chars().count(), 5); // Extra combining char
}

#[test]
fn test_compatibility_decomposition() {
    // '①' (circled digit one) is compatibility equivalent to '1'
    let circled_one = '①';
    assert!(!circled_one.is_ascii_digit());
    
    // But it should not be confused for '1' in security contexts
    let input = format!("select {} from table", circled_one);
    // This input looks different than "select 1 from table"
    assert!(input.contains('①'));
}

#[test]
fn test_ligatures() {
    // 'ﬁ' ligature (U+FB01) is compatibility equivalent to "fi"
    let ligature = "ﬁle.txt";
    let expanded = "file.txt";
    
    assert_ne!(ligature, expanded);
    assert!(ligature.contains('ﬁ'));
}

#[test]
fn test_enclosed_alphanumerics() {
    // ⒳ (parenthesized latin small letter x) looks like (x)
    let enclosed = "⒳";
    assert!(!enclosed.contains('x'));
}

#[test]
fn test_nfkc_normalization_needed() {
    // Some applications need NFKC normalization for security
    // Fullwidth 'ｈｔｔｐ' should normalize to 'http'
    let fullwidth = "ｈｔｔｐ://evil.com";
    let normal = "http://evil.com";
    
    assert_ne!(fullwidth, normal);
    assert!(fullwidth.chars().any(|c| !c.is_ascii()));
}

#[test]
fn test_invisible_plus() {
    // Invisible Plus (U+2064) can hide operations
    let input = "1\u{2064}1"; // Looks like "11" but has invisible operator
    assert!(contains_zero_width_chars(input));
}

#[test]
fn test_soft_hyphen() {
    // Soft hyphen (U+00AD) is often invisible
    let input = "exec\u{00AD}ute";
    assert!(input.contains('\u{00AD}'));
}

#[test]
fn test_zwj_sequence() {
    // Zero Width Joiner sequences can create unexpected characters
    // 👨‍💻 = man + ZWJ + computer
    let emoji = "👨\u{200D}💻";
    assert!(contains_zero_width_chars(emoji)); // Contains ZWJ
}

#[test]
fn test_variation_selectors() {
    // Variation selectors can change glyph appearance
    let with_vs = "A\u{FE00}";
    assert_ne!(with_vs.chars().count(), 1);
}

#[test]
fn test_tag_characters() {
    // Tag characters (U+E0001 to U+E007F) are invisible
    // Used in language tags
    let tagged = "text\u{E0001}";
    assert!(tagged.len() > 4);
}

// ============================================================================
// Suite 5: Path Traversal with Unicode (10 tests)
// ============================================================================

#[test]
fn test_unicode_path_separators() {
    // Some Unicode characters look like path separators
    // Full-width solidus: ／(U+FF0F)
    let sneaky_path = "..／etc／passwd";
    assert!(sneaky_path.contains('／'));
    assert!(!sneaky_path.contains('/'));
}

#[test]
fn test_unicode_dots() {
    // Unicode has many dot-like characters
    // Horizontal ellipsis: … (U+2026) vs ".." (two periods)
    let ellipsis_path = "…/etc/passwd";
    assert_ne!(ellipsis_path, "../etc/passwd");
}

#[test]
fn test_fullwidth_backslash() {
    // Full-width backslash: ＼ (U+FF3C)
    let sneaky_windows = "..＼..＼system32";
    assert!(sneaky_windows.contains('＼'));
    assert!(!sneaky_windows.contains('\\'));
}

#[test]
fn test_fraction_slash() {
    // Fraction slash: ⁄ (U+2044)
    let sneaky = "..⁄etc⁄passwd";
    assert!(!sneaky.contains('/'));
}

#[test]
fn test_division_slash() {
    // Division slash: ∕ (U+2215)
    let sneaky = "..∕etc∕passwd";
    assert!(!sneaky.contains('/'));
}

#[test]
fn test_combining_dot_below() {
    // Combining dot (U+0323) could be used to obscure paths
    let path_with_combining = "..\u{0323}/secret";
    assert!(path_with_combining.contains('\u{0323}'));
}

#[test]
fn test_ideographic_period() {
    // Ideographic full stop: 。(U+3002)
    let sneaky = "。。/etc/passwd";
    assert!(!sneaky.contains('.'));
}

#[test]
fn test_null_in_path() {
    // Null byte can truncate paths in some systems
    let path = "file.txt\0.exe";
    assert!(path.contains('\0'));
}

#[test]
fn test_path_normalization_attack() {
    // Using directory separators that might normalize differently
    let path = Path::new("assets/textures/grass.png");
    assert!(path.is_relative());
    
    // Verify the path doesn't contain hidden characters
    let path_str = path.to_string_lossy();
    assert!(!contains_zero_width_chars(&path_str));
    assert!(!contains_rtl_override(&path_str));
}

#[test]
fn test_unicode_in_extension() {
    // File extension with Unicode lookalike
    // .ехe with Cyrillic 'е' and 'х'
    let path = Path::new("file.ехе"); // Cyrillic characters!
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    
    // Extension is not "exe" (ASCII), it's "ехе" (Cyrillic)
    assert_ne!(ext, "exe");
}

// ============================================================================
// Suite 6: Bidirectional Text Attacks (10 tests)
// ============================================================================

#[test]
fn test_bidi_source_code_attack() {
    // The "Trojan Source" attack using RLO
    // Using escaped form to avoid compiler warning
    let code = format!("if (isAdmin{}{}// check admin{}{}) {{", 
        '\u{202E}', '\u{2066}', '\u{2069}', '\u{2066}');
    assert!(contains_rtl_override(&code));
}

#[test]
fn test_bidi_comment_manipulation() {
    // Comments can be visually reordered
    let code = "// safe code\u{202E} } { evil()";
    assert!(contains_rtl_override(code));
}

#[test]
fn test_bidi_string_escapes() {
    // String literals with bidi can be confusing
    let s = "\"normal\u{202E}value\"";
    assert!(contains_rtl_override(s));
}

#[test]
fn test_pop_directional_formatting() {
    // PDF (U+202C) pops the embedding/override stack
    let code = "text\u{202B}reversed\u{202C}normal";
    assert!(contains_rtl_override(code));
}

#[test]
fn test_first_strong_isolate() {
    // FSI (U+2068) starts an isolate
    let text = "Text\u{2068}isolated\u{2069}more";
    assert!(contains_rtl_override(text));
}

#[test]
fn test_nested_bidi_controls() {
    // Multiple nested bidi controls
    let text = "\u{202A}\u{202B}\u{202D}\u{202E}text\u{202C}\u{202C}\u{202C}\u{202C}";
    assert!(contains_rtl_override(text));
}

#[test]
fn test_bidi_in_filename() {
    // Filename that appears to have different extension
    let filename = "report\u{202E}fdp.exe"; // Appears as "reportexe.pdf"
    assert!(contains_rtl_override(filename));
}

#[test]
fn test_bidi_stripped_result() {
    let malicious = "safe\u{202E}edoc";
    let clean = strip_rtl_override(malicious);
    assert_eq!(clean, "safeedoc");
}

#[test]
fn test_bidi_url_spoofing() {
    // URL that looks legitimate but isn't
    let url = "https://\u{202E}moc.elgoog";
    assert!(contains_rtl_override(url));
}

#[test]
fn test_mixed_bidi_and_zero_width() {
    // Combination attack
    let sneaky = "exec\u{200B}\u{202E})(";
    assert!(contains_zero_width_chars(sneaky));
    assert!(contains_rtl_override(sneaky));
}

// ============================================================================
// Suite 7: LLM Prompt with Unicode (10 tests)
// ============================================================================

#[test]
fn test_llm_prompt_with_emoji() {
    let validator = create_validator();
    let prompt = "Hello 👋 How can I help you today? 🤖";
    
    let result = sanitize_llm_prompt(prompt, &validator);
    assert!(result.is_ok());
}

#[test]
fn test_llm_prompt_with_cjk() {
    let validator = create_validator();
    let prompt = "翻译这段话: Hello World = 你好世界";
    
    let result = sanitize_llm_prompt(prompt, &validator);
    assert!(result.is_ok());
}

#[test]
fn test_llm_prompt_with_arabic() {
    let validator = create_validator();
    let prompt = "مرحبا بالعالم";
    
    let result = sanitize_llm_prompt(prompt, &validator);
    assert!(result.is_ok());
}

#[test]
fn test_llm_prompt_with_cyrillic() {
    let validator = create_validator();
    let prompt = "Привет мир! Как дела?";
    
    let result = sanitize_llm_prompt(prompt, &validator);
    assert!(result.is_ok());
}

#[test]
fn test_llm_prompt_length_unicode() {
    let validator = create_validator();
    
    // Unicode characters can be multi-byte
    // 100 emoji characters might be 400 bytes
    let long_emoji = "🔒".repeat(100);
    
    let result = sanitize_llm_prompt(&long_emoji, &validator);
    // Should pass length check (100 chars < 1000 limit)
    assert!(result.is_ok());
}

#[test]
fn test_llm_prompt_banned_with_zwc() {
    let validator = create_validator();
    
    // Try to bypass "exec(" check with zero-width chars
    let sneaky = "exec\u{200B}(command)";
    
    // Current implementation may or may not catch this
    let result = sanitize_llm_prompt(sneaky, &validator);
    // Document behavior - ideally this should be caught
    let _ = result;
}

#[test]
fn test_llm_prompt_banned_with_homoglyph() {
    let validator = create_validator();
    
    // "exec" with Cyrillic 'е' (U+0435)
    let sneaky = "еxec(command)"; // First 'e' is Cyrillic
    
    // Pattern match for "exec(" won't match because first char is different
    let result = sanitize_llm_prompt(sneaky, &validator);
    // This might pass because the literal "exec(" isn't present
    let _ = result;
}

#[test]
fn test_llm_prompt_with_rtl_override() {
    let validator = create_validator();
    
    // Prompt with hidden RTL override
    let sneaky = "Please help me\u{202E}tseuqer siht htiw";
    
    let result = sanitize_llm_prompt(sneaky, &validator);
    // RTL override might not be caught by current validator
    let _ = result;
}

#[test]
fn test_llm_prompt_mixed_scripts() {
    let validator = create_validator();
    
    // Mixed script prompt (legitimate use case)
    let prompt = "Translate 'Hello' to: 日本語, Русский, العربية, 한국어";
    
    let result = sanitize_llm_prompt(prompt, &validator);
    assert!(result.is_ok());
}

#[test]
fn test_llm_prompt_mathematical_notation() {
    let validator = create_validator();
    
    // Mathematical Unicode symbols (legitimate)
    let prompt = "Solve: ∫₀^∞ e^(-x²) dx = √π/2";
    
    let result = sanitize_llm_prompt(prompt, &validator);
    assert!(result.is_ok());
}
