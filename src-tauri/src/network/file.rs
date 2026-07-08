use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::fs::OpenOptions;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, SeekFrom};
use tokio::net::TcpStream;
use uuid::Uuid;

use crate::crypto::sha;
use crate::network::framing;
use crate::protocol::envelope::Envelope;
use crate::protocol::file::{FileChunkPayload, FileRequestPayload, FileResponsePayload};
use crate::state::AppState;

/// Handles an incoming file request on the sender side
pub async fn handle_file_request(
    stream: TcpStream,
    payload_val: serde_json::Value,
    state: Arc<AppState>,
    app_handle: AppHandle,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let req_res: Result<FileRequestPayload, _> = serde_json::from_value(payload_val.clone());
    let file_id = req_res.as_ref().map(|r| r.file_id).unwrap_or_default();

    match handle_file_request_impl(stream, payload_val, state, app_handle.clone()).await {
        Ok(()) => Ok(()),
        Err(e) => {
            if !file_id.is_nil() {
                let _ = app_handle.emit(
                    "upload-error",
                    serde_json::json!({
                        "fileId": file_id,
                        "error": e.to_string()
                    }),
                );
            }
            Err(e)
        }
    }
}

async fn handle_file_request_impl(
    mut stream: TcpStream,
    payload_val: serde_json::Value,
    state: Arc<AppState>,
    app_handle: AppHandle,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _ = stream.set_nodelay(true);
    let req: FileRequestPayload = serde_json::from_value(payload_val)?;

    // 1. Look up file in shared files
    let shared_file = {
        let files = state.shared_files.read().await;
        files.get(&req.file_id).cloned()
    };

    if shared_file.is_none() {
        let resp = FileResponsePayload {
            status: "error".to_string(),
            start_offset: 0,
            file_size: 0,
            message: "File not found or no longer shared".to_string(),
        };
        let envelope = Envelope::new("file_response", &resp)?;
        let frame_bytes = envelope.to_encrypted_bytes()?;
        framing::write_frame(&mut stream, &frame_bytes).await?;
        return Err("File not found or no longer shared".into());
    }

    let file_info = shared_file.unwrap();

    // 2. Open file and seek
    let mut file = match tokio::fs::File::open(&file_info.file_path).await {
        Ok(f) => f,
        Err(e) => {
            let resp = FileResponsePayload {
                status: "error".to_string(),
                start_offset: 0,
                file_size: 0,
                message: format!("Failed to open file: {}", e),
            };
            let envelope = Envelope::new("file_response", &resp)?;
            let frame_bytes = envelope.to_encrypted_bytes()?;
            let _ = framing::write_frame(&mut stream, &frame_bytes).await;
            return Err(e.into());
        }
    };

    let file_len = file_info.size;
    let start_offset = req.start_offset;

    if start_offset > file_len {
        let resp = FileResponsePayload {
            status: "error".to_string(),
            start_offset: 0,
            file_size: file_len,
            message: "Invalid start offset".to_string(),
        };
        let envelope = Envelope::new("file_response", &resp)?;
        let frame_bytes = envelope.to_encrypted_bytes()?;
        framing::write_frame(&mut stream, &frame_bytes).await?;
        return Err("Invalid start offset".into());
    }

    file.seek(SeekFrom::Start(start_offset)).await?;

    // 3. Send successful response
    let resp = FileResponsePayload {
        status: "ok".to_string(),
        start_offset,
        file_size: file_len,
        message: "".to_string(),
    };
    let envelope = Envelope::new("file_response", &resp)?;
    let frame_bytes = envelope.to_encrypted_bytes()?;
    framing::write_frame(&mut stream, &frame_bytes).await?;

    // 4. Stream file chunks (4MB each)
    const CHUNK_SIZE: usize = 4 * 1024 * 1024;
    let mut buffer = vec![0u8; CHUNK_SIZE];
    let mut current_offset = start_offset;
    let mut chunk_index = (start_offset / CHUNK_SIZE as u64) as u64;

    // Notify upload start
    let _ = app_handle.emit(
        "upload-progress",
        serde_json::json!({
            "fileId": file_info.file_id,
            "fileName": file_info.name,
            "bytesUploaded": current_offset,
            "totalBytes": file_len,
            "status": "uploading"
        }),
    );

    let mut last_emit = std::time::Instant::now();

    loop {
        let count = file.read(&mut buffer).await?;
        if count == 0 {
            break; // EOF
        }

        let chunk_data = &buffer[..count];
        let chunk_sha = sha::compute_sha256_bytes(chunk_data);

        let chunk_meta = FileChunkPayload {
            file_id: file_info.file_id,
            index: chunk_index,
            offset: current_offset,
            size: count as u32,
            chunk_sha256: chunk_sha,
        };

        // Write chunk header frame
        let chunk_envelope = Envelope::new("file_chunk", &chunk_meta)?;
        let chunk_frame_bytes = chunk_envelope.to_encrypted_bytes()?;
        framing::write_frame(&mut stream, &chunk_frame_bytes).await?;

        // Write raw binary payload immediately
        stream.write_all(chunk_data).await?;

        current_offset += count as u64;
        chunk_index += 1;

        // Throttled progress report (150ms)
        if last_emit.elapsed().as_millis() >= 150 {
            let _ = app_handle.emit(
                "upload-progress",
                serde_json::json!({
                    "fileId": file_info.file_id,
                    "fileName": file_info.name,
                    "bytesUploaded": current_offset,
                    "totalBytes": file_len,
                    "status": "uploading"
                }),
            );
            last_emit = std::time::Instant::now();
        }
    }

    stream.flush().await?;

    // Notify final success
    let _ = app_handle.emit(
        "upload-success",
        serde_json::json!({
            "fileId": file_info.file_id,
            "bytesUploaded": file_len,
            "totalBytes": file_len,
            "status": "success"
        }),
    );

    Ok(())
}

/// Starts file download task in the background
pub fn start_download(
    app_handle: AppHandle,
    state: Arc<AppState>,
    peer_id: Uuid,
    file_id: Uuid,
    file_name: String,
    file_size: u64,
    save_path: String,
) {
    tauri::async_runtime::spawn(async move {
        if let Err(e) = download_task(
            app_handle.clone(),
            state,
            peer_id,
            file_id,
            file_name.clone(),
            file_size,
            save_path.clone(),
        )
        .await
        {
            eprintln!("Download task error for {}: {}", file_name, e);
            let _ = app_handle.emit(
                "download-error",
                serde_json::json!({
                    "fileId": file_id,
                    "error": e.to_string()
                }),
            );
        }
    });
}

async fn download_task(
    app_handle: AppHandle,
    state: Arc<AppState>,
    peer_id: Uuid,
    file_id: Uuid,
    file_name: String,
    file_size: u64,
    save_path: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let clean_file_name = PathBuf::from(&file_name)
        .file_name()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid file name"))?
        .to_string_lossy()
        .into_owned();

    let save_path_buf = PathBuf::from(&save_path);
    for component in save_path_buf.components() {
        if let std::path::Component::ParentDir = component {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "Access denied: path traversal attempt",
            )
            .into());
        }
    }

    let mut part_path = save_path_buf.clone();
    let mut part_file_name = part_path.file_name().unwrap_or_default().to_os_string();
    part_file_name.push(".part");
    part_path.set_file_name(part_file_name);

    if let Some(parent) = save_path_buf.parent() {
        if !parent.exists() {
            tokio::fs::create_dir_all(parent).await?;
        }
    }

    // 1. Determine local offset if the .part file already exists
    let mut start_offset = 0;
    if tokio::fs::metadata(&part_path).await.is_ok() {
        if let Ok(meta) = tokio::fs::metadata(&part_path).await {
            let len = meta.len();
            if len < file_size {
                start_offset = len;
            } else if len == file_size {
                // Already downloaded, just rename
                tokio::fs::rename(&part_path, &save_path).await?;
                let _ = app_handle.emit(
                    "download-success",
                    serde_json::json!({
                        "fileId": file_id,
                        "savePath": save_path
                    }),
                );
                return Ok(());
            } else {
                // If local size is larger for some reason, truncate it
                let _ = tokio::fs::remove_file(&part_path).await;
            }
        }
    }

    // 2. Resolve peer details
    let (peer_ip, peer_port) = {
        let peers = state.online_peers.read().await;
        if let Some(info) = peers.get(&peer_id) {
            (info.ip.clone(), info.payload.tcp_port)
        } else {
            return Err("Sender is offline or not found".into());
        }
    };

    // 3. Connect to sender TCP port
    let dest_addr = format!("{}:{}", peer_ip, peer_port);
    let mut stream = TcpStream::connect(&dest_addr).await?;
    let _ = stream.set_nodelay(true);

    // 4. Send File Request
    let req = FileRequestPayload {
        file_id,
        start_offset,
    };
    let envelope = Envelope::new("file_request", &req)?;
    let frame_bytes = envelope.to_encrypted_bytes()?;
    framing::write_frame(&mut stream, &frame_bytes).await?;

    // 5. Read Handshake Response
    let resp_bytes = framing::read_frame(&mut stream).await?;
    let resp_envelope = Envelope::from_encrypted_bytes(&resp_bytes)?;

    if resp_envelope.v != 1 || resp_envelope.msg_type != "file_response" {
        return Err("Invalid file handshake response".into());
    }

    let response: FileResponsePayload = serde_json::from_value(resp_envelope.payload)?;
    if response.status != "ok" {
        return Err(format!("Sender rejected transfer: {}", response.message).into());
    }

    // 6. Open local part file for seeking/writing
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&part_path)
        .await?;

    file.seek(SeekFrom::Start(start_offset)).await?;

    let mut bytes_downloaded = start_offset;

    // Notify frontend download started/resumed
    let _ = app_handle.emit(
        "download-progress",
        serde_json::json!({
            "fileId": file_id,
            "fileName": clean_file_name.clone(),
            "bytesDownloaded": bytes_downloaded,
            "totalBytes": file_size,
            "status": "downloading"
        }),
    );

    let mut last_emit = std::time::Instant::now();

    // 7. Receive chunks
    while bytes_downloaded < file_size {
        // Read metadata frame for chunk
        let chunk_meta_bytes = framing::read_frame(&mut stream).await?;
        let chunk_envelope = Envelope::from_encrypted_bytes(&chunk_meta_bytes)?;

        if chunk_envelope.v != 1 || chunk_envelope.msg_type != "file_chunk" {
            return Err("Invalid file chunk header".into());
        }

        let chunk_meta: FileChunkPayload = serde_json::from_value(chunk_envelope.payload)?;

        // Read raw binary bytes
        let mut chunk_data = vec![0u8; chunk_meta.size as usize];
        stream.read_exact(&mut chunk_data).await?;

        // Verify SHA-256 of the chunk
        let computed_hash = sha::compute_sha256_bytes(&chunk_data);
        if computed_hash != chunk_meta.chunk_sha256 {
            return Err("File chunk corruption detected (SHA-256 mismatch)".into());
        }

        // Write to local file: avoid seek if write is contiguous
        let current_pos = file.seek(SeekFrom::Current(0)).await.unwrap_or(0);
        if chunk_meta.offset != current_pos {
            file.seek(SeekFrom::Start(chunk_meta.offset)).await?;
        }
        file.write_all(&chunk_data).await?;

        bytes_downloaded += chunk_meta.size as u64;

        // Throttled progress report (150ms)
        if last_emit.elapsed().as_millis() >= 150 {
            let _ = app_handle.emit(
                "download-progress",
                serde_json::json!({
                    "fileId": file_id,
                    "fileName": clean_file_name.clone(),
                    "bytesDownloaded": bytes_downloaded,
                    "totalBytes": file_size,
                    "status": "downloading"
                }),
            );
            last_emit = std::time::Instant::now();
        }
    }

    file.flush().await?;
    drop(file);

    // Emit final progress showing 100% downloaded
    let _ = app_handle.emit(
        "download-progress",
        serde_json::json!({
            "fileId": file_id,
            "fileName": clean_file_name.clone(),
            "bytesDownloaded": file_size,
            "totalBytes": file_size,
            "status": "downloading"
        }),
    );

    // 8. Rename part file to original path
    tokio::fs::rename(&part_path, &save_path).await?;

    // Emit final success
    let _ = app_handle.emit(
        "download-success",
        serde_json::json!({
            "fileId": file_id,
            "savePath": save_path
        }),
    );

    Ok(())
}
