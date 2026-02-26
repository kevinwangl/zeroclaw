use super::traits::{
    ChatMessage, ChatRequest, ChatResponse, Provider, StreamChunk, StreamError, StreamOptions,
    StreamResult,
};
use crate::tools::ToolSpec;
use async_trait::async_trait;
use anyhow::{Context, Result};
use futures_util::stream;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

pub struct KiroProvider {
    kiro_path: String,
    agent: Option<String>,
    model: Option<String>,
}

impl KiroProvider {
    pub fn new(kiro_path: Option<&str>, model: Option<&str>) -> Self {
        let agent = std::env::var("KIRO_AGENT").ok();
        
        Self {
            kiro_path: kiro_path.unwrap_or("kiro-cli").to_string(),
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
            .stderr(Stdio::null());

        let output = cmd
            .output()
            .await
            .context("Failed to execute kiro-cli")?;
        
        if !output.status.success() {
            anyhow::bail!("kiro-cli exited with status: {}", output.status);
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
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
