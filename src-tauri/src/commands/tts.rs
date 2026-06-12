use base64::{engine::general_purpose::STANDARD, Engine};

use crate::service::tts::{
    provider_for, validate_request, TtsSynthesisRequest, TtsSynthesisResponse,
};

#[tauri::command]
pub async fn synthesize_speech(
    request: TtsSynthesisRequest,
) -> Result<TtsSynthesisResponse, String> {
    validate_request(&request)?;
    let provider = provider_for(&request.provider)?;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|error| format!("创建 TTS HTTP 客户端失败: {error}"))?;
    let audio = provider.synthesize(&client, &request).await?;

    Ok(TtsSynthesisResponse {
        audio_base64: STANDARD.encode(audio.bytes),
        content_type: audio.content_type,
        provider: audio.provider,
        voice: audio.voice,
    })
}
