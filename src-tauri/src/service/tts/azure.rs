use async_trait::async_trait;

use super::{SynthesizedAudio, TtsProvider, TtsSynthesisRequest};

pub struct AzureTtsProvider;

#[async_trait]
impl TtsProvider for AzureTtsProvider {
    async fn synthesize(
        &self,
        client: &reqwest::Client,
        request: &TtsSynthesisRequest,
    ) -> Result<SynthesizedAudio, String> {
        let region = request
            .region
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "Azure Speech Region 未配置".to_string())?;
        let voice = request
            .voice
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("en-US-JennyNeural");
        let language = request
            .language
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("en-US");
        let rate = normalize_rate(request.rate.as_deref().unwrap_or("0%"));
        let volume = normalize_volume(request.volume.as_deref().unwrap_or("+35%"));
        let endpoint =
            format!("https://{region}.tts.speech.microsoft.com/cognitiveservices/v1");
        let ssml = format!(
            "<speak version='1.0' xml:lang='{language}'><voice name='{voice}'><prosody rate='{rate}' volume='{volume}'>{}</prosody></voice></speak>",
            escape_xml(request.text.trim())
        );

        let response = client
            .post(endpoint)
            .header("Ocp-Apim-Subscription-Key", request.api_key.trim())
            .header("Content-Type", "application/ssml+xml")
            .header(
                "X-Microsoft-OutputFormat",
                "audio-24khz-96kbitrate-mono-mp3",
            )
            .header("User-Agent", "Recall")
            .body(ssml)
            .send()
            .await
            .map_err(|error| format!("Azure TTS 请求失败: {error}"))?;

        let status = response.status();
        if !status.is_success() {
            let detail = response.text().await.unwrap_or_default();
            return Err(format!(
                "Azure TTS 返回 HTTP {}{}",
                status.as_u16(),
                response_detail(&detail)
            ));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|error| format!("读取 Azure TTS 音频失败: {error}"))?;
        if bytes.is_empty() {
            return Err("Azure TTS 返回了空音频".into());
        }

        Ok(SynthesizedAudio {
            bytes: bytes.to_vec(),
            content_type: "audio/mpeg".into(),
            provider: "azure".into(),
            voice: voice.into(),
        })
    }
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn normalize_rate(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.ends_with('%') {
        trimmed.to_string()
    } else {
        format!("{trimmed}%")
    }
}

fn normalize_volume(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.ends_with('%') {
        trimmed.to_string()
    } else {
        format!("{trimmed}%")
    }
}

fn response_detail(detail: &str) -> String {
    let compact = detail.trim().replace('\n', " ");
    if compact.is_empty() {
        String::new()
    } else {
        format!(": {}", compact.chars().take(240).collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use super::{escape_xml, normalize_rate, normalize_volume};

    #[test]
    fn escapes_ssml_text() {
        assert_eq!(escape_xml("A&B < C"), "A&amp;B &lt; C");
    }

    #[test]
    fn normalizes_rate() {
        assert_eq!(normalize_rate("-10"), "-10%");
        assert_eq!(normalize_rate("5%"), "5%");
    }

    #[test]
    fn normalizes_volume() {
        assert_eq!(normalize_volume("+35"), "+35%");
        assert_eq!(normalize_volume("+20%"), "+20%");
    }
}
