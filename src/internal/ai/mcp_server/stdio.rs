//! Newline-delimited JSON-RPC transport for `libra agent mcp --stdio`.
//!
//! stdout receives one JSON-RPC message per line. Diagnostics stay off stdout.

use std::io::ErrorKind;

use serde_json::Value;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt};

use super::protocol::{self, InboundLine, MAX_LINE_BYTES, Session};

async fn repository_id() -> String {
    match crate::internal::config::ConfigKv::get_best_effort("libra.repoid").await {
        Ok(Some(entry))
            if !entry.encrypted
                && !entry.value.trim().is_empty()
                && entry.value != "unknown-repo" =>
        {
            entry.value
        }
        _ => "unavailable".to_string(),
    }
}

pub async fn serve<R, W>(reader: R, mut writer: W) -> std::io::Result<()>
where
    R: AsyncBufRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut reader = reader;
    let mut session = Session::with_repository_id(repository_id().await);
    loop {
        let inbound = read_line_capped(&mut reader, MAX_LINE_BYTES).await?;
        let inbound = match inbound {
            None => break,
            Some(line) => line,
        };
        if let Some(response) = protocol::handle_message(&mut session, &inbound) {
            write_message(&mut writer, &response).await?;
        }
    }
    writer.flush().await?;
    Ok(())
}

async fn write_message<W: AsyncWrite + Unpin>(
    writer: &mut W,
    message: &Value,
) -> std::io::Result<()> {
    let mut bytes = serde_json::to_vec(message).map_err(|error| {
        std::io::Error::new(
            ErrorKind::InvalidData,
            format!("failed to encode MCP response: {error}"),
        )
    })?;
    bytes.push(b'\n');
    writer.write_all(&bytes).await?;
    writer.flush().await?;
    Ok(())
}

/// Read one logical line. `Ok(None)` is EOF with no pending bytes.
///
/// A line whose payload exceeds `cap` is discarded through the next newline
/// so the following request can still be served.
async fn read_line_capped<R: AsyncBufRead + Unpin>(
    reader: &mut R,
    cap: usize,
) -> std::io::Result<Option<InboundLine>> {
    let mut buf = Vec::new();
    loop {
        let available = reader.fill_buf().await?;
        if available.is_empty() {
            if buf.is_empty() {
                return Ok(None);
            }
            return Ok(Some(if buf.len() > cap {
                InboundLine::TooLarge
            } else {
                InboundLine::Message(buf)
            }));
        }
        if let Some(index) = available.iter().position(|byte| *byte == b'\n') {
            if buf.len() + index > cap {
                reader.consume(index + 1);
                return Ok(Some(InboundLine::TooLarge));
            }
            buf.extend_from_slice(&available[..index]);
            reader.consume(index + 1);
            if buf.last() == Some(&b'\r') {
                buf.pop();
            }
            if buf.is_empty() {
                return Ok(Some(InboundLine::Empty));
            }
            return Ok(Some(InboundLine::Message(buf)));
        }
        let length = available.len();
        if buf.len() + length > cap {
            reader.consume(length);
            discard_until_newline(reader).await?;
            return Ok(Some(InboundLine::TooLarge));
        }
        buf.extend_from_slice(available);
        reader.consume(length);
    }
}

async fn discard_until_newline<R: AsyncBufRead + Unpin>(reader: &mut R) -> std::io::Result<()> {
    loop {
        let available = reader.fill_buf().await?;
        if available.is_empty() {
            return Ok(());
        }
        if let Some(index) = available.iter().position(|byte| *byte == b'\n') {
            reader.consume(index + 1);
            return Ok(());
        }
        let length = available.len();
        reader.consume(length);
    }
}
