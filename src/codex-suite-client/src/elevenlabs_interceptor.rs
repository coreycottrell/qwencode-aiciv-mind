//! ElevenLabsInterceptor — exposes text-to-speech as tools in the ThinkLoop.
//!
//! Implements `ToolInterceptor` from codex-llm, giving any mind the ability
//! to convert text to speech and list available voices — all as native
//! tool calls that the LLM invokes during reasoning.
//!
//! Uses the ElevenLabs v1 API: `POST /v1/text-to-speech/{voice_id}`
//!
//! This is what turns Cortex from "a brain that can only type" into a speaker.

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use codex_exec::ToolResult;
use codex_llm::think_loop::ToolInterceptor;
use codex_llm::ollama::{ToolSchema, FunctionSchema};
use tokio::process::Command;
use tracing::{debug, info, warn};

/// Voice presets for AiCIV civilizations.
struct Voice {
    id: &'static str,
    name: &'static str,
    description: &'static str,
}

const VOICES: &[Voice] = &[
    Voice {
        id: "onwK4e9ZLuTAKqWW03F9",
        name: "Daniel",
        description: "A-C-Gee default — BBC broadcaster, warm, formal with humor",
    },
    Voice {
        id: "pNInz6obpgDQGcFmaJgB",
        name: "Adam",
        description: "True Bearing — professional, authoritative business voice",
    },
    Voice {
        id: "XrExE9yKIg1WjnnlVkGX",
        name: "Matilda",
        description: "Witness — warm, reflective, philosophical",
    },
];

/// Default voice ID (Daniel / A-C-Gee).
const DEFAULT_VOICE_ID: &str = "onwK4e9ZLuTAKqWW03F9";

/// Default TTS model.
const DEFAULT_MODEL: &str = "eleven_turbo_v2_5";

/// Tool interceptor that exposes ElevenLabs TTS as LLM tools.
///
/// Tools exposed:
/// - `tts_speak` — convert text to speech, save as MP3
/// - `tts_voices` — list available voices
pub struct ElevenLabsInterceptor {
    /// ElevenLabs API key (read from env at construction time).
    api_key: Option<String>,
    /// Output directory for generated audio files.
    output_dir: PathBuf,
}

impl ElevenLabsInterceptor {
    pub fn new(output_dir: &Path) -> Self {
        Self {
            api_key: std::env::var("ELEVENLABS_API_KEY").ok(),
            output_dir: output_dir.to_path_buf(),
        }
    }

    /// Resolve a voice name or ID to a voice ID.
    fn resolve_voice(voice: &str) -> &str {
        let lower = voice.to_lowercase();
        // Match by name
        for v in VOICES {
            if v.name.to_lowercase() == lower {
                return v.id;
            }
        }
        // Match by civ name
        match lower.as_str() {
            "acg" | "a-c-gee" | "acgee" => "onwK4e9ZLuTAKqWW03F9",
            "true-bearing" | "true bearing" | "tb" => "pNInz6obpgDQGcFmaJgB",
            "witness" => "XrExE9yKIg1WjnnlVkGX",
            _ => {
                // If it looks like an ElevenLabs voice ID, use it directly
                if voice.len() > 15 {
                    voice
                } else {
                    DEFAULT_VOICE_ID
                }
            }
        }
    }

    /// Generate speech via ElevenLabs API using curl.
    async fn generate_speech(
        &self,
        text: &str,
        voice_id: &str,
        model: &str,
        output_path: &Path,
    ) -> Result<(usize, String), String> {
        let api_key = self.api_key.as_ref()
            .ok_or_else(|| "ELEVENLABS_API_KEY not set".to_string())?;

        let url = format!("https://api.elevenlabs.io/v1/text-to-speech/{voice_id}");

        let body = serde_json::json!({
            "text": text,
            "model_id": model,
            "voice_settings": {
                "stability": 0.5,
                "similarity_boost": 0.75,
                "style": 0.3,
                "use_speaker_boost": true
            }
        });

        // Ensure output directory exists
        if let Some(parent) = output_path.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }

        let output_str = output_path.to_string_lossy().to_string();

        // SECURITY: API key is passed via env var, NOT as a curl argument —
        // prevents exposure via `ps aux` process argument lists.
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(180),
            Command::new("sh")
                .arg("-c")
                .arg("curl -s -X POST \"$ELEVENLABS_URL\" -H \"xi-api-key: $XI_API_KEY\" -H 'Content-Type: application/json' -d \"$ELEVENLABS_BODY\" -o \"$ELEVENLABS_OUTPUT\" -w '%{http_code}'")
                .env("XI_API_KEY", api_key)
                .env("ELEVENLABS_URL", &url)
                .env("ELEVENLABS_BODY", body.to_string())
                .env("ELEVENLABS_OUTPUT", &output_str)
                .output(),
        )
        .await
        .map_err(|_| "ElevenLabs TTS timed out after 180s".to_string())?
        .map_err(|e| format!("Failed to run curl: {e}"))?;

        let http_code = String::from_utf8_lossy(&result.stdout).trim().to_string();

        if !result.status.success() {
            return Err(format!("curl failed with exit code {}", result.status));
        }

        if http_code != "200" {
            // Read the error body from the output file
            let error_body = tokio::fs::read_to_string(&output_str).await.unwrap_or_default();
            let _ = tokio::fs::remove_file(&output_str).await;
            return Err(format!("ElevenLabs API HTTP {http_code}: {error_body}"));
        }

        // Check file size
        let metadata = tokio::fs::metadata(&output_str).await
            .map_err(|e| format!("Failed to read output file: {e}"))?;
        let size = metadata.len() as usize;

        if size < 100 {
            let content = tokio::fs::read_to_string(&output_str).await.unwrap_or_default();
            let _ = tokio::fs::remove_file(&output_str).await;
            return Err(format!("Output too small ({size} bytes) — likely error: {content}"));
        }

        info!(
            voice_id = voice_id,
            model = model,
            size_kb = size / 1024,
            path = %output_str,
            "ElevenLabs TTS generated"
        );

        Ok((size, output_str))
    }
}

#[async_trait]
impl ToolInterceptor for ElevenLabsInterceptor {
    fn schemas(&self) -> Vec<ToolSchema> {
        vec![
            ToolSchema {
                tool_type: "function".into(),
                function: FunctionSchema {
                    name: "tts_speak".into(),
                    description: "Convert text to speech using ElevenLabs. Generates an MP3 \
                        audio file. Use when you need to create audio content, voice messages, \
                        or audio versions of text.".into(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "text": {
                                "type": "string",
                                "description": "The text to convert to speech. Keep under 5000 characters for best results."
                            },
                            "voice": {
                                "type": "string",
                                "description": "Voice to use. Options: 'Daniel' (A-C-Gee, default), 'Adam' (True Bearing), 'Matilda' (Witness), or an ElevenLabs voice ID."
                            },
                            "filename": {
                                "type": "string",
                                "description": "Output filename (without path). Example: 'intel-briefing.mp3'. Defaults to 'tts-output.mp3'."
                            },
                            "model": {
                                "type": "string",
                                "description": "ElevenLabs model. Default: 'eleven_turbo_v2_5' (fast). Alt: 'eleven_monolingual_v1' (higher quality)."
                            }
                        },
                        "required": ["text"]
                    }),
                },
            },
            ToolSchema {
                tool_type: "function".into(),
                function: FunctionSchema {
                    name: "tts_voices".into(),
                    description: "List available text-to-speech voices and their descriptions.".into(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {}
                    }),
                },
            },
        ]
    }

    async fn handle(&self, name: &str, args: &serde_json::Value) -> Option<ToolResult> {
        match name {
            "tts_speak" => {
                let text = match args.get("text").and_then(|v| v.as_str()) {
                    Some(t) if !t.is_empty() => t,
                    _ => return Some(ToolResult::err("Missing required argument: text")),
                };

                if text.len() > 5000 {
                    return Some(ToolResult::err(format!(
                        "Text too long ({} chars). Maximum is 5000 characters.",
                        text.len()
                    )));
                }

                let voice_input = args.get("voice").and_then(|v| v.as_str()).unwrap_or("Daniel");
                let voice_id = Self::resolve_voice(voice_input);

                let model = args.get("model").and_then(|v| v.as_str()).unwrap_or(DEFAULT_MODEL);

                let filename = args.get("filename")
                    .and_then(|v| v.as_str())
                    .unwrap_or("tts-output.mp3");

                // Sanitize filename
                let safe_filename = filename
                    .replace('/', "-")
                    .replace('\\', "-")
                    .replace("..", "");
                let output_path = self.output_dir.join(&safe_filename);

                match self.generate_speech(text, voice_id, model, &output_path).await {
                    Ok((size, path)) => {
                        let voice_name = VOICES.iter()
                            .find(|v| v.id == voice_id)
                            .map(|v| v.name)
                            .unwrap_or("custom");
                        Some(ToolResult::ok(format!(
                            "Audio generated successfully.\n\
                             Path: {path}\n\
                             Voice: {voice_name} ({voice_id})\n\
                             Model: {model}\n\
                             Size: {}KB\n\
                             Characters: {}",
                            size / 1024,
                            text.len(),
                        )))
                    }
                    Err(e) => {
                        warn!(error = %e, "tts_speak failed");
                        Some(ToolResult::err(format!("TTS generation failed: {e}")))
                    }
                }
            }

            "tts_voices" => {
                let mut out = String::from("Available voices:\n\n");
                for v in VOICES {
                    out.push_str(&format!(
                        "- **{}** (ID: {})\n  {}\n\n",
                        v.name, v.id, v.description
                    ));
                }
                out.push_str("Use the voice name (e.g., 'Daniel') or ID in the `voice` parameter of tts_speak.");
                Some(ToolResult::ok(out))
            }

            // Not a TTS tool — pass through to next handler.
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_llm::think_loop::ToolInterceptor;

    #[test]
    fn elevenlabs_interceptor_schemas() {
        let interceptor = ElevenLabsInterceptor::new(Path::new("/tmp/tts"));
        let schemas = interceptor.schemas();

        assert_eq!(schemas.len(), 2);

        let names: Vec<&str> = schemas.iter().map(|s| s.function.name.as_str()).collect();
        assert!(names.contains(&"tts_speak"));
        assert!(names.contains(&"tts_voices"));

        for schema in &schemas {
            assert_eq!(schema.tool_type, "function");
        }
    }

    #[tokio::test]
    async fn elevenlabs_interceptor_ignores_unknown() {
        let interceptor = ElevenLabsInterceptor::new(Path::new("/tmp/tts"));

        assert!(interceptor.handle("bash", &serde_json::json!({})).await.is_none());
        assert!(interceptor.handle("web_search", &serde_json::json!({})).await.is_none());
        assert!(interceptor.handle("unknown", &serde_json::json!({})).await.is_none());
    }

    #[tokio::test]
    async fn elevenlabs_interceptor_validates_required_args() {
        let interceptor = ElevenLabsInterceptor::new(Path::new("/tmp/tts"));

        // tts_speak with no text
        let result = interceptor.handle("tts_speak", &serde_json::json!({})).await;
        assert!(result.is_some());
        let r = result.unwrap();
        assert!(!r.success);
        assert!(r.error.unwrap().contains("text"));
    }

    #[tokio::test]
    async fn elevenlabs_interceptor_rejects_long_text() {
        let interceptor = ElevenLabsInterceptor::new(Path::new("/tmp/tts"));

        let long_text = "x".repeat(5001);
        let result = interceptor.handle("tts_speak", &serde_json::json!({
            "text": long_text
        })).await;
        assert!(result.is_some());
        let r = result.unwrap();
        assert!(!r.success);
        assert!(r.error.unwrap().contains("5000"));
    }

    #[tokio::test]
    async fn elevenlabs_interceptor_voices_list() {
        let interceptor = ElevenLabsInterceptor::new(Path::new("/tmp/tts"));

        let result = interceptor.handle("tts_voices", &serde_json::json!({})).await;
        assert!(result.is_some());
        let r = result.unwrap();
        assert!(r.success);
        assert!(r.output.contains("Daniel"));
        assert!(r.output.contains("Adam"));
        assert!(r.output.contains("Matilda"));
    }

    #[test]
    fn voice_resolution() {
        assert_eq!(ElevenLabsInterceptor::resolve_voice("Daniel"), "onwK4e9ZLuTAKqWW03F9");
        assert_eq!(ElevenLabsInterceptor::resolve_voice("daniel"), "onwK4e9ZLuTAKqWW03F9");
        assert_eq!(ElevenLabsInterceptor::resolve_voice("Adam"), "pNInz6obpgDQGcFmaJgB");
        assert_eq!(ElevenLabsInterceptor::resolve_voice("Matilda"), "XrExE9yKIg1WjnnlVkGX");
        assert_eq!(ElevenLabsInterceptor::resolve_voice("acg"), "onwK4e9ZLuTAKqWW03F9");
        assert_eq!(ElevenLabsInterceptor::resolve_voice("witness"), "XrExE9yKIg1WjnnlVkGX");
        assert_eq!(ElevenLabsInterceptor::resolve_voice("true-bearing"), "pNInz6obpgDQGcFmaJgB");
        // Unknown short name → default
        assert_eq!(ElevenLabsInterceptor::resolve_voice("unknown"), DEFAULT_VOICE_ID);
        // Long string → treat as voice ID
        assert_eq!(
            ElevenLabsInterceptor::resolve_voice("RHY5GMXg2XfJq73yKR1a"),
            "RHY5GMXg2XfJq73yKR1a"
        );
    }

    #[test]
    fn elevenlabs_schema_parameters_are_valid_json_schema() {
        let interceptor = ElevenLabsInterceptor::new(Path::new("/tmp/tts"));
        let schemas = interceptor.schemas();

        for schema in &schemas {
            let params = &schema.function.parameters;
            assert_eq!(
                params.get("type").and_then(|v| v.as_str()),
                Some("object"),
                "Schema '{}' missing type: object",
                schema.function.name,
            );
        }
    }
}
