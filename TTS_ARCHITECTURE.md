# TTS Architecture

## Flow

1. Pages call `speakWord()` from `src/platform/tts.js`.
2. The frontend resolves the active provider configuration.
3. IndexedDB is checked with a provider-aware cache key.
4. `synthesize_speech` is invoked in the Tauri Rust backend.
5. The backend selects a `TtsProvider`, fetches audio, and returns a normalized response.
6. The frontend decodes and plays the audio.
7. Android system TTS is used only when cloud synthesis fails.

## Providers

- Azure Speech: `src-tauri/src/service/tts/azure.rs`
- Aliyun DashScope: `src-tauri/src/service/tts/aliyun.rs`
- Provider contract and registry: `src-tauri/src/service/tts/mod.rs`

The original frontend Aliyun prototype remains in `src/utils/speech.js` for reference.

## Add A Provider

1. Add a provider module under `src-tauri/src/service/tts/`.
2. Implement the `TtsProvider` trait.
3. Register its id in `provider_for()`.
4. Add provider defaults and fields in `src/platform/tts/config.js`.
5. Add the provider controls to `src/pages/SettingsPage.vue`.

No study, dictation, deck, or flash-card page changes are required.

## Configuration

Copy the relevant names from `.env.example`, or configure the provider in the app:

- Azure: Speech Key, Region, Voice, Language, Rate, Volume
- Aliyun: DashScope API Key, Model, Voice

Settings saved in the app override environment defaults. Audio cache entries include provider,
region, model, voice, language, rate, and volume, so switching voices never reuses incompatible audio.
