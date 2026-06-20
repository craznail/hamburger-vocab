use async_trait::async_trait;
use serde::{Deserialize, Serialize};

mod aliyun;
mod azure;

pub use aliyun::AliyunTtsProvider;
pub use azure::AzureTtsProvider;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsSynthesisRequest {
    pub provider: String,
    pub text: String,
    pub api_key: String,
    pub region: Option<String>,
    pub voice: Option<String>,
    pub language: Option<String>,
    pub rate: Option<String>,
    pub volume: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsSynthesisResponse {
    pub audio_base64: String,
    pub content_type: String,
    pub provider: String,
    pub voice: String,
}

pub struct SynthesizedAudio {
    pub bytes: Vec<u8>,
    pub content_type: String,
    pub provider: String,
    pub voice: String,
}

#[async_trait]
pub trait TtsProvider: Send + Sync {
    async fn synthesize(
        &self,
        client: &reqwest::Client,
        request: &TtsSynthesisRequest,
    ) -> Result<SynthesizedAudio, String>;
}

pub fn provider_for(name: &str) -> Result<Box<dyn TtsProvider>, String> {
    match name.trim().to_ascii_lowercase().as_str() {
        "azure" => Ok(Box::new(AzureTtsProvider)),
        "aliyun" | "dashscope" => Ok(Box::new(AliyunTtsProvider)),
        other => Err(format!("不支持的 TTS provider: {other}")),
    }
}

pub fn validate_request(request: &TtsSynthesisRequest) -> Result<(), String> {
    if request.text.trim().is_empty() {
        return Err("TTS 文本不能为空".into());
    }
    if request.text.chars().count() > 1000 {
        return Err("TTS 文本不能超过 1000 个字符".into());
    }
    if request.api_key.trim().is_empty() {
        return Err(format!("{} API Key 未配置", request.provider));
    }
    Ok(())
}
