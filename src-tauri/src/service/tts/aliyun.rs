use async_trait::async_trait;
use serde_json::json;

use super::{SynthesizedAudio, TtsProvider, TtsSynthesisRequest};

const DASHSCOPE_API_URL: &str =
    "https://dashscope.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation";

pub struct AliyunTtsProvider;

#[async_trait]
impl TtsProvider for AliyunTtsProvider {
    async fn synthesize(
        &self,
        client: &reqwest::Client,
        request: &TtsSynthesisRequest,
    ) -> Result<SynthesizedAudio, String> {
        let model = request
            .model
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("qwen3-tts-flash");
        let voice = request
            .voice
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("Jennifer");

        let response = client
            .post(DASHSCOPE_API_URL)
            .bearer_auth(request.api_key.trim())
            .json(&json!({
                "model": model,
                "input": {
                    "text": request.text.trim(),
                    "voice": voice,
                    "language_type": "Auto"
                }
            }))
            .send()
            .await
            .map_err(|error| format!("阿里云 TTS 请求失败: {error}"))?;

        let status = response.status();
        let payload: serde_json::Value = response
            .json()
            .await
            .map_err(|error| format!("解析阿里云 TTS 响应失败: {error}"))?;

        if !status.is_success() {
            let message = payload
                .get("message")
                .and_then(|value| value.as_str())
                .unwrap_or("未知错误");
            return Err(format!(
                "阿里云 TTS 返回 HTTP {}: {message}",
                status.as_u16()
            ));
        }

        let audio_url = payload
            .pointer("/output/audio/url")
            .or_else(|| payload.pointer("/output/results/0/url"))
            .and_then(|value| value.as_str())
            .ok_or_else(|| "阿里云 TTS 响应中未找到音频 URL".to_string())?;

        let audio_response = client
            .get(audio_url)
            .send()
            .await
            .map_err(|error| format!("下载阿里云 TTS 音频失败: {error}"))?;
        let audio_status = audio_response.status();
        if !audio_status.is_success() {
            return Err(format!(
                "下载阿里云 TTS 音频返回 HTTP {}",
                audio_status.as_u16()
            ));
        }
        let content_type = audio_response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("audio/mpeg")
            .to_string();
        let bytes = audio_response
            .bytes()
            .await
            .map_err(|error| format!("读取阿里云 TTS 音频失败: {error}"))?;

        Ok(SynthesizedAudio {
            bytes: bytes.to_vec(),
            content_type,
            provider: "aliyun".into(),
            voice: voice.into(),
        })
    }
}
