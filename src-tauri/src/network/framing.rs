use std::io;

/// Reads a length-prefixed frame from the reader.
/// Frame format: [4 bytes payload length (BigEndian)][JSON payload]
pub async fn read_frame<R>(reader: &mut R) -> io::Result<Vec<u8>>
where
    R: tokio::io::AsyncReadExt + Unpin,
{
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;
    
    // Safety limit to prevent memory exhaustion (e.g., max 10MB JSON metadata)
    if len > 10 * 1024 * 1024 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Frame length {} exceeds maximum limit of 10MB", len),
        ));
    }
    
    let mut payload = vec![0u8; len];
    reader.read_exact(&mut payload).await?;
    Ok(payload)
}

/// Writes a length-prefixed frame to the writer.
pub async fn write_frame<W>(writer: &mut W, payload: &[u8]) -> io::Result<()>
where
    W: tokio::io::AsyncWriteExt + Unpin,
{
    let len = payload.len() as u32;
    writer.write_all(&len.to_be_bytes()).await?;
    writer.write_all(payload).await?;
    writer.flush().await?;
    Ok(())
}
