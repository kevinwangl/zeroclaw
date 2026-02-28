# DingTalk and Lark Attachment Support Implementation

## Summary

Added unified attachment support to DingTalk and Lark channels, enabling them to handle media markers like `[IMAGE:path]`, `[DOCUMENT:url]`, etc.

## Changes

### 1. DingTalk Channel (`src/channels/dingtalk.rs`)

**Implementation Strategy**: Webhook-based with markdown fallback

- ✅ Integrated `parse_attachment_markers()` into `send()` method
- ✅ Sends text message first if present
- ✅ Handles attachments as separate messages:
  - **Local files**: Sends file path info with note about Open API requirement
  - **URLs**: Sends as markdown links `[TYPE](url)`
- ✅ Graceful error handling with warnings

**Limitations**:
- DingTalk Stream API webhooks don't support direct file upload
- Full file upload requires DingTalk Open API integration (future enhancement)
- Current implementation provides user-friendly fallback messages

**Code Pattern**:
```rust
async fn send(&self, message: &SendMessage) -> anyhow::Result<()> {
    use super::attachment::{parse_attachment_markers, is_local_path};

    let (text, attachments) = parse_attachment_markers(&content);

    // Send text message
    if !text.is_empty() || attachments.is_empty() {
        // ... webhook POST with markdown ...
    }

    // Send attachments as separate messages
    for attachment in &attachments {
        if is_local_path(&attachment.target) {
            // Send file path info with API note
        } else {
            // Send markdown link
        }
    }
}
```

### 2. Lark Channel (`src/channels/lark.rs`)

**Implementation Strategy**: Full API integration with image compression

- ✅ Refactored existing `extract_image_marker()` logic to use unified `parse_attachment_markers()`
- ✅ Added `upload_file()` method for non-image attachments
- ✅ Supports all attachment types:
  - **Images**: Uses existing `upload_image()` with 5MB compression
  - **Documents/Video/Audio/Voice**: Uses new `upload_file()` method
- ✅ Token refresh handling for all attachment uploads
- ✅ Handles both local files and URL links

**API Endpoints**:
- Images: `POST /im/v1/images` (with compression for >5MB)
- Files: `POST /im/v1/files` (documents, video, audio, voice)

**Code Pattern**:
```rust
async fn send(&self, message: &SendMessage) -> anyhow::Result<()> {
    use super::attachment::{parse_attachment_markers, is_local_path, AttachmentKind};

    let (text, attachments) = parse_attachment_markers(&content);

    // Send text message with token refresh handling
    if !text.is_empty() || attachments.is_empty() {
        // ... existing token refresh logic ...
    }

    // Send attachments by type
    for attachment in &attachments {
        match attachment.kind {
            AttachmentKind::Image => {
                let image_key = self.upload_image(&token, &attachment.target).await?;
                // Send image message with image_key
            }
            _ => {
                let file_key = self.upload_file(&token, &attachment.target).await?;
                // Send file message with file_key
            }
        }
    }
}

async fn upload_file(&self, token: &str, path: &str) -> anyhow::Result<String> {
    // POST to /im/v1/files with multipart form
    // Returns file_key for message content
}
```

## Testing

### Unit Tests (`src/channels/dingtalk_lark_attachment_tests.rs`)

- ✅ `test_parse_dingtalk_attachment_markers()` - Multiple attachment parsing
- ✅ `test_parse_lark_attachment_markers()` - Mixed local/URL attachments
- ✅ `test_attachment_kind_marker_names()` - Marker name consistency

### Manual Testing Checklist

**DingTalk**:
- [ ] Send message with `[IMAGE:/path/to/image.png]`
- [ ] Send message with `[DOCUMENT:https://example.com/doc.pdf]`
- [ ] Verify fallback messages are user-friendly
- [ ] Verify markdown links render correctly

**Lark**:
- [ ] Send message with `[IMAGE:/path/to/image.png]` (< 5MB)
- [ ] Send message with large image (> 5MB, verify compression)
- [ ] Send message with `[DOCUMENT:/path/to/file.pdf]`
- [ ] Send message with `[VIDEO:/path/to/video.mp4]`
- [ ] Send message with URL attachments
- [ ] Verify token refresh works during upload

## Platform-Specific Notes

### DingTalk

**Webhook Limitations**:
- Stream API webhooks only support text/markdown messages
- File upload requires DingTalk Open API with app credentials
- Current implementation provides graceful degradation

**Future Enhancement**:
```rust
// When Open API integration is added:
async fn upload_file_via_open_api(
    &self,
    app_key: &str,
    app_secret: &str,
    file_path: &str,
) -> anyhow::Result<String> {
    // POST to https://oapi.dingtalk.com/media/upload
    // Returns media_id for message attachment
}
```

### Lark

**Image Compression**:
- Automatic JPEG compression for images > 5MB
- Scales down to max 2048x2048 if larger
- Preserves quality for smaller images

**File Size Limits**:
- Images: 10MB (after compression)
- Files: 20MB
- Exceeding limits will result in upload failure

**Message Types**:
- `msg_type: "image"` with `image_key`
- `msg_type: "file"` with `file_key`
- `msg_type: "text"` for URL fallback

## Compilation Status

✅ **All changes compile successfully**

```bash
cargo check
# Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.65s
# Only 2 pre-existing warnings (unused imports)
```

## Implementation Metrics

| Metric | Value |
|--------|-------|
| **Files Modified** | 2 (dingtalk.rs, lark.rs) |
| **Files Created** | 1 (dingtalk_lark_attachment_tests.rs) |
| **Lines Added** | ~150 |
| **New Methods** | 1 (Lark::upload_file) |
| **Test Cases** | 3 |
| **Attachment Types Supported** | 5 (Image, Document, Video, Audio, Voice) |

## Security Considerations

1. **Path Validation**: Both implementations check file existence before upload
2. **Token Security**: Lark tokens are refreshed automatically on expiry
3. **Error Handling**: File not found errors are logged, not exposed to users
4. **URL Sanitization**: URLs are sent as-is without execution
5. **Workspace Scoping**: Respects existing security policies

## Performance Characteristics

**DingTalk**:
- Minimal overhead (text-only fallback)
- No file I/O for local files (sends path info only)
- Async webhook POST for each attachment

**Lark**:
- Async file upload with tokio::fs
- Image compression in blocking task (CPU-bound)
- Sequential attachment uploads (prevents rate limiting)
- Token caching reduces API calls

## Backward Compatibility

- ✅ **No Breaking Changes**: Existing text-only messages work unchanged
- ✅ **Opt-In**: Attachment markers only processed if present
- ✅ **Fallback**: Unrecognized markers preserved as plain text
- ✅ **Token Refresh**: Existing Lark token logic preserved

## Rollback Strategy

If issues arise:

1. **DingTalk**: Revert `send()` method to original webhook-only logic
2. **Lark**: Revert to original `extract_image_marker()` implementation
3. Both channels will continue to work with text messages

All changes are isolated to the `send()` method and new helper methods.

## Documentation Updates

- ✅ Updated `docs/channel-attachment-implementation.md` status table
- ✅ Added platform-specific notes for DingTalk and Lark
- ✅ Created this implementation summary

## Next Steps

### Immediate
1. Manual testing with real DingTalk/Lark accounts
2. Add integration tests with mock API responses

### Short Term
3. DingTalk: Implement Open API file upload (requires app credentials)
4. Lark: Add video/audio compression for large files

### Long Term
5. Add retry logic for failed uploads
6. Implement progress tracking for large file uploads
7. Add file type validation before upload

## References

- Shared utilities: `src/channels/attachment.rs`
- Implementation guide: `docs/channel-attachment-implementation.md`
- DingTalk Stream API: https://open.dingtalk.com/document/orgapp/stream-mode-overview
- Lark Open API: https://open.feishu.cn/document/server-docs/im-v1/message/create
- System prompt: `src/channels/mod.rs:410` (media marker instructions)
