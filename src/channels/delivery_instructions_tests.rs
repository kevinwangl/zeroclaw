#[cfg(test)]
mod channel_delivery_instructions_tests {
    use crate::channels::channel_delivery_instructions;

    #[test]
    fn telegram_has_specific_instructions() {
        let instructions = channel_delivery_instructions("telegram");
        assert!(instructions.is_some());
        let text = instructions.unwrap();
        assert!(text.contains("Telegram"));
        assert!(text.contains("[IMAGE:<path-or-url>]"));
        assert!(text.contains("**bold**"));
    }

    #[test]
    fn discord_has_default_instructions() {
        let instructions = channel_delivery_instructions("discord");
        assert!(instructions.is_some());
        let text = instructions.unwrap();
        assert!(text.contains("[IMAGE:<path-or-url>]"));
        assert!(text.contains("Be concise and direct"));
        assert!(text.contains("Use tool results silently"));
    }

    #[test]
    fn slack_has_default_instructions() {
        let instructions = channel_delivery_instructions("slack");
        assert!(instructions.is_some());
        let text = instructions.unwrap();
        assert!(text.contains("[IMAGE:<path-or-url>]"));
    }

    #[test]
    fn mattermost_has_default_instructions() {
        let instructions = channel_delivery_instructions("mattermost");
        assert!(instructions.is_some());
    }

    #[test]
    fn matrix_has_default_instructions() {
        let instructions = channel_delivery_instructions("matrix");
        assert!(instructions.is_some());
    }

    #[test]
    fn dingtalk_has_default_instructions() {
        let instructions = channel_delivery_instructions("dingtalk");
        assert!(instructions.is_some());
        let text = instructions.unwrap();
        assert!(text.contains("[IMAGE:<path-or-url>]"));
        assert!(text.contains("Be concise and direct"));
    }

    #[test]
    fn lark_has_default_instructions() {
        let instructions = channel_delivery_instructions("lark");
        assert!(instructions.is_some());
    }

    #[test]
    fn feishu_has_default_instructions() {
        let instructions = channel_delivery_instructions("feishu");
        assert!(instructions.is_some());
    }

    #[test]
    fn signal_has_default_instructions() {
        let instructions = channel_delivery_instructions("signal");
        assert!(instructions.is_some());
    }

    #[test]
    fn whatsapp_has_default_instructions() {
        let instructions = channel_delivery_instructions("whatsapp");
        assert!(instructions.is_some());
    }

    #[test]
    fn qq_has_default_instructions() {
        let instructions = channel_delivery_instructions("qq");
        assert!(instructions.is_some());
    }

    #[test]
    fn cli_has_no_instructions() {
        let instructions = channel_delivery_instructions("cli");
        assert!(instructions.is_none());
    }

    #[test]
    fn dummy_has_no_instructions() {
        let instructions = channel_delivery_instructions("dummy");
        assert!(instructions.is_none());
    }

    #[test]
    fn clawdtalk_has_no_instructions() {
        let instructions = channel_delivery_instructions("ClawdTalk");
        assert!(instructions.is_none());
    }

    #[test]
    fn default_instructions_contain_media_markers() {
        let instructions = channel_delivery_instructions("discord");
        assert!(instructions.is_some());
        let text = instructions.unwrap();
        assert!(text.contains("[IMAGE:<path-or-url>]"));
        assert!(text.contains("[DOCUMENT:<path-or-url>]"));
        assert!(text.contains("[VIDEO:<path-or-url>]"));
        assert!(text.contains("[AUDIO:<path-or-url>]"));
        assert!(text.contains("[VOICE:<path-or-url>]"));
    }

    #[test]
    fn default_instructions_emphasize_conciseness() {
        let instructions = channel_delivery_instructions("slack");
        assert!(instructions.is_some());
        let text = instructions.unwrap();
        assert!(text.contains("Be concise and direct"));
        assert!(text.contains("Skip filler phrases"));
    }

    #[test]
    fn default_instructions_guide_tool_result_usage() {
        let instructions = channel_delivery_instructions("mattermost");
        assert!(instructions.is_some());
        let text = instructions.unwrap();
        assert!(text.contains("Use tool results silently"));
        assert!(text.contains("do not narrate"));
    }
}
