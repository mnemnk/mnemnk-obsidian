use anyhow::{Context as _, Result, bail};
use clap::Parser;
use dotenv::dotenv;
use log;
use reqwest::{
    Client,
    header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, BufReader, stdin};
use tokio::signal::ctrl_c;

const AGENT_NAME: &str = "mnemnk-obsidian-append-note";

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AgentConfig {}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {}
    }
}

impl From<&str> for AgentConfig {
    fn from(_s: &str) -> Self {
        AgentConfig::default()
    }
}

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(short = 'c', long = "config", help = "JSON config string")]
    config: Option<String>,
}

#[derive(Debug, Deserialize)]
struct InData {
    ctx: Value,
    data: Value,
}

struct ObsidianAppendNoteAgent {
    #[allow(unused)]
    config: AgentConfig,
}

impl ObsidianAppendNoteAgent {
    async fn new(config: AgentConfig) -> Result<Self> {
        Ok(Self { config })
    }

    async fn run(&mut self) -> Result<()> {
        let mut reader = BufReader::new(stdin());
        let mut line = String::new();

        loop {
            tokio::select! {
                result = reader.read_line(&mut line) => {
                    if result.is_ok() {
                        self.process_line(&line).await.unwrap_or_else(|e| {
                            log::error!("Error processing line: {}", e);
                        });
                    } else {
                        log::error!("Error reading from stdin: {:?}", result);
                    }
                    line.clear();
                }

                _ = ctrl_c() => {
                    log::info!("Received interrupt signal, shutting down");
                    break;
                }
            }
        }

        log::info!("Shutting down {}", AGENT_NAME);
        Ok(())
    }

    async fn process_line(&mut self, line: &str) -> Result<()> {
        let line = line.trim();
        if line.is_empty() {
            return Ok(());
        }

        log::debug!("Processing line: {}", line);

        if let Some((cmd, args)) = line.split_once(' ') {
            match cmd {
                ".CONFIG" => {}

                ".IN" => {
                    let in_data: InData = serde_json::from_str(args)?;
                    self.process_input(in_data.ctx, in_data.data).await?;
                }

                ".QUIT" => {
                    log::info!("Received quit command");
                    std::process::exit(0);
                }

                _ => {
                    log::warn!("Unknown command: {}", cmd);
                }
            }
        }

        Ok(())
    }

    async fn process_input(&self, ctx: Value, data: Value) -> Result<()> {
        let Some(obj) = data.as_object() else {
            bail!("Invalid data for append_note command");
        };
        let Some(value) = obj.get("value") else {
            bail!("Missing value");
        };
        let Some(obj) = value.as_object() else {
            bail!("data is not an object value");
        };
        let path = obj
            .get("path")
            .and_then(|v| v.as_str())
            .context("Missing path")?;
        if path.is_empty() {
            bail!("Empty path");
        }
        if path.ends_with("/") {
            bail!("Path ends with /");
        }
        let content = obj
            .get("content")
            .and_then(|v| v.as_str())
            .context("Missing content")?;

        self.append_note(path, content).await?;

        let out = json!({
            "ctx": ctx,
            "ch": "data",
            "data": data,
        });
        println!(".OUT {}", serde_json::to_string(&out)?);

        Ok(())
    }

    async fn append_note(&self, path: &str, content: &str) -> Result<()> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/markdown"));

        let api_key = std::env::var("OBSIDIAN_API_KEY").unwrap_or_default();
        if !api_key.is_empty() {
            let auth_value = format!("Bearer {}", api_key);
            headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_value)?);
        }

        let client = Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        let api_url = std::env::var("OBSIDIAN_API_URL")
            .unwrap_or_else(|_| "http://localhost:27123".to_string());

        let url = format!("{}/vault/{}", api_url, urlencoding::encode(path));
        let response = client.post(&url).body(content.to_string()).send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Failed to append_note: {}",
                response.status()
            ))
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    dotenv().ok();

    let args = Args::parse();
    let config = args.config.as_deref().unwrap_or("").into();

    let mut agent = ObsidianAppendNoteAgent::new(config).await?;
    agent.run().await?;

    Ok(())
}
