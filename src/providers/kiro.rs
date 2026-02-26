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
/// so channels with attachment support (e.g. Telegram) can send the file.
/// For channels without attachment support (e.g. Feishu text-only send),
/// the marker is still cleaner than raw markdown image syntax.
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
            .arg("--no-interactive")
            .arg(prompt);
        
        if let Some(ref agent) = self.agent {
            cmd.arg("--agent").arg(agent);
        }
        
        if let Some(ref model) = self.model {
            cmd.arg("--model").arg(model);
        }
        
        cmd.stdout(Stdio::piped())
            .stderr(Stdio::null())
            .env("NO_COLOR", "1")
            .env("TERM", "dumb");

        let output = cmd
            .output()
            .await
            .context("Failed to execute kiro-cli")?;
        
        if !output.status.success() {
            anyhow::bail!("kiro-cli exited with status: {}", output.status);
        }

        let raw = String::from_utf8_lossy(&output.stdout);
        Ok(strip_ansi_and_artifacts(&raw))
    }

    fn messages_to_prompt(&self, messages: &[ChatMessage]) -> String {
        let mut parts = Vec::new();
        
        for msg in messages {
            match msg.role.as_str() {
                "system" => parts.push(format!("System: {}", msg.content)),
                "user" => parts.push(format!("User: {}", msg.content)),
                "assistant" => parts.push(format!("Assistant: {}", msg.content)),
                _ => {}
            }
        }
        
        parts.join("\n\n")
    }
}

#[async_trait]
impl Provider for KiroProvider {
    fn supports_native_tools(&self) -> bool {
        false
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
    fn messages_to_prompt_formats_correctly() {
        let provider = KiroProvider::new(None, None);
        let messages = vec![
            ChatMessage::system("You are helpful"),
            ChatMessage::user("Hello"),
        ];
        let prompt = provider.messages_to_prompt(&messages);
        assert!(prompt.contains("System: You are helpful"));
        assert!(prompt.contains("User: Hello"));
    }
}
