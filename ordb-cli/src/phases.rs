use crate::api_client;
use crate::cli::Cli;
use crate::db;
use crate::scanner;
use crate::metadata;
use crate::enrichment;
use rusqlite::Connection;
use reqwest::Client;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

pub async fn run_pipeline(args: &Cli, conn: &Connection) -> anyhow::Result<()> {
    let client = Client::new();
    let api_url = "http://127.0.0.1:8000"; // the default url for uvicorn
    
    // Phase 1: Warm-up
    println!("Phase 1: Warm-up");
    api_client::warm_up(&client, api_url).await?;
    println!("AI Microservice is ready.");
    
    // Phase 2: Scanning & Deduplication
    println!("Phase 2: Scanning & Deduplication");
    // Store source directories for commit/purge
    for src in &args.source {
        let abs = src.canonicalize().unwrap_or(src.clone());
        db::insert_source(conn, &abs.to_string_lossy())?;
    }
    let mut rx = scanner::scan_directories(args.source.clone()).await;
    while let Some(file_info) = rx.recv().await {
        // Insert as PENDIENTE (returns None if already exists from a previous run)
        let id = db::insert_file(
            conn,
            &file_info.path,
            file_info.size,
            file_info.mime_type.as_deref(),
            file_info.hash.as_deref(),
        )?;

        let id = match id {
            Some(id) => id,
            None => continue, // Already processed in a previous run, skip
        };

        // Files that couldn't be read or are empty → OMITIDO
        if let Some(reason) = &file_info.skip_reason {
            db::update_staging_status(conn, id, "OMITIDO", Some(reason))?;
            continue;
        }

        let hash = file_info.hash.as_deref().unwrap_or("");

        // Deduplicate
        if let Some(primary_id) = db::find_primary_by_hash(conn, hash)? {
            db::update_status(conn, id, "DUPLICADO_EXACTO", Some(primary_id))?;
        } else {
            db::update_status(conn, id, "PRIMARIO", None)?;
        }
    }
    println!("Phase 2 Complete.");
    
    // Phase 3: Enrichment & Classification
    println!("Phase 3: Enrichment & Classification");
    let primary_files = db::get_primary_files(conn)?;
    let mut used_paths = HashMap::new();
    let dest_base = args.destination.as_ref()
        .ok_or_else(|| anyhow::anyhow!("--destination is required for the pipeline"))?;

    let mut images_to_classify = Vec::new();
    
    for file in &primary_files {
        let is_image = file.mime_type.as_deref().unwrap_or("").starts_with("image/");
        if is_image {
            images_to_classify.push(file.source_path.clone());
        }
    }
    
    // Process classification in batches
    let mut classification_results = HashMap::new();
    for chunk in images_to_classify.chunks(args.batch_size) {
        if let Ok(results) = api_client::classify_batch(&client, api_url, chunk.to_vec()).await {
            for res in results {
                let cat = if res.confidence < args.confidence_threshold {
                    "Desconocido".to_string()
                } else {
                    res.category
                };
                classification_results.insert(res.path, (cat, res.confidence));
            }
        } else {
            eprintln!("Warning: Failed to classify a batch of images.");
        }
    }

    for file in primary_files {
        let is_image = file.mime_type.as_deref().unwrap_or("").starts_with("image/");
        let is_audio = file.mime_type.as_deref().unwrap_or("").starts_with("audio/");

        let mut date_src = None;
        let mut date_val = None;
        let mut artist = None;
        let mut album = None;
        let mut category = "Otros".to_string();
        let mut confidence = None;

        if is_image {
            let d_info = metadata::extract_date(&file.source_path);
            date_src = Some(d_info.source);
            date_val = d_info.value;
            
            if let Some((cat, conf)) = classification_results.get(&file.source_path) {
                category = cat.clone();
                confidence = Some(*conf);
            } else {
                category = "Desconocido".to_string();
            }
        } else if is_audio {
            let m_info = metadata::extract_music_tags(&file.source_path);
            artist = m_info.artist;
            album = m_info.album;
            category = "Musica".to_string();
        }

        let dest_path_str = enrichment::project_dest_path(
            dest_base,
            &file.source_path,
            file.mime_type.as_deref(),
            &category,
            date_val.as_deref(),
            date_src.as_deref(),
            artist.as_deref(),
            album.as_deref(),
            &file.blake3_hash,
            &mut used_paths,
        );

        db::update_enrichment(
            conn, 
            file.id, 
            Some(&category), 
            confidence, 
            date_src.as_deref(), 
            date_val.as_deref(), 
            artist.as_deref(), 
            album.as_deref(), 
            &dest_path_str
        )?;
    }
    println!("Phase 3 Complete.");
    
    // Phase 4: Staging
    println!("Phase 4: Staging");
    if args.dry_run {
        println!("Dry run enabled. Skipping physical file copies.");
        // Export projected paths report
        let report_path = "dry_run_report.txt";
        let staging_files = db::get_staging_files(conn)?;
        let mut lines = Vec::new();
        lines.push(format!("{:<80} → {}", "ORIGEN", "DESTINO"));
        lines.push("-".repeat(160));
        for file in &staging_files {
            if let Some(dest) = &file.dest_path {
                lines.push(format!("{:<80} → {}", file.source_path, dest));
            }
        }
        lines.push(String::new());
        lines.push(format!("Total archivos proyectados: {}", staging_files.len()));
        std::fs::write(report_path, lines.join("\n"))?;
        println!("Reporte exportado a: {}", report_path);
    } else {
        let staging_files = db::get_staging_files(conn)?;
        for file in staging_files {
            if let Some(dest) = file.dest_path {
                let dest_path = Path::new(&dest);
                if let Some(parent) = dest_path.parent() {
                    if let Err(e) = std::fs::create_dir_all(parent) {
                        db::update_staging_status(conn, file.id, "ERROR", Some(&e.to_string()))?;
                        continue;
                    }
                }
                match std::fs::copy(&file.source_path, &dest_path) {
                    Ok(_) => {
                        db::update_staging_status(conn, file.id, "COMPLETADO", None)?;
                    }
                    Err(e) => {
                        db::update_staging_status(conn, file.id, "ERROR", Some(&e.to_string()))?;
                    }
                }
            }
        }
    }
    println!("Phase 4 Complete.");
    
    // Final Report
    let mut stmt = conn.prepare("SELECT status, count(*) FROM files GROUP BY status")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    })?;
    
    println!("\n=== Reporte Final ===");
    for row in rows.flatten() {
        println!("{}: {}", row.0, row.1);
    }
    println!("=====================\n");
    
    Ok(())
}

pub fn commit(conn: &Connection) -> anyhow::Result<()> {
    let sources = db::get_sources(conn)?;
    if sources.is_empty() {
        println!("No source directories recorded. Run the pipeline first.");
        return Ok(());
    }

    for source in sources {
        let dir = Path::new(&source);
        if !dir.exists() {
            println!("Source {:?} no longer exists, skipping.", dir);
            continue;
        }
        let parent = dir.parent().unwrap_or(Path::new("."));
        let trash_dir = parent.join("_trash_organizador");
        if !trash_dir.exists() {
            std::fs::create_dir_all(&trash_dir)?;
        }
        let target = trash_dir.join(dir.file_name().unwrap_or_default());

        println!("Moving {:?} to {:?}", dir, target);
        if let Err(_) = std::fs::rename(&dir, &target) {
            println!("rename failed (cross-device?), falling back to copy + delete...");
            copy_dir_recursive(&dir, &target)?;
            std::fs::remove_dir_all(&dir)?;
        }
    }
    println!("Commit successfully completed.");
    Ok(())
}

pub fn rollback(conn: &Connection) -> anyhow::Result<()> {
    let mut stmt = conn.prepare("SELECT DISTINCT dest_path FROM files WHERE dest_path IS NOT NULL")?;
    let paths: rusqlite::Result<Vec<String>> = stmt.query_map([], |row| row.get(0))?.collect();
    let paths = paths?;

    let mut top_level_dest = std::collections::HashSet::new();
    for p in paths {
        if let Some(top) = extract_top_level_dir(&p) {
            top_level_dest.insert(top);
        }
    }

    if top_level_dest.is_empty() {
        println!("No destination directories found to rollback.");
        return Ok(());
    }

    println!("\nSe eliminarán los siguientes directorios de destino:");
    for dir in &top_level_dest {
        println!("  - {:?}", dir);
    }
    print!("\n¿Continuar con el rollback? (s/n): ");
    std::io::Write::flush(&mut std::io::stdout())?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if input.trim().to_lowercase() != "s" {
        println!("Rollback cancelado.");
        return Ok(());
    }

    for dir in top_level_dest {
        println!("Removing destination dir: {:?}", dir);
        if dir.exists() {
            std::fs::remove_dir_all(&dir)?;
        }
    }

    conn.execute("UPDATE files SET status = 'PENDIENTE', error_msg = NULL", [])?;
    println!("Rollback successfully completed.");
    Ok(())
}

pub fn purge(conn: &Connection, force: bool) -> anyhow::Result<()> {
    if !force {
        anyhow::bail!("Purge requires --force flag.");
    }
    let sources = db::get_sources(conn)?;
    let mut trash_dirs = std::collections::HashSet::new();
    for source in sources {
        let dir = Path::new(&source);
        let parent = dir.parent().unwrap_or(Path::new("."));
        trash_dirs.insert(parent.join("_trash_organizador"));
    }

    for dir in trash_dirs {
        println!("Purging trash dir: {:?}", dir);
        if dir.exists() {
            std::fs::remove_dir_all(&dir)?;
        }
    }
    println!("Purge successfully completed.");
    Ok(())
}

/// Extract the first meaningful directory from a path.
/// Works with both absolute (C:\Users\...) and relative (test_data\...) paths.
fn extract_top_level_dir(path_str: &str) -> Option<PathBuf> {
    let path = Path::new(path_str);
    let mut components = path.components();
    let mut result = PathBuf::new();

    // Consume prefix and root components (for absolute paths)
    for comp in components.by_ref() {
        match comp {
            std::path::Component::Prefix(_) | std::path::Component::RootDir => {
                result.push(comp);
            }
            std::path::Component::Normal(name) => {
                result.push(name);
                return Some(result);
            }
            _ => {}
        }
    }
    None
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in walkdir::WalkDir::new(src) {
        let entry = entry?;
        let rel = entry.path().strip_prefix(src)?;
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target)?;
        } else {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}
