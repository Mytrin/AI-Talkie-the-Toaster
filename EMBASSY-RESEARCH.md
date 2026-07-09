# Embassy / ESP firmware research

Can an ESP-class board do mic + speaker in embassy Rust? What networking? Do we need the relay?
Findings as of 2026-07-09. **No hardware chosen yet, no firmware scaffolded.**

## TL;DR
- Audio: **yes** — full-duplex I2S mic + speaker, async, stable on released esp-hal today.
- Networking: **yes** — esp-wifi + embassy-net (TCP/UDP/DNS/DHCP).
- Relay: **keep it.** Gemini Live is wss+TLS+JSON+base64 — too heavy/fragile on-device for v1. Direct-to-Gemini is a v2 (ESP32-S3 + PSRAM only).

## Audio — production-usable today
- HAL: `esp-hal` 1.1.0 (no_std, async, embassy-native). Module `esp_hal::i2s::master`.
- I2S mic RX (e.g. INMP441): async DMA, stable. Example `qa-test/src/bin/embassy_i2s_read.rs`.
- I2S speaker TX (e.g. MAX98357A): async DMA, stable. Example `embassy_i2s_sound.rs`.
- Full-duplex: ESP32 / ESP32-S3 have **2** I2S peripherals → independent mic+speaker. ESP32-C6/C3 have **1** (shared-clock duplex only).
- PDM mic: **bleeding edge** — landed in esp-hal `main` 2026-07-02, not in any release. C6 gives raw bitstream (SW decode needed); S3/ESP32 have HW PDM2PCM. Production PDM today = fall back to `esp-idf-hal` (std). Avoid unless a chosen mic forces it.

## Networking — not the blocker
- `esp-wifi` + `embassy-net` 0.7: async TCP/UDP/DNS/DHCP, WiFi STA. Well-trodden on S3/C6.
- TLS is not in embassy-net; it layers on top of a TCP socket.

## Gemini Live wire protocol (confirmed)
- Transport: **WebSocket over TLS** — `wss://generativelanguage.googleapis.com/ws/...`.
- Auth: API key, or (recommended) ephemeral token as URL query param → minting needs a backend regardless.
- Framing: JSON messages; audio is **base64 PCM inside JSON** (`realtimeInput`).
- Audio: input 16-bit PCM **16 kHz** mono; output 16-bit PCM **24 kHz**. ~640 kbps duplex.

## On-device direct-to-Gemini — feasible but fragile (v2)
Blockers are RAM + stack maturity, not bandwidth or WiFi:
- TLS 1.3 wants ~16 KB × 2 record buffers; coexisting with WS framing + JSON + base64 + audio ring buffers in ~512 KB SRAM is tight → needs PSRAM.
- Every layer is 0.x: `embedded-tls` or `esp-mbedtls` (prefer mbedtls — HW AES/SHA on S3/C6, robust cert chains), `edge-ws` for WebSocket.
- Cert store + SNTP clock required or handshake silently fails.
- Ephemeral-token auth needs a backend anyway.

## Recommended architecture: keep the relay
- **ESP ↔ relay:** dumb protocol over **plain TCP** — length-prefixed raw 16-bit PCM frames + tiny control opcodes. No TLS / JSON / base64 / cert store / SNTP on device. Firmware only needs embassy-net TCP + an I2S loop.
- **relay ↔ Gemini:** `crates/relay` (axum/tokio) terminates `wss://`, does base64+JSON, holds the API key / mints ephemeral tokens.
- Relay is the correct engineering call, not a WiFi workaround.
- v2 (optional): direct `wss://` on ESP32-S3 + PSRAM + `esp-mbedtls` + `edge-ws`, still with a backend for token minting.

## Board guidance (when hardware is picked)
- **ESP32-S3** — best: 2 I2S, PSRAM, HW crypto; only board where v2 direct is realistic.
- **ESP32-C6** — 1 I2S, less RAM, relay mandatory, PDM raw-only.
- **Plain ESP32** — 2 I2S + HW PDM2PCM but older, no native USB.

## Relay scaffold state
`crates/relay` device-facing side is a **raw TCP** listener (`src/device.rs`) speaking the dumb PCM protocol — the ws client stays on the Gemini side only. Both the device bridge and the Gemini session are stubs pending firmware + hardware.

## Sources
- esp-hal: https://github.com/esp-rs/esp-hal (I2S examples under `qa-test/`)
- esp-idf-hal (PDM fallback): https://github.com/esp-rs/esp-idf-hal
- embassy-net: https://docs.embassy.dev/embassy-net/
- TLS: https://docs.rs/embedded-tls · https://github.com/esp-rs/esp-mbedtls
- WebSocket: https://lib.rs/crates/edge-ws
- Gemini Live: https://ai.google.dev/gemini-api/docs/live-api · https://ai.google.dev/api/live
