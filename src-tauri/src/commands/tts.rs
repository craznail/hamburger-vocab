use base64::{engine::general_purpose::STANDARD, Engine};
use tauri::State;

use crate::http::HttpClientState;
use crate::service::tts::{
    provider_for, validate_request, TtsSynthesisRequest, TtsSynthesisResponse,
};

#[tauri::command]
pub async fn synthesize_speech(
    request: TtsSynthesisRequest,
    client_state: State<'_, HttpClientState>,
) -> Result<TtsSynthesisResponse, String> {
    validate_request(&request)?;
    let provider = provider_for(&request.provider)?;
    let audio = provider
        .synthesize(&client_state.client, &request)
        .await?;

    Ok(TtsSynthesisResponse {
        audio_base64: STANDARD.encode(audio.bytes),
        content_type: audio.content_type,
        provider: audio.provider,
        voice: audio.voice,
    })
}
