# AI-Talkie-the-Toaster

Voice AI toaster. A board captures mic audio and plays speaker audio; a **relay**
server bridges the board to the **Google Gemini Live API** and dispatches Gemini
**tool calls** to relays on the board.

## Why a relay
Gemini Live is `wss://` + TLS + JSON + base64 PCM — too heavy for the board in v1.
The relay terminates all of that and speaks a dumb protocol to the device.
Full rationale + firmware research: **EMBASSY-RESEARCH.md**.

## Workspace
Cargo workspace. Members:
- `crates/relay` — bin `toaster-relay`. axum/tokio server. **Built.**
- `crates/firmware` — embassy ESP firmware. **Not scaffolded yet** (hardware undecided).

## Relay crate (`crates/relay`)
- `src/main.rs` — starts HTTP `/health` + raw-TCP device listener.
- `src/config.rs` — env config (`HTTP_ADDR`, `DEVICE_ADDR`, `GEMINI_API_KEY`).
- `src/device.rs` — dumb board protocol: `Frame` (AudioIn/AudioOut/RelaySet), `handle`. **Bridge is a stub.**
- `src/gemini.rs` — Gemini Live adapter (relay-side wss). **Stub.**

## Transport
- Board ↔ relay: **plain TCP**, frames `u32 len` + `u8 kind` + payload (AudioIn / AudioOut / RelaySet).
- Relay ↔ Gemini: `wss://`, JSON + base64 PCM, holds the API key.

## Run
`GEMINI_API_KEY=… cargo run -p toaster-relay`

## Conventions
- **Conventional Commits** for all commit messages (`feat:`, `fix:`, `chore:`, `docs:`, …).
- No silent failure — errors bubble via `anyhow`; degraded paths log `warn`.
- Gemini Live: https://ai.google.dev/gemini-api/docs/live-api
