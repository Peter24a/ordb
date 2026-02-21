use rusqlite::{Connection, Result, params, OptionalExtension};
use std::path::Path;

pub fn init_db<P: AsRef<Path>>(db_path: P) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    
    let schema = include_str!("schema.sql");
    conn.execute_batch(schema)?;
    
    Ok(conn)
}

pub fn insert_source(conn: &Connection, path: &str) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO sources (path) VALUES (?1)",
        params![path],
    )?;
    Ok(())
}

pub fn get_sources(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT path FROM sources")?;
    let sources = stmt.query_map([], |row| row.get(0))?.collect::<Result<Vec<_>>>()?;
    Ok(sources)
}

pub fn insert_file(conn: &Connection, path: &str, size: u64, mime: Option<&str>, hash: Option<&str>) -> Result<Option<i64>> {
    let rows = conn.execute(
        "INSERT OR IGNORE INTO files (source_path, file_size, mime_type, blake3_hash, status)
         VALUES (?1, ?2, ?3, ?4, 'PENDIENTE')",
        params![path, size, mime, hash],
    )?;
    if rows == 0 {
        // Already existed (resume case), return None to signal skip
        Ok(None)
    } else {
        Ok(Some(conn.last_insert_rowid()))
    }
}

pub fn find_primary_by_hash(conn: &Connection, hash: &str) -> Result<Option<i64>> {
    conn.query_row(
        "SELECT id FROM files WHERE blake3_hash = ?1 AND status = 'PRIMARIO' LIMIT 1",
        params![hash],
        |row| row.get(0),
    ).optional()
}

pub fn update_status(conn: &Connection, id: i64, status: &str, primary_id: Option<i64>) -> Result<()> {
    conn.execute(
        "UPDATE files SET status = ?1, primary_id = ?2 WHERE id = ?3",
        params![status, primary_id, id],
    )?;
    Ok(())
}

pub struct PrimaryFile {
    pub id: i64,
    pub source_path: String,
    pub mime_type: Option<String>,
    pub blake3_hash: String,
}

pub fn get_primary_files(conn: &Connection) -> Result<Vec<PrimaryFile>> {
    let mut stmt = conn.prepare("SELECT id, source_path, mime_type, blake3_hash FROM files WHERE status = 'PRIMARIO'")?;
    let files = stmt.query_map([], |row| {
        Ok(PrimaryFile {
            id: row.get(0)?,
            source_path: row.get(1)?,
            mime_type: row.get(2)?,
            blake3_hash: row.get(3)?,
        })
    })?.collect::<Result<Vec<_>>>()?;
    Ok(files)
}

pub fn update_enrichment(
    conn: &Connection, 
    id: i64, 
    category: Option<&str>, 
    confidence: Option<f32>, 
    date_source: Option<&str>, 
    date_value: Option<&str>, 
    artist: Option<&str>, 
    album: Option<&str>,
    dest_path: &str
) -> Result<()> {
    conn.execute(
        "UPDATE files SET 
            category = ?1, 
            confidence = ?2, 
            date_source = ?3, 
            date_value = ?4, 
            artist = ?5, 
            album = ?6,
            dest_path = ?7
         WHERE id = ?8",
        params![category, confidence, date_source, date_value, artist, album, dest_path, id],
    )?;
    Ok(())
}

pub struct StagingFile {
    pub id: i64,
    pub source_path: String,
    pub dest_path: Option<String>,
}

pub fn get_staging_files(conn: &Connection) -> Result<Vec<StagingFile>> {
    let mut stmt = conn.prepare("SELECT id, source_path, dest_path FROM files WHERE status = 'PRIMARIO'")?;
    let files = stmt.query_map([], |row| {
        Ok(StagingFile {
            id: row.get(0)?,
            source_path: row.get(1)?,
            dest_path: row.get(2)?,
        })
    })?.collect::<Result<Vec<_>>>()?;
    Ok(files)
}

pub fn update_staging_status(conn: &Connection, id: i64, status: &str, error_msg: Option<&str>) -> Result<()> {
    conn.execute(
        "UPDATE files SET status = ?1, error_msg = ?2 WHERE id = ?3",
        params![status, error_msg, id],
    )?;
    Ok(())
}

