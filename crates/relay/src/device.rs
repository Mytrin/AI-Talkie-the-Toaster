//! Dumb board protocol over plain TCP. The board can't do TLS/WebSocket, so the
//! relay speaks length-prefixed frames: `u32 len (BE)` + `u8 kind` + payload.

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[derive(Debug)]
pub enum Frame {
    /// Mic PCM from the board (16-bit LE, 16 kHz mono).
    AudioIn(Vec<u8>),
    /// Speaker PCM to the board (16-bit LE, 24 kHz mono).
    AudioOut(Vec<u8>),
    /// Set relay `id` on/off (Gemini tool call result).
    RelaySet { id: u8, on: bool },
}

impl Frame {
    fn kind(&self) -> u8 {
        match self {
            Frame::AudioIn(_) => 0x01,
            Frame::AudioOut(_) => 0x02,
            Frame::RelaySet { .. } => 0x03,
        }
    }

    fn payload(&self) -> Vec<u8> {
        match self {
            Frame::AudioIn(p) | Frame::AudioOut(p) => p.clone(),
            Frame::RelaySet { id, on } => vec![*id, *on as u8],
        }
    }

    pub async fn read(stream: &mut TcpStream) -> anyhow::Result<Frame> {
        let len = stream.read_u32().await? as usize;
        anyhow::ensure!(len >= 1, "frame length {len} too short (no kind byte)");
        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf).await?;
        let (kind, payload) = (buf[0], buf[1..].to_vec());
        match kind {
            0x01 => Ok(Frame::AudioIn(payload)),
            0x02 => Ok(Frame::AudioOut(payload)),
            0x03 => {
                anyhow::ensure!(payload.len() == 2, "RelaySet payload must be 2 bytes");
                Ok(Frame::RelaySet {
                    id: payload[0],
                    on: payload[1] != 0,
                })
            }
            other => anyhow::bail!("unknown frame kind {other:#x}"),
        }
    }

    pub async fn write(&self, stream: &mut TcpStream) -> anyhow::Result<()> {
        let payload = self.payload();
        stream.write_u32((payload.len() + 1) as u32).await?;
        stream.write_u8(self.kind()).await?;
        stream.write_all(&payload).await?;
        stream.flush().await?;
        Ok(())
    }
}

/// Bridge one board connection to a Gemini Live session.
pub async fn handle(mut stream: TcpStream, _gemini: &crate::gemini::Client) -> anyhow::Result<()> {
    let peer = stream.peer_addr()?;
    tracing::info!(%peer, "board connected");
    // TODO: open Gemini session, pump AudioIn->Gemini and Gemini->AudioOut,
    // emit RelaySet on tool calls. Fail loud — never drop frames silently.
    let _ = Frame::AudioOut(Vec::new()).write(&mut stream).await;
    anyhow::bail!("board<->gemini bridge not implemented")
}
