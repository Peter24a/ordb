use std::path::PathBuf;
use std::fs;
use walkdir::WalkDir;
use tokio::sync::mpsc;
use log::info;

pub struct FileInfo {
    pub path: String,
    pub size: u64,
    pub mime_type: Option<String>,
    pub hash: Option<String>,
    pub skip_reason: Option<String>,
}

pub async fn scan_directories(sources: Vec<PathBuf>) -> mpsc::Receiver<FileInfo> {
    let (tx, rx) = mpsc::channel(100);

    tokio::task::spawn_blocking(move || {
        for source in sources {
            info!("Scanning directory: {:?}", source);
            for entry in WalkDir::new(&source).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    let path = match entry.path().canonicalize() {
                        Ok(abs) => abs.to_string_lossy().to_string(),
                        Err(_) => entry.path().to_string_lossy().to_string(),
                    };
                    let size = entry.metadata().map(|m| m.len()).unwrap_or(0);

                    // 0-byte files: emit as skippable
                    if size == 0 {
                        let info = FileInfo {
                            path,
                            size,
                            mime_type: None,
                            hash: None,
                            skip_reason: Some("Archivo vacío (0 bytes)".to_string()),
                        };
                        if tx.blocking_send(info).is_err() { break; }
                        continue;
                    }

                    // Try to open and hash the file
                    let file = match fs::File::open(entry.path()) {
                        Ok(f) => f,
                        Err(e) => {
                            let info = FileInfo {
                                path,
                                size,
                                mime_type: None,
                                hash: None,
                                skip_reason: Some(format!("No se pudo abrir: {}", e)),
                            };
                            if tx.blocking_send(info).is_err() { break; }
                            continue;
                        }
                    };

                    let mut hasher = blake3::Hasher::new();
                    if let Err(e) = std::io::copy(&mut std::io::BufReader::new(file), &mut hasher) {
                        let info = FileInfo {
                            path,
                            size,
                            mime_type: None,
                            hash: None,
                            skip_reason: Some(format!("Error al leer contenido: {}", e)),
                        };
                        if tx.blocking_send(info).is_err() { break; }
                        continue;
                    }

                    let hash = hasher.finalize().to_hex().to_string();
                    let mime_type = infer::get_from_path(entry.path())
                        .ok().flatten().map(|m| m.mime_type().to_string());

                    let info = FileInfo {
                        path,
                        size,
                        mime_type,
                        hash: Some(hash),
                        skip_reason: None,
                    };

                    if tx.blocking_send(info).is_err() { break; }
                }
            }
        }
    });

    rx
}
