//! ImageGenInterceptor — exposes image generation as a tool in the ThinkLoop.
//!
//! Implements `ToolInterceptor` from codex-llm, giving any mind the ability
//! to generate images using the Gemini API (Imagen).
//!
//! Uses a Python helper script (`tools/image_gen.py`) that wraps the
//! `google-genai` SDK with `gemini-3-pro-image-preview`.
//!
//! This is what turns Cortex from "a brain that can only write" into a visual creator.

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use codex_exec::ToolResult;
use codex_llm::think_loop::ToolInterceptor;
use codex_llm::ollama::{ToolSchema, FunctionSchema};
use tokio::process::Command;
use tracing::{debug, info, warn};

/// Truncate a string to at most `max` bytes, landing on a valid UTF-8 char boundary.
fn safe_truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        return s;
    }
    let mut end = max;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

/// Style presets for Cortex image generation.
const STYLES: &[(&str, &str)] = &[
    ("cortex", "dark background (#0d0d0d), orange accent (#ff6b35), monospace typography, machine aesthetic, contemplative, technical"),
    ("cyberpunk", "cyberpunk aesthetic, neon colors, dark background, futuristic"),
    ("minimal", "minimalist design, clean lines, simple composition, white space"),
    ("professional", "professional, corporate, clean, modern, polished"),
    ("organic", "organic shapes, natural colors, flowing lines, biomorphic"),
    ("infographic", "professional infographic, clear labels, structured layout, dark background, data visualization"),
];

/// Tool interceptor that exposes Gemini image generation as LLM tools.
///
/// Tools exposed:
/// - `generate_image` — generate an image from a text prompt
/// - `image_styles` — list available style presets
pub struct ImageGenInterceptor {
    /// GEMINI_API_KEY (read from env at construction time).
    api_key: Option<String>,
    /// Output directory for generated images.
    output_dir: PathBuf,
    /// Project root (to find the Python helper).
    project_root: PathBuf,
}

impl ImageGenInterceptor {
    pub fn new(output_dir: &Path, project_root: &Path) -> Self {
        // Try GEMINI_API_KEY first, then GOOGLE_API_KEY
        let api_key = std::env::var("GEMINI_API_KEY")
            .or_else(|_| std::env::var("GOOGLE_API_KEY"))
            .ok();
        Self {
            api_key,
            output_dir: output_dir.to_path_buf(),
            project_root: project_root.to_path_buf(),
        }
    }

    /// Resolve a style name to a style hint string.
    fn resolve_style(style: &str) -> Option<&'static str> {
        let lower = style.to_lowercase();
        STYLES.iter()
            .find(|(name, _)| *name == lower)
            .map(|(_, hint)| *hint)
    }

    /// Generate an image using the Python helper script.
    async fn generate(
        &self,
        prompt: &str,
        aspect_ratio: &str,
        style: Option<&str>,
        filename: &str,
    ) -> Result<(String, usize), String> {
        let api_key = self.api_key.as_ref()
            .ok_or_else(|| "GEMINI_API_KEY not set (also checked GOOGLE_API_KEY)".to_string())?;

        // Build enhanced prompt with style
        let enhanced_prompt = match style.and_then(Self::resolve_style) {
            Some(hint) => format!("{prompt}. Style: {hint}"),
            None => {
                // If style was provided but not recognized, append it as-is
                match style {
                    Some(s) if !s.is_empty() => format!("{prompt}. Style: {s}"),
                    _ => prompt.to_string(),
                }
            }
        };

        // Sanitize filename
        let safe_filename = filename
            .replace('/', "-")
            .replace('\\', "-")
            .replace("..", "");
        let output_path = self.output_dir.join(&safe_filename);

        // Ensure output directory exists
        if let Some(parent) = output_path.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }

        let output_str = output_path.to_string_lossy().to_string();

        // Use the Python google-genai SDK directly via inline script
        // This avoids depending on ACG's image_gen.py and keeps Cortex self-contained
        let python_script = format!(
            r#"
import os, sys, json
os.environ["GEMINI_API_KEY"] = "{api_key}"
os.environ["GOOGLE_API_KEY"] = "{api_key}"

try:
    from google import genai
    from google.genai import types
except ImportError:
    print(json.dumps({{"error": "Install: pip install google-genai"}}))
    sys.exit(1)

client = genai.Client(api_key="{api_key}")

try:
    response = client.models.generate_content(
        model="gemini-3-pro-image-preview",
        contents="""{enhanced_prompt_escaped}""",
        config=types.GenerateContentConfig(
            response_modalities=['IMAGE'],
            image_config=types.ImageConfig(
                aspect_ratio="{aspect_ratio}",
                image_size="2K"
            )
        )
    )

    for part in response.parts:
        if hasattr(part, 'inline_data') and part.inline_data is not None:
            image = part.as_image()
            image.save("{output_escaped}")
            size = os.path.getsize("{output_escaped}")
            print(json.dumps({{"success": True, "path": "{output_escaped}", "size": size}}))
            sys.exit(0)

    print(json.dumps({{"error": "No image in response"}}))
    sys.exit(1)

except Exception as e:
    print(json.dumps({{"error": str(e)}}))
    sys.exit(1)
"#,
            api_key = api_key,
            enhanced_prompt_escaped = enhanced_prompt.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n"),
            aspect_ratio = aspect_ratio,
            output_escaped = output_str.replace('\\', "\\\\").replace('"', "\\\""),
        );

        // Find the Python venv
        let python = self.project_root.join(".venv").join("bin").join("python3");
        let python_str = if python.exists() {
            python.to_string_lossy().to_string()
        } else {
            "python3".to_string()
        };

        debug!(
            prompt = %enhanced_prompt,
            aspect = %aspect_ratio,
            output = %output_str,
            "ImageGen: generating image"
        );

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(120),
            Command::new(&python_str)
                .arg("-c")
                .arg(&python_script)
                .output(),
        )
        .await
        .map_err(|_| "Image generation timed out after 120s".to_string())?
        .map_err(|e| format!("Failed to run python: {e}"))?;

        let stdout = String::from_utf8_lossy(&result.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&result.stderr).trim().to_string();

        if !stderr.is_empty() {
            debug!(stderr = %stderr, "ImageGen python stderr");
        }

        // Parse JSON output
        let parsed: serde_json::Value = serde_json::from_str(&stdout)
            .map_err(|_| format!("Failed to parse output: {stdout}"))?;

        if let Some(error) = parsed.get("error").and_then(|v| v.as_str()) {
            return Err(error.to_string());
        }

        let path = parsed.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'path' in response".to_string())?
            .to_string();

        let size = parsed.get("size")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        info!(
            path = %path,
            size_kb = size / 1024,
            aspect = %aspect_ratio,
            "ImageGen: image generated"
        );

        Ok((path, size))
    }
}

#[async_trait]
impl ToolInterceptor for ImageGenInterceptor {
    fn schemas(&self) -> Vec<ToolSchema> {
        vec![
            ToolSchema {
                tool_type: "function".into(),
                function: FunctionSchema {
                    name: "generate_image".into(),
                    description: "Generate an image using Gemini (Imagen). Creates PNG images \
                        from text descriptions. Use for blog hero images, infographics, \
                        diagrams, social media graphics, and visual content. \
                        Gemini excels at text in images, diagrams, and structured layouts.".into(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "prompt": {
                                "type": "string",
                                "description": "Detailed description of the image to generate. Be specific about composition, colors, text to include, and layout."
                            },
                            "aspect_ratio": {
                                "type": "string",
                                "enum": ["1:1", "16:9", "9:16", "4:3", "3:2", "21:9"],
                                "description": "Aspect ratio. Use 16:9 for hero/banner, 1:1 for social media, 4:3 for cards. Default: 16:9."
                            },
                            "style": {
                                "type": "string",
                                "description": "Style preset: 'cortex' (dark+orange machine aesthetic, DEFAULT), 'cyberpunk', 'minimal', 'professional', 'organic', 'infographic'. Or provide a custom style description."
                            },
                            "filename": {
                                "type": "string",
                                "description": "Output filename (without path). Example: 'day-800-hero.png'. Defaults to 'generated-{timestamp}.png'."
                            }
                        },
                        "required": ["prompt"]
                    }),
                },
            },
            ToolSchema {
                tool_type: "function".into(),
                function: FunctionSchema {
                    name: "image_styles".into(),
                    description: "List available image generation style presets and their descriptions.".into(),
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
            "generate_image" => {
                let prompt = match args.get("prompt").and_then(|v| v.as_str()) {
                    Some(p) if !p.is_empty() => p,
                    _ => return Some(ToolResult::err("Missing required argument: prompt")),
                };

                let aspect_ratio = args.get("aspect_ratio")
                    .and_then(|v| v.as_str())
                    .unwrap_or("16:9");

                // Validate aspect ratio
                let valid_ratios = ["1:1", "16:9", "9:16", "4:3", "3:2", "21:9"];
                if !valid_ratios.contains(&aspect_ratio) {
                    return Some(ToolResult::err(format!(
                        "Invalid aspect_ratio '{}'. Valid: {:?}",
                        aspect_ratio, valid_ratios
                    )));
                }

                let style = args.get("style").and_then(|v| v.as_str());

                let filename = args.get("filename")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| {
                        let ts = chrono::Utc::now().format("%Y%m%d-%H%M%S");
                        format!("generated-{ts}.png")
                    });

                match self.generate(prompt, aspect_ratio, style, &filename).await {
                    Ok((path, size)) => {
                        let style_name = style.unwrap_or("(none)");
                        Some(ToolResult::ok(format!(
                            "Image generated successfully.\n\
                             Path: {path}\n\
                             Aspect: {aspect_ratio}\n\
                             Style: {style_name}\n\
                             Size: {}KB\n\
                             Prompt: {}",
                            size / 1024,
                            safe_truncate(prompt, 100),
                        )))
                    }
                    Err(e) => {
                        warn!(error = %e, "generate_image failed");
                        Some(ToolResult::err(format!("Image generation failed: {e}")))
                    }
                }
            }

            "image_styles" => {
                let mut out = String::from("Available image style presets:\n\n");
                for (name, desc) in STYLES {
                    out.push_str(&format!("- **{name}**: {desc}\n\n"));
                }
                out.push_str("Use the style name in the `style` parameter of generate_image.\n");
                out.push_str("Default for Cortex content: 'cortex' (dark bg, orange accent, machine aesthetic).\n");
                out.push_str("You can also pass a custom style description string.");
                Some(ToolResult::ok(out))
            }

            // Not an image gen tool — pass through to next handler.
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_llm::think_loop::ToolInterceptor;

    #[test]
    fn image_gen_interceptor_schemas() {
        let interceptor = ImageGenInterceptor::new(
            Path::new("/tmp/images"),
            Path::new("/tmp/project"),
        );
        let schemas = interceptor.schemas();

        assert_eq!(schemas.len(), 2);

        let names: Vec<&str> = schemas.iter().map(|s| s.function.name.as_str()).collect();
        assert!(names.contains(&"generate_image"));
        assert!(names.contains(&"image_styles"));

        for schema in &schemas {
            assert_eq!(schema.tool_type, "function");
        }
    }

    #[tokio::test]
    async fn image_gen_interceptor_ignores_unknown() {
        let interceptor = ImageGenInterceptor::new(
            Path::new("/tmp/images"),
            Path::new("/tmp/project"),
        );

        assert!(interceptor.handle("bash", &serde_json::json!({})).await.is_none());
        assert!(interceptor.handle("tts_speak", &serde_json::json!({})).await.is_none());
        assert!(interceptor.handle("unknown", &serde_json::json!({})).await.is_none());
    }

    #[tokio::test]
    async fn image_gen_interceptor_validates_required_args() {
        let interceptor = ImageGenInterceptor::new(
            Path::new("/tmp/images"),
            Path::new("/tmp/project"),
        );

        // generate_image with no prompt
        let result = interceptor.handle("generate_image", &serde_json::json!({})).await;
        assert!(result.is_some());
        let r = result.unwrap();
        assert!(!r.success);
        assert!(r.error.unwrap().contains("prompt"));
    }

    #[tokio::test]
    async fn image_gen_interceptor_validates_aspect_ratio() {
        let interceptor = ImageGenInterceptor::new(
            Path::new("/tmp/images"),
            Path::new("/tmp/project"),
        );

        let result = interceptor.handle("generate_image", &serde_json::json!({
            "prompt": "test image",
            "aspect_ratio": "5:3"
        })).await;
        assert!(result.is_some());
        let r = result.unwrap();
        assert!(!r.success);
        assert!(r.error.unwrap().contains("Invalid aspect_ratio"));
    }

    #[tokio::test]
    async fn image_gen_interceptor_styles_list() {
        let interceptor = ImageGenInterceptor::new(
            Path::new("/tmp/images"),
            Path::new("/tmp/project"),
        );

        let result = interceptor.handle("image_styles", &serde_json::json!({})).await;
        assert!(result.is_some());
        let r = result.unwrap();
        assert!(r.success);
        assert!(r.output.contains("cortex"));
        assert!(r.output.contains("cyberpunk"));
        assert!(r.output.contains("infographic"));
    }

    #[test]
    fn style_resolution() {
        assert!(ImageGenInterceptor::resolve_style("cortex").is_some());
        assert!(ImageGenInterceptor::resolve_style("CORTEX").is_some());
        assert!(ImageGenInterceptor::resolve_style("cyberpunk").is_some());
        assert!(ImageGenInterceptor::resolve_style("unknown_style").is_none());
    }

    #[test]
    fn image_gen_schema_parameters_are_valid_json_schema() {
        let interceptor = ImageGenInterceptor::new(
            Path::new("/tmp/images"),
            Path::new("/tmp/project"),
        );
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
