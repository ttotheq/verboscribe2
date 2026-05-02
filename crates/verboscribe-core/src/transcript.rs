use regex::{Captures, Regex};
use std::collections::HashSet;

use crate::{ProcessedTranscript, TargetApp, TranscriptProcessor};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanupLevel {
    None,
    Light,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StylePreset {
    Automatic,
    General,
    TerminalCode,
    MessagesChat,
    Email,
    NotesDocs,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranscriptProcessingOptions {
    pub cleanup_level: CleanupLevel,
    pub command_mode_enabled: bool,
    pub snippets: String,
    pub style_preset: StylePreset,
}

impl Default for TranscriptProcessingOptions {
    fn default() -> Self {
        Self {
            cleanup_level: CleanupLevel::None,
            command_mode_enabled: false,
            snippets: String::new(),
            style_preset: StylePreset::Automatic,
        }
    }
}

pub struct DefaultTranscriptProcessor {
    options: TranscriptProcessingOptions,
}

impl DefaultTranscriptProcessor {
    pub fn new(options: TranscriptProcessingOptions) -> Self {
        Self { options }
    }
}

impl TranscriptProcessor for DefaultTranscriptProcessor {
    fn process(&self, raw: &str, target: Option<&TargetApp>) -> ProcessedTranscript {
        process_transcript(raw, target, &self.options)
    }
}

pub fn process_transcript(
    transcript: &str,
    target: Option<&TargetApp>,
    options: &TranscriptProcessingOptions,
) -> ProcessedTranscript {
    let raw = normalize_whitespace(transcript.trim());
    let mut text = raw.clone();

    if options.command_mode_enabled {
        text = apply_command_mode(&text);
    }

    text = cleanup(&text, options.cleanup_level);
    text = SnippetExpander::expand(&text, &options.snippets);
    text = apply_style_preset(
        effective_style_preset(options.style_preset, target),
        &text,
        options.cleanup_level,
    );

    ProcessedTranscript {
        raw,
        inserted: text.trim().to_string(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnippetRule {
    pub trigger: String,
    pub expansion: String,
}

pub struct SnippetExpander;

impl SnippetExpander {
    pub fn rules(text: &str) -> Vec<SnippetRule> {
        text.split(['\n', ';'])
            .filter_map(|line| {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    return None;
                }

                let (trigger, expansion) =
                    if let Some((trigger, expansion)) = trimmed.split_once("=>") {
                        (trigger.trim(), expansion.trim())
                    } else if let Some((trigger, expansion)) = trimmed.split_once('\t') {
                        (trigger.trim(), expansion.trim())
                    } else {
                        return None;
                    };

                if trigger.is_empty() || expansion.is_empty() {
                    None
                } else {
                    Some(SnippetRule {
                        trigger: trigger.to_string(),
                        expansion: expansion.to_string(),
                    })
                }
            })
            .collect()
    }

    pub fn expand(text: &str, rules_text: &str) -> String {
        Self::expand_rules(text, &Self::rules(rules_text))
    }

    pub fn expand_rules(text: &str, rules: &[SnippetRule]) -> String {
        rules.iter().fold(text.to_string(), |current, rule| {
            expand_rule(&current, rule)
        })
    }
}

pub struct PersonalDictionary;

impl PersonalDictionary {
    pub fn terms(text: &str) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut terms = Vec::new();

        for raw in text.split([',', '\n']) {
            let term = raw.trim();
            if term.is_empty() {
                continue;
            }

            let key = term.to_lowercase();
            if seen.insert(key) {
                terms.push(term.to_string());
            }
        }

        terms
    }

    pub fn combined_prompt(context: &str, dictionary_text: &str) -> String {
        let trimmed_context = context.trim();
        let terms = Self::terms(dictionary_text);
        if terms.is_empty() {
            return trimmed_context.to_string();
        }

        let dictionary_prompt = format!(
            "Prefer these exact names, terms, acronyms, and pinned words when heard: {}.",
            terms.join(", ")
        );

        if trimmed_context.is_empty() {
            dictionary_prompt
        } else {
            format!("{trimmed_context}\n\n{dictionary_prompt}")
        }
    }
}

fn apply_command_mode(text: &str) -> String {
    let mut result = text.to_string();
    for (phrase, replacement) in [
        ("new paragraph", "\n\n"),
        ("new line", "\n"),
        ("press enter", "\n"),
        ("comma", ","),
        ("period", "."),
        ("full stop", "."),
        ("question mark", "?"),
        ("exclamation mark", "!"),
        ("colon", ":"),
        ("semicolon", ";"),
        ("open quote", "\""),
        ("close quote", "\""),
    ] {
        result = replace_standalone_phrase(&result, phrase, replacement);
    }

    result = remove_scratch_that_segment(&result);
    normalize_spacing_around_punctuation(&result)
}

fn cleanup(text: &str, level: CleanupLevel) -> String {
    match level {
        CleanupLevel::None => text.to_string(),
        CleanupLevel::Light => light_cleanup(text),
        CleanupLevel::Medium => medium_cleanup(text),
        CleanupLevel::High => high_cleanup(text),
    }
}

fn apply_style_preset(preset: StylePreset, text: &str, cleanup_level: CleanupLevel) -> String {
    if cleanup_level == CleanupLevel::None {
        return text.to_string();
    }

    match preset {
        StylePreset::Automatic | StylePreset::General => text.to_string(),
        StylePreset::TerminalCode => remove_short_trailing_period(text, 80),
        StylePreset::MessagesChat => remove_short_trailing_period(text, 120),
        StylePreset::Email => email_style(text),
        StylePreset::NotesDocs => notes_docs_style(text),
    }
}

fn effective_style_preset(preset: StylePreset, target: Option<&TargetApp>) -> StylePreset {
    if preset != StylePreset::Automatic {
        return preset;
    }

    let target_text = target
        .map(|target| {
            format!(
                "{} {}",
                target.identifier.as_deref().unwrap_or_default(),
                target.name.as_deref().unwrap_or_default()
            )
            .to_lowercase()
        })
        .unwrap_or_default();

    if [
        "terminal",
        "iterm",
        "warp",
        "xcode",
        "cursor",
        "visualstudio",
        "code",
    ]
    .iter()
    .any(|needle| target_text.contains(needle))
    {
        return StylePreset::TerminalCode;
    }

    if ["messages", "slack", "discord", "teams"]
        .iter()
        .any(|needle| target_text.contains(needle))
    {
        return StylePreset::MessagesChat;
    }

    if ["mail", "outlook", "superhuman"]
        .iter()
        .any(|needle| target_text.contains(needle))
    {
        return StylePreset::Email;
    }

    if [
        "notes", "textedit", "pages", "word", "obsidian", "notion", "bear",
    ]
    .iter()
    .any(|needle| target_text.contains(needle))
    {
        return StylePreset::NotesDocs;
    }

    StylePreset::General
}

fn light_cleanup(text: &str) -> String {
    let mut result = normalize_whitespace_preserving_line_breaks(text);
    result = remove_fillers(&result, &["um", "uh", "er", "ah"]);
    result = normalize_spacing_around_punctuation(&result);
    result = capitalize_sentence_starts(&result);
    normalize_standalone_i(&result)
}

fn medium_cleanup(text: &str) -> String {
    let mut result = light_cleanup(text);
    result = remove_fillers(
        &result,
        &["like", "you know", "I mean", "sort of", "kind of"],
    );
    result = remove_immediate_repeated_words(&result);
    capitalize_sentence_starts(&result)
}

fn high_cleanup(text: &str) -> String {
    let mut result = medium_cleanup(text);
    result = remove_leading_phrases(
        &result,
        &["so", "okay", "alright", "basically", "actually", "just"],
    );
    result = replace_regex(&result, r"(?i)\bI think that\b", "I think");
    result = replace_regex(&result, r"(?i)\bin order to\b", "to");
    capitalize_sentence_starts(&result)
}

fn remove_short_trailing_period(text: &str, max_len: usize) -> String {
    let mut result = text.to_string();
    if !result.contains('\n') && result.len() < max_len && result.ends_with('.') {
        result.pop();
    }
    result
}

fn email_style(text: &str) -> String {
    let result = capitalize_sentence_starts(text);
    match result.chars().last() {
        Some(last) if last.is_alphanumeric() => format!("{result}."),
        _ => result,
    }
}

fn notes_docs_style(text: &str) -> String {
    let result = normalize_whitespace_preserving_line_breaks(text);
    let result = replace_regex(&result, r"(?m)^[-*]\s+", "- ");
    capitalize_sentence_starts(&result)
}

fn expand_rule(text: &str, rule: &SnippetRule) -> String {
    let inner_pattern = standalone_pattern(&rule.trigger);
    if inner_pattern.is_empty() {
        return text.to_string();
    }

    let pattern = boundary_pattern(&inner_pattern, true);
    Regex::new(&pattern)
        .expect("snippet pattern should compile")
        .replace_all(text, |captures: &Captures<'_>| {
            let prefix = captures
                .get(1)
                .map(|value| value.as_str())
                .unwrap_or_default();
            let suffix = captures
                .get(3)
                .map(|value| value.as_str())
                .unwrap_or_default();
            format!("{prefix}{}{suffix}", rule.expansion)
        })
        .to_string()
}

fn standalone_pattern(trigger: &str) -> String {
    let parts: Vec<String> = trigger
        .split(|character: char| character.is_whitespace() || character == '-' || character == '_')
        .filter(|part| !part.is_empty())
        .map(regex::escape)
        .collect();

    if parts.is_empty() {
        String::new()
    } else {
        format!(r"(?i){}", parts.join(r"(?:[\s\-_]+)"))
    }
}

fn replace_standalone_phrase(text: &str, phrase: &str, replacement: &str) -> String {
    let pattern = boundary_pattern(&format!(r"(?i){}", regex::escape(phrase)), false);
    Regex::new(&pattern)
        .expect("standalone phrase pattern should compile")
        .replace_all(text, |captures: &Captures<'_>| {
            let prefix = captures
                .get(1)
                .map(|value| value.as_str())
                .unwrap_or_default();
            let suffix = captures
                .get(3)
                .map(|value| value.as_str())
                .unwrap_or_default();
            format!("{prefix}{replacement}{suffix}")
        })
        .to_string()
}

fn boundary_pattern(inner_pattern: &str, include_digits: bool) -> String {
    let boundary_class = if include_digits {
        "A-Za-z0-9"
    } else {
        "A-Za-z"
    };
    format!(r"(^|[^{boundary_class}])({inner_pattern})([^{boundary_class}]|$)")
}

fn remove_scratch_that_segment(text: &str) -> String {
    replace_regex(
        text,
        r"(?i)(^|[.!?\n]\s*)[^.!?\n]*\bscratch that\b\s*",
        "$1",
    )
}

fn remove_fillers(text: &str, fillers: &[&str]) -> String {
    let mut result = text.to_string();
    for filler in fillers {
        let pattern = format!(r"(?i)(^|[\s,]){}([,\s]|$)", regex::escape(filler));
        result = replace_regex(&result, &pattern, "$1");
    }
    normalize_whitespace_preserving_line_breaks(&result)
}

fn remove_leading_phrases(text: &str, phrases: &[&str]) -> String {
    phrases.iter().fold(text.to_string(), |current, phrase| {
        let pattern = format!(r"(?i)^\s*{}\b[, ]*", regex::escape(phrase));
        replace_regex(&current, &pattern, "")
    })
}

fn remove_immediate_repeated_words(text: &str) -> String {
    let re =
        Regex::new(r"(?i)\b([A-Za-z']+)\s+([A-Za-z']+)\b").expect("repeat pattern should compile");
    let mut result = text.to_string();
    loop {
        let next = re
            .replace_all(&result, |captures: &Captures<'_>| {
                if captures[1].eq_ignore_ascii_case(&captures[2]) {
                    captures[1].to_string()
                } else {
                    captures[0].to_string()
                }
            })
            .to_string();
        if next == result {
            return result;
        }
        result = next;
    }
}

fn normalize_whitespace(text: &str) -> String {
    replace_regex(text, r"\s+", " ")
}

fn normalize_whitespace_preserving_line_breaks(text: &str) -> String {
    let result = replace_regex(text, r"[ \t]+", " ");
    let result = replace_regex(&result, r" *\n *", "\n");
    replace_regex(&result, r"\n{3,}", "\n\n").trim().to_string()
}

fn normalize_spacing_around_punctuation(text: &str) -> String {
    let result = replace_regex(text, r"\s+([,.:;!?])", "$1");
    let result = replace_regex(&result, r#"([,.:;!?])([^\s\n"'])"#, "$1 $2");
    let result = replace_regex(&result, r#"\s+""#, " \"");
    normalize_whitespace_preserving_line_breaks(&result)
}

fn capitalize_sentence_starts(text: &str) -> String {
    let mut result = String::new();
    let mut should_capitalize = true;

    for character in text.chars() {
        if should_capitalize && character.is_alphabetic() {
            result.push_str(&character.to_uppercase().to_string());
            should_capitalize = false;
        } else {
            result.push(character);
        }

        if ".!?\n".contains(character) {
            should_capitalize = true;
        } else if !character.is_whitespace() {
            should_capitalize = false;
        }
    }

    result
}

fn normalize_standalone_i(text: &str) -> String {
    replace_regex(text, r"(?i)\bi\b", "I")
}

fn replace_regex(text: &str, pattern: &str, replacement: &str) -> String {
    Regex::new(pattern)
        .expect("regex should compile")
        .replace_all(text, replacement)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snippet_expander_parses_supported_rule_formats() {
        let rules = SnippetExpander::rules("brb => be right back; omw\tOn my way\nbad rule");

        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].trigger, "brb");
        assert_eq!(rules[0].expansion, "be right back");
        assert_eq!(rules[1].trigger, "omw");
        assert_eq!(rules[1].expansion, "On my way");
    }

    #[test]
    fn snippet_expander_only_expands_standalone_triggers() {
        let expanded = SnippetExpander::expand(
            "brb, this should not change xbrb or brbcode.",
            "brb => be right back",
        );

        assert_eq!(
            expanded,
            "be right back, this should not change xbrb or brbcode."
        );
    }

    #[test]
    fn snippet_expander_treats_hyphen_underscore_and_spaces_as_trigger_separators() {
        let expanded = SnippetExpander::expand(
            "please use bug fix and bug_fix today",
            "bug-fix => bug fix branch",
        );

        assert_eq!(
            expanded,
            "please use bug fix branch and bug fix branch today"
        );
    }

    #[test]
    fn personal_dictionary_trims_splits_and_deduplicates_terms() {
        let terms = PersonalDictionary::terms("OpenAI,  Codex\nopenai\nWhisper");

        assert_eq!(terms, vec!["OpenAI", "Codex", "Whisper"]);
    }

    #[test]
    fn personal_dictionary_appends_dictionary_hints_to_context() {
        let prompt = PersonalDictionary::combined_prompt(
            "Use concise technical language.",
            "Codex, whisper.cpp",
        );

        assert_eq!(
            prompt,
            "Use concise technical language.\n\nPrefer these exact names, terms, acronyms, and pinned words when heard: Codex, whisper.cpp."
        );
    }

    #[test]
    fn raw_first_default_preserves_normalized_raw_text() {
        let processed = process_transcript(
            "  hello   world  ",
            None,
            &TranscriptProcessingOptions::default(),
        );

        assert_eq!(processed.raw, "hello world");
        assert_eq!(processed.inserted, "hello world");
        assert!(!processed.did_change());
    }

    #[test]
    fn command_mode_applies_punctuation_and_line_breaks() {
        let processed = process_transcript(
            "hello comma new line world question mark",
            None,
            &TranscriptProcessingOptions {
                command_mode_enabled: true,
                ..TranscriptProcessingOptions::default()
            },
        );

        assert_eq!(processed.inserted, "hello,\nworld?");
    }

    #[test]
    fn medium_cleanup_removes_fillers_and_repeated_words() {
        let processed = process_transcript(
            "um i mean hello hello world",
            None,
            &TranscriptProcessingOptions {
                cleanup_level: CleanupLevel::Medium,
                style_preset: StylePreset::General,
                ..TranscriptProcessingOptions::default()
            },
        );

        assert_eq!(processed.inserted, "Hello world");
    }

    #[test]
    fn automatic_terminal_style_removes_short_trailing_period() {
        let target = TargetApp {
            name: Some("Terminal".to_string()),
            identifier: Some("com.apple.Terminal".to_string()),
        };
        let processed = process_transcript(
            "run tests.",
            Some(&target),
            &TranscriptProcessingOptions {
                cleanup_level: CleanupLevel::Light,
                style_preset: StylePreset::Automatic,
                ..TranscriptProcessingOptions::default()
            },
        );

        assert_eq!(processed.inserted, "Run tests");
    }
}
