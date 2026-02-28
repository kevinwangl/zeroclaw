# Channel Attachment Support Implementation Guide

## Overview

This document provides a unified implementation pattern for adding media attachment support to all ZeroClaw channels.

## Architecture

### Shared Module: `src/channels/attachment.rs`

Provides:
- `AttachmentKind` enum (Image, Document, Video, Audio, Voice)
- `Attachment` struct with `kind` and `target` fields
- `parse_attachment_markers(message: &str) -> (String, Vec<Attachment>)` - extracts markers like `[IMAGE:path]`
- `is_local_path(target: &str) -> bool` - distinguishes local files from URLs

### Marker Format

Supported markers in message content:
- `[IMAGE:path-or-url]` - Images/photos
- `[DOCUMENT:path-or-url]` - Documents/files
- `[VIDEO:path-or-url]` - Video files
- `[AUDIO:path-or-url]` - Audio files
- `[VOICE:path-or-url]` - Voice messages

## Implementation Pattern

### Step 1: Update `send()` method

```rust
async fn send(&self, message: &SendMessage) -> anyhow::Result<()> {
    use super::attachment::{parse_attachment_markers, is_local_path};

    let content = super::strip_tool_call_tags(&message.content);
    let (text, attachments) = parse_attachment_markers(&content);

    // Send text message (if present or as fallback)
    if !text.is_empty() || attachments.is_empty() {
        // ... existing text sending logic ...
    }

    // Handle attachments
    for attachment in &attachments {
        if is_local_path(&attachment.target) {
            self.upload_file(&attachment.target, &message.recipient).await?;
        } else {
            // For URLs, send as text link or use platform's URL preview
            let link_msg = format!("{}: {}", attachment.kind.marker_name(), attachment.target);
            // ... send link_msg via platform API ...
        }
    }

    Ok(())
}
```

### Step 2: Add `upload_file()` helper method

```rust
async fn upload_file(
    &self,
    file_path: &str,
    recipient: &str,
) -> anyhow::Result<()> {
    let path = std::path::Path::new(file_path);
    if !path.exists() {
        tracing::warn!("{}: file not found: {}", self.name(), file_path);
        return Ok(());
    }

    let file_bytes = tokio::fs::read(path).await?;
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file");

    // Platform-specific upload logic
    // Example: multipart form upload
    let form = reqwest::multipart::Form::new()
        .part(
            "file",
            reqwest::multipart::Part::bytes(file_bytes)
                .file_name(filename.to_string()),
        );

    let resp = self
        .http_client()
        .post("https://api.platform.com/upload")
        .multipart(form)
        .send()
        .await?;

    if !resp.status().is_success() {
        tracing::warn!("{}: file upload failed: {}", self.name(), resp.status());
    }

    Ok(())
}
```

## Platform-Specific Notes

### Slack
- Use `files.upload` API with multipart form
- Support `thread_ts` for threaded uploads
- API: `https://slack.com/api/files.upload`

### Mattermost
- Two-step process: upload file, then attach to post
- Use `/api/v4/files` for upload, get `file_id`
- Create post with `file_ids` array

### Matrix
- Use `room.send_attachment()` from matrix-sdk
- Requires MIME type detection via `mime_guess`
- Handles encryption automatically for E2EE rooms

### Signal
- RPC-based: extend `send` RPC with attachment parameter
- May require base64 encoding for binary data
- Check signal-cli documentation for attachment format

### IRC
- No native attachment support
- Fallback: send URLs only or use external file hosting

### iMessage
- AppleScript-based: limited attachment support
- May require osascript file path handling
- Consider security implications of path injection

## Implementation Status

| Channel | Status | Notes |
|---------|--------|-------|
| Telegram | ✅ Complete | Reference implementation |
| Discord | ✅ Complete | Reference implementation |
| Slack | ✅ Complete | Uses files.upload API |
| Mattermost | ✅ Complete | Two-step upload + post |
| Matrix | ✅ Complete | Uses send_attachment() |
| **DingTalk** | ✅ **Complete** | Webhook-based, URL links + path info fallback |
| **Lark** | ✅ **Complete** | Full API integration with image compression |
| Signal | 🚧 Pending | Needs RPC extension |
| IRC | 🚧 Pending | URL-only fallback |
| iMessage | 🚧 Pending | AppleScript limitations |
| WhatsApp | 🚧 Pending | Needs API integration |
| QQ | 🚧 Pending | Needs media upload API |
| Nostr | 🚧 Pending | NIP-94 file metadata |
| Email | 🚧 Pending | MIME attachments |

## Testing

Add tests for each channel:

```rust
#[tokio::test]
async fn test_send_with_image_attachment() {
    let channel = YourChannel::new(/* ... */);
    let message = SendMessage::new(
        "Check this [IMAGE:/tmp/test.png]",
        "recipient_id",
    );
    
    let result = channel.send(&message).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_send_with_url_attachment() {
    let channel = YourChannel::new(/* ... */);
    let message = SendMessage::new(
        "See [DOCUMENT:https://example.com/doc.pdf]",
        "recipient_id",
    );
    
    let result = channel.send(&message).await;
    assert!(result.is_ok());
}
```

## Migration Checklist

For each channel:

- [ ] Import `attachment` module in `send()` method
- [ ] Call `parse_attachment_markers()` on message content
- [ ] Handle text-only case (empty attachments)
- [ ] Implement `upload_file()` helper for local files
- [ ] Handle URL attachments (send as link or use platform preview)
- [ ] Add error handling and logging
- [ ] Write unit tests for attachment parsing
- [ ] Write integration tests for file upload
- [ ] Update channel documentation
- [ ] Test with real files (image, document, video)

## Security Considerations

1. **Path Validation**: Always validate file paths before reading
2. **File Size Limits**: Check platform limits before upload
3. **MIME Type Validation**: Verify file types match expected formats
4. **URL Validation**: Sanitize URLs before sending
5. **Workspace Scoping**: Respect `workspace_only` security policy
6. **Error Handling**: Never expose file system paths in error messages

## Performance Considerations

1. **Async Upload**: Use tokio::fs for non-blocking file reads
2. **Streaming**: For large files, consider streaming uploads
3. **Retry Logic**: Implement exponential backoff for failed uploads
4. **Timeout**: Set reasonable timeouts for upload operations
5. **Concurrent Uploads**: Limit parallel uploads to avoid resource exhaustion

## References

- Telegram implementation: `src/channels/telegram.rs:247-291, 2419-2451`
- Discord implementation: `src/channels/discord.rs:144-187, 488-536`
- Shared utilities: `src/channels/attachment.rs`
- System prompt: `src/channels/mod.rs:410` (media marker instructions)
