#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dingtalk_attachment_markers() {
        use crate::channels::attachment::parse_attachment_markers;

        let input = "Check this report [IMAGE:/tmp/chart.png] and [DOCUMENT:/tmp/report.pdf]";
        let (text, attachments) = parse_attachment_markers(input);

        assert_eq!(text, "Check this report  and");
        assert_eq!(attachments.len(), 2);
        assert_eq!(attachments[0].target, "/tmp/chart.png");
        assert_eq!(attachments[1].target, "/tmp/report.pdf");
    }

    #[test]
    fn test_parse_lark_attachment_markers() {
        use crate::channels::attachment::parse_attachment_markers;

        let input = "Screenshot: [IMAGE:/tmp/screen.png]\nVideo: [VIDEO:https://example.com/demo.mp4]";
        let (text, attachments) = parse_attachment_markers(input);

        assert_eq!(text, "Screenshot: \nVideo:");
        assert_eq!(attachments.len(), 2);
        assert_eq!(attachments[0].target, "/tmp/screen.png");
        assert_eq!(attachments[1].target, "https://example.com/demo.mp4");
    }

    #[test]
    fn test_attachment_kind_marker_names() {
        use crate::channels::attachment::AttachmentKind;

        assert_eq!(AttachmentKind::Image.marker_name(), "IMAGE");
        assert_eq!(AttachmentKind::Document.marker_name(), "DOCUMENT");
        assert_eq!(AttachmentKind::Video.marker_name(), "VIDEO");
        assert_eq!(AttachmentKind::Audio.marker_name(), "AUDIO");
        assert_eq!(AttachmentKind::Voice.marker_name(), "VOICE");
    }
}
