/// Shared attachment parsing utilities for channel implementations.
///
/// This module provides a unified way to parse media markers like [IMAGE:path],
/// [DOCUMENT:url], etc. from message content.

#[derive(Debug, Clone, PartialEq)]
pub enum AttachmentKind {
    Image,
    Document,
    Video,
    Audio,
    Voice,
}

impl AttachmentKind {
    pub fn from_marker(marker: &str) -> Option<Self> {
        match marker.trim().to_ascii_uppercase().as_str() {
            "IMAGE" | "PHOTO" => Some(Self::Image),
            "DOCUMENT" | "FILE" => Some(Self::Document),
            "VIDEO" => Some(Self::Video),
            "AUDIO" => Some(Self::Audio),
            "VOICE" => Some(Self::Voice),
            _ => None,
        }
    }

    pub fn marker_name(&self) -> &'static str {
        match self {
            Self::Image => "IMAGE",
            Self::Document => "DOCUMENT",
            Self::Video => "VIDEO",
            Self::Audio => "AUDIO",
            Self::Voice => "VOICE",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Attachment {
    pub kind: AttachmentKind,
    pub target: String,
}

/// Parse attachment markers from message content.
/// Returns (cleaned_text, attachments).
///
/// Recognizes patterns: [IMAGE:path], [DOCUMENT:url], [VIDEO:path], [AUDIO:path], [VOICE:path]
pub fn parse_attachment_markers(message: &str) -> (String, Vec<Attachment>) {
    let mut cleaned = String::with_capacity(message.len());
    let mut attachments = Vec::new();
    let mut cursor = 0;

    while cursor < message.len() {
        let Some(open_rel) = message[cursor..].find('[') else {
            cleaned.push_str(&message[cursor..]);
            break;
        };

        let open = cursor + open_rel;
        cleaned.push_str(&message[cursor..open]);

        let Some(close_rel) = message[open..].find(']') else {
            cleaned.push_str(&message[open..]);
            break;
        };

        let close = open + close_rel;
        let marker = &message[open + 1..close];

        let parsed = marker.split_once(':').and_then(|(kind, target)| {
            let kind = AttachmentKind::from_marker(kind)?;
            let target = target.trim();
            if target.is_empty() {
                return None;
            }
            Some(Attachment {
                kind,
                target: target.to_string(),
            })
        });

        if let Some(attachment) = parsed {
            attachments.push(attachment);
        } else {
            cleaned.push_str(&message[open..=close]);
        }

        cursor = close + 1;
    }

    (cleaned.trim().to_string(), attachments)
}

/// Check if a target is a local file path (vs URL).
pub fn is_local_path(target: &str) -> bool {
    !target.starts_with("http://") && !target.starts_with("https://")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_image() {
        let (text, attachments) = parse_attachment_markers("Check this [IMAGE:/tmp/a.png]");
        assert_eq!(text, "Check this");
        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments[0].kind, AttachmentKind::Image);
        assert_eq!(attachments[0].target, "/tmp/a.png");
    }

    #[test]
    fn parse_multiple_attachments() {
        let (text, attachments) = parse_attachment_markers(
            "Report\n[IMAGE:https://example.com/a.png]\n[DOCUMENT:/tmp/report.pdf]",
        );
        assert_eq!(text, "Report");
        assert_eq!(attachments.len(), 2);
        assert_eq!(attachments[0].kind, AttachmentKind::Image);
        assert_eq!(attachments[1].kind, AttachmentKind::Document);
    }

    #[test]
    fn parse_preserves_non_markers() {
        let (text, attachments) = parse_attachment_markers("Hello [world] and [not:a:marker]");
        assert_eq!(text, "Hello [world] and [not:a:marker]");
        assert_eq!(attachments.len(), 0);
    }

    #[test]
    fn is_local_path_detection() {
        assert!(is_local_path("/tmp/file.png"));
        assert!(is_local_path("~/file.png"));
        assert!(is_local_path("./file.png"));
        assert!(!is_local_path("http://example.com/file.png"));
        assert!(!is_local_path("https://example.com/file.png"));
    }
}
