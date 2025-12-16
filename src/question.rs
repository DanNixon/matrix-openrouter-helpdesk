use matrix_sdk::ruma::{events::Mentions, UserId};
use regex::Regex;

pub(super) fn extract(
    message_body: &str,
    mentions: Option<Mentions>,
    bot_id: &UserId,
    bot_display_name: Option<String>,
) -> Option<String> {
    let mention_username = format!("@{}", bot_id.localpart());

    if message_body.contains(&mention_username) {
        let re = Regex::new(&format!(
            "\\[{mention_username}:.*\\]\\(.*\\)\\s*:\\s*(.+)|\\[{mention_username}:.*\\]\\(.*\\)\\s+(.+)"
        ))
        .unwrap();

        if let Some(q) = re
            .captures(message_body)
            .map(|c| c.get(1).or(c.get(2)).unwrap().as_str().to_string())
        {
            return Some(q);
        }

        let re = Regex::new(&format!(
            "{mention_username}\\s*:\\s*(.+)|{mention_username}\\s+(.+)"
        ))
        .unwrap();

        if let Some(q) = re
            .captures(message_body)
            .map(|c| c.get(1).or(c.get(2)).unwrap().as_str().to_string())
        {
            return Some(q);
        }
    }

    if let Some(mentions) = mentions {
        if mentions.user_ids.contains(bot_id) {
            let bot_display_name = bot_display_name.unwrap_or(mention_username);

            let re = Regex::new(&format!(
                "{bot_display_name}\\s*:\\s*(.+)|{bot_display_name}\\s+(.+)"
            ))
            .unwrap();

            if let Some(q) = re
                .captures(message_body)
                .map(|c| c.get(1).or(c.get(2)).unwrap().as_str().to_string())
            {
                return Some(q);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_question_with_mention_username_colon() {
        let bot_id = UserId::parse("@test:example.org").unwrap();
        let message = "@test: What is the answer?";
        let result = extract(message, None, &bot_id, None).unwrap();
        assert_eq!(result, "What is the answer?");
    }

    #[test]
    fn test_extract_question_with_mention_username_space_colon() {
        let bot_id = UserId::parse("@test:example.org").unwrap();
        let message = "@test : What is the answer?";
        let result = extract(message, None, &bot_id, None).unwrap();
        assert_eq!(result, "What is the answer?");
    }

    #[test]
    fn test_extract_question_with_mention_username_space() {
        let bot_id = UserId::parse("@test:example.org").unwrap();
        let message = "@test What is the answer?";
        let result = extract(message, None, &bot_id, None).unwrap();
        assert_eq!(result, "What is the answer?");
    }

    #[test]
    fn test_extract_question_with_markdown_mention_username_colon() {
        let bot_id = UserId::parse("@test:example.org").unwrap();
        let message =
            "[@test:example.org](https://matrix.to/#/@test:example.org): What is the answer?";
        let result = extract(message, None, &bot_id, None).unwrap();
        assert_eq!(result, "What is the answer?");
    }

    #[test]
    fn test_extract_question_with_markdown_mention_username_space_colon() {
        let bot_id = UserId::parse("@test:example.org").unwrap();
        let message =
            "[@test:example.org](https://matrix.to/#/@test:example.org) : What is the answer?";
        let result = extract(message, None, &bot_id, None).unwrap();
        assert_eq!(result, "What is the answer?");
    }

    #[test]
    fn test_extract_question_with_markdown_mention_username_space() {
        let bot_id = UserId::parse("@test:example.org").unwrap();
        let message =
            "[@test:example.org](https://matrix.to/#/@test:example.org) What is the answer?";
        let result = extract(message, None, &bot_id, None).unwrap();
        assert_eq!(result, "What is the answer?");
    }

    #[test]
    fn test_extract_question_with_display_name_colon() {
        let bot_id = UserId::parse("@test:example.org").unwrap();
        let mut mentions = Mentions::new();
        mentions.user_ids.insert(bot_id.clone());
        let message = "Testy McTestface: How are you?";
        let result = extract(
            message,
            Some(mentions),
            &bot_id,
            Some("Testy McTestface".to_string()),
        )
        .unwrap();
        assert_eq!(result, "How are you?");
    }

    #[test]
    fn test_extract_question_with_display_name_space_colon() {
        let bot_id = UserId::parse("@test:example.org").unwrap();
        let mut mentions = Mentions::new();
        mentions.user_ids.insert(bot_id.clone());
        let message = "Testy McTestface : How are you?";
        let result = extract(
            message,
            Some(mentions),
            &bot_id,
            Some("Testy McTestface".to_string()),
        )
        .unwrap();
        assert_eq!(result, "How are you?");
    }

    #[test]
    fn test_extract_question_with_display_name_space() {
        let bot_id = UserId::parse("@test:example.org").unwrap();
        let mut mentions = Mentions::new();
        mentions.user_ids.insert(bot_id.clone());
        let message = "Testy McTestface How are you?";
        let result = extract(
            message,
            Some(mentions),
            &bot_id,
            Some("Testy McTestface".to_string()),
        )
        .unwrap();
        assert_eq!(result, "How are you?");
    }

    #[test]
    fn test_extract_question_no_match() {
        let bot_id = UserId::parse("@test:example.org").unwrap();
        let message = "Hello world";
        let result = extract(message, None, &bot_id, None);
        assert!(result.is_none());
    }
}
