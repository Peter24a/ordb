//! EN:
//! This module is part of the `ordb-cli` application.
//! It provides the `project_dest_path` function to determine the final destination path
//! of a file based on its metadata, including MIME type, dates, and category.
//! It handles name collisions by injecting a hash prefix into the file name.
//!
//! ES:
//! Este módulo forma parte de la aplicación `ordb-cli`.
//! Proporciona la función `project_dest_path` para determinar la ruta de destino final
//! de un archivo en función de sus metadatos, incluyendo tipo MIME, fechas y categoría.
//! Maneja las colisiones de nombres inyectando un prefijo del hash en el nombre del archivo.

use std::path::Path;
use chrono::{DateTime, Datelike};
use std::collections::HashMap;

pub fn project_dest_path(
    base_dest: &Path,
    source_path_str: &str,
    mime: Option<&str>,
    category: &str,
    date_val: Option<&str>,
    date_src: Option<&str>,
    artist: Option<&str>,
    album: Option<&str>,
    hash: &str,
    used_paths: &mut HashMap<String, String>,
) -> String {
    let source_path = Path::new(source_path_str);
    let original_name = source_path.file_name().unwrap_or_default().to_string_lossy();
    let mut relative_parts = Vec::new();

    let is_image = mime.unwrap_or("").starts_with("image/");
    let is_video = mime.unwrap_or("").starts_with("video/");
    let is_audio = mime.unwrap_or("").starts_with("audio/");

    if is_image || is_video {
        relative_parts.push("Imagenes".to_string());
        
        if let Some(src) = date_src {
            if src == "SOSPECHOSA" {
                relative_parts.push("Fecha_Sospechosa".to_string());
            } else if let Some(val) = date_val {
                if let Ok(dt) = DateTime::parse_from_rfc3339(val) {
                    relative_parts.push(format!("{:04}", dt.year()));
                    let month_name = match dt.month() {
                        1 => "Enero", 2 => "Febrero", 3 => "Marzo", 4 => "Abril",
                        5 => "Mayo", 6 => "Junio", 7 => "Julio", 8 => "Agosto",
                        9 => "Septiembre", 10 => "Octubre", 11 => "Noviembre", 12 => "Diciembre",
                        _ => "Desconocido",
                    };
                    relative_parts.push(format!("{:02}_{}", dt.month(), month_name));
                } else {
                    relative_parts.push("Sin_Fecha".to_string());
                }
            } else {
                relative_parts.push("Sin_Fecha".to_string());
            }
        } else {
            relative_parts.push("Sin_Fecha".to_string());
        }
        
        relative_parts.push(category.to_string());
    } else if is_audio {
        relative_parts.push("Musica".to_string());
        if let Some(art) = artist {
            relative_parts.push(art.to_string());
            if let Some(alb) = album {
                relative_parts.push(alb.to_string());
            }
        } else {
            relative_parts.push("Desconocido".to_string());
        }
    } else {
        relative_parts.push("Otros".to_string());
        let ext = source_path.extension().unwrap_or_default().to_string_lossy();
        if !ext.is_empty() {
            relative_parts.push(ext.to_string());
        } else {
            relative_parts.push("Sin_Extension".to_string());
        }
    }

    let mut projected = base_dest.to_path_buf();
    for part in relative_parts {
        projected.push(part);
    }
    
    let mut projected_file = projected.clone();
    projected_file.push(original_name.as_ref());
    
    let mut final_path_str = projected_file.to_string_lossy().to_string();
    
    if let Some(existing_hash) = used_paths.get(&final_path_str) {
        if existing_hash != hash {
            let file_stem = source_path.file_stem().unwrap_or_default().to_string_lossy();
            let ext = source_path.extension().unwrap_or_default().to_string_lossy();
            let hash_prefix = &hash[0..8];
            
            let new_name = if ext.is_empty() {
                format!("{}_{}", file_stem, hash_prefix)
            } else {
                format!("{}_{}.{}", file_stem, hash_prefix, ext)
            };
            
            let mut projected_with_hash = projected;
            projected_with_hash.push(new_name);
            final_path_str = projected_with_hash.to_string_lossy().to_string();
        }
    }
    
    used_paths.insert(final_path_str.clone(), hash.to_string());
    final_path_str
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_name_collision_hash_injection() {
        let base_dest = Path::new("/dest");
        let mut used_paths = HashMap::new();

        let path1 = project_dest_path(
            base_dest,
            "/source1/IMG_001.jpg",
            Some("image/jpeg"),
            "Persona_Sola",
            Some("2023-01-01T12:00:00Z"),
            Some("EXIF_ORIGINAL"),
            None,
            None,
            "1111111111111111111111111111111111111111111111111111111111111111",
            &mut used_paths,
        );

        let path2 = project_dest_path(
            base_dest,
            "/source2/IMG_001.jpg",
            Some("image/jpeg"),
            "Persona_Sola",
            Some("2023-01-01T15:00:00Z"),
            Some("EXIF_ORIGINAL"),
            None,
            None,
            "2222222222222222222222222222222222222222222222222222222222222222",
            &mut used_paths,
        );

        #[cfg(target_os = "windows")]
        assert!(path1.ends_with("IMG_001.jpg") || path1.ends_with("IMG_001.jpg"));
        
        assert!(path2.ends_with("IMG_001_22222222.jpg"));
    }
}
