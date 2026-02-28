use super::traits::{ChatMessage, Provider};
use async_trait::async_trait;
use anyhow::{Context, Result};
use std::process::Stdio;
use tokio::process::Command;

/// Strip ANSI escape codes and terminal artifacts from kiro-cli output.
fn strip_ansi_and_artifacts(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                while let Some(&nc) = chars.peek() {
                    chars.next();
                    if nc.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            result.push(c);
        }
    }

    let cleaned: String = result
        .lines()
        .map(|line| {
            let l = line.strip_prefix("> ").unwrap_or(line);
            l.strip_suffix("mm")
                .unwrap_or_else(|| l.strip_suffix('m').unwrap_or(l))
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Convert markdown images to ZeroClaw [IMAGE:path] markers
    convert_md_images(&cleaned.trim().to_string())
}

/// Convert `![alt](file:///path)` and `![alt](/path)` to `[IMAGE:/path]`
/// Also detect bare image file paths like `/path/to/image.png` in text.
fn convert_md_images(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut remaining = s;

    while let Some(start) = remaining.find("![") {
        result.push_str(&remaining[..start]);
        remaining = &remaining[start..];

        if let Some(paren_start) = remaining.find("](") {
            if let Some(paren_end) = remaining[paren_start + 2..].find(')') {
                let alt_text = &remaining[2..paren_start];
                let url = &remaining[paren_start + 2..paren_start + 2 + paren_end];
                let path = url.strip_prefix("file://").unwrap_or(url);

                if path.starts_with('/') {
                    result.push_str(&format!("[IMAGE:{path}]"));
                } else if !alt_text.is_empty() {
                    result.push_str(&format!("[IMAGE:{url}]"));
                } else {
                    result.push_str(&remaining[..paren_start + 2 + paren_end + 1]);
                }
                remaining = &remaining[paren_start + 2 + paren_end + 1..];
                continue;
            }
        }

        result.push_str(&remaining[..2]);
        remaining = &remaining[2..];
    }

    result.push_str(remaining);
    detect_bare_image_paths(&result)
}

/// Find bare absolute image paths in text and wrap as [IMAGE:path]
fn detect_bare_image_paths(s: &str) -> String {
    let img_exts = [".png", ".jpg", ".jpeg", ".gif", ".webp", ".bmp"];
    let mut result = s.to_string();
    for line in s.lines() {
        for word in line.split_whitespace() {
            let clean = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '/' && c != '.' && c != '_' && c != '-');
            if clean.starts_with('/')
                && img_exts.iter().any(|ext| clean.to_lowercase().ends_with(ext))
                && !result.contains(&format!("[IMAGE:{clean}]"))
            {
                result = result.replace(clean, &format!("[IMAGE:{clean}]"));
            }
        }
    }
    result
}

pub struct KiroProvider {
    kiro_path: String,
    agent: Option<String>,
    model: Option<String>,
}

impl KiroProvider {
    pub fn new(kiro_path: Option<&str>, model: Option<&str>) -> Self {
        let agent = std::env::var("KIRO_AGENT").ok();
        let resolved_path = kiro_path
            .map(ToString::to_string)
            .or_else(|| std::env::var("KIRO_CLI_PATH").ok())
            .or_else(|| which::which("kiro-cli").ok().map(|p| p.to_string_lossy().to_string()))
            .unwrap_or_else(|| "kiro-cli".to_string());

        Self {
            kiro_path: resolved_path,
            agent,
            model: model.map(ToString::to_string),
        }
    }

    async fn invoke_kiro(&self, prompt: &str) -> Result<String> {
        let mut cmd = Command::new(&self.kiro_path);
        cmd.arg("chat")
            .arg("--no-interactive");
        
        if let Some(ref agent) = self.agent {
            cmd.arg("--agent").arg(agent);
        }
        
        if let Some(ref model) = self.model {
            cmd.arg("--model").arg(model);
        }
        
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env("NO_COLOR", "1")
            .env("TERM", "dumb");

        let mut child = cmd
            .spawn()
            .context("Failed to spawn kiro-cli")?;

        // Write prompt via stdin to avoid ARG_MAX limits
        if let Some(mut stdin) = child.stdin.take() {
            use tokio::io::AsyncWriteExt;
            stdin.write_all(prompt.as_bytes()).await
                .context("Failed to write prompt to kiro-cli stdin")?;
            drop(stdin); // Close stdin so kiro-cli knows input is complete
        }

        let output = child.wait_with_output().await
            .context("Failed to wait for kiro-cli")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("kiro-cli exited with status: {} stderr: {}", output.status, stderr);
        }

        let raw = String::from_utf8_lossy(&output.stdout);
        let text = strip_ansi_and_artifacts(&raw);

        // Detect kiro-cli context overflow returned as successful output
        if text.to_lowercase().contains("context window has overflowed") {
            anyhow::bail!("kiro-cli: context window has overflowed");
        }

        Ok(text)
    }

    fn messages_to_prompt(&self, messages: &[ChatMessage]) -> String {
        // Kiro CLI adds its own system prompt, so we must be aggressive about
        // trimming to avoid double-context. Strategy:
        // - From system message: keep ONLY the tool-use protocol block (injected
        //   by chat() default impl) which the LLM needs to emit <tool_call> tags.
        //   Skip ## Tools list, ## Your Task, ## Safety etc — redundant with kiro-cli.
        // - Keep only the last few user/assistant turns to stay within budget.
        const MAX_PROMPT_CHARS: usize = 24_000;
        const MAX_HISTORY_TURNS: usize = 8;

        let mut parts = Vec::new();

        for msg in messages {
            match msg.role.as_str() {
                "system" => {
                    // Extract only the tool-use protocol block (starts after
                    // the injected "## Tool Use Protocol" or similar header
                    // added by ToolsPayload::PromptGuided). This contains
                    // the actual tool schemas the LLM needs.
                    if let Some(idx) = msg.content.find("## Tool Use") {
                        parts.push(msg.content[idx..].to_string());
                    }
                }
                "user" | "assistant" => {
                    let prefix = if msg.role == "user" { "User" } else { "Assistant" };
                    parts.push(format!("{prefix}: {}", msg.content));
                }
                _ => {}
            }
        }

        // Keep system (first) + last N turns
        if parts.len() > MAX_HISTORY_TURNS + 1 {
            let system_part = parts.remove(0);
            let tail = parts.split_off(parts.len().saturating_sub(MAX_HISTORY_TURNS));
            parts = std::iter::once(system_part).chain(tail).collect();
        }

        let mut prompt = parts.join("\n\n");
        if prompt.len() > MAX_PROMPT_CHARS {
            prompt.truncate(prompt.floor_char_boundary(MAX_PROMPT_CHARS));
        }
        prompt
    }
}

#[async_trait]
impl Provider for KiroProvider {
    fn supports_native_tools(&self) -> bool {
        false
    }

    fn supports_vision(&self) -> bool {
        true
    }

    fn supports_raw_image_markers(&self) -> bool {
        true
    }

    async fn chat_with_system(
        &self,
        system: Option<&str>,
        message: &str,
        _model: &str,
        _temperature: f64,
    ) -> Result<String> {
        let mut prompt = String::new();
        
        if let Some(sys) = system {
            prompt.push_str("System: ");
            prompt.push_str(sys);
            prompt.push_str("\n\n");
        }
        
        prompt.push_str("User: ");
        prompt.push_str(message);

        self.invoke_kiro(&prompt).await
    }

    async fn chat_with_history(
        &self,
        messages: &[ChatMessage],
        _model: &str,
        _temperature: f64,
    ) -> Result<String> {
        let prompt = self.messages_to_prompt(messages);
        self.invoke_kiro(&prompt).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_to_prompt_extracts_tools_and_user() {
        let provider = KiroProvider::new(None, None);
        let system = "## Tools\n\n- **screenshot**: Take a screenshot\n\n## Your Task\n\nAct on requests.\n\n## Workspace\n\nWorking dir: /tmp\n\n## Tool Use Protocol\n\nYou have tools. Use <tool_call> to invoke.\n\nAvailable tools:\n- screenshot: {}\n";
        let messages = vec![
            ChatMessage::system(system.to_string()),
            ChatMessage::user("Take a screenshot"),
        ];
        let prompt = provider.messages_to_prompt(&messages);
        // Should keep tool use protocol
        assert!(prompt.contains("## Tool Use Protocol"));
        assert!(prompt.contains("User: Take a screenshot"));
        // Should NOT keep ## Tools list, ## Your Task, ## Workspace
        assert!(!prompt.contains("## Tools"));
        assert!(!prompt.contains("## Your Task"));
        assert!(!prompt.contains("Working dir"));
    }
}
