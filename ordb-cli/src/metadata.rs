use std::fs;
use chrono::{DateTime, Local};
use id3::TagLike;

pub struct DateInfo {
    pub source: String,
    pub value: Option<String>,
}

pub struct MusicInfo {
    pub artist: Option<String>,
    pub album: Option<String>,
}

pub fn extract_date(path: &str) -> DateInfo {
    if let Ok(file) = fs::File::open(path) {
        let mut reader = std::io::BufReader::new(&file);
        if let Ok(exif) = exif::Reader::new().read_from_container(&mut reader) {
            if let Some(field) = exif.get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY) {
                if let exif::Value::Ascii(ref vec) = field.value {
                    if let Some(date_str) = vec.first() {
                        let raw = std::str::from_utf8(date_str).unwrap_or("");
                        // EXIF format: "2023:06:15 14:30:00" → RFC3339: "2023-06-15T14:30:00+00:00"
                        if raw.len() >= 19 {
                            let rfc3339 = format!(
                                "{}-{}-{}T{}+00:00",
                                &raw[0..4], &raw[5..7], &raw[8..10], &raw[11..19]
                            );
                            // Check for suspicious year (< 1990)
                            let year: i32 = raw[0..4].parse().unwrap_or(0);
                            if year < 1990 {
                                return DateInfo { source: "SOSPECHOSA".to_string(), value: Some(rfc3339) };
                            }
                            return DateInfo { source: "EXIF_ORIGINAL".to_string(), value: Some(rfc3339) };
                        }
                    }
                }
            }
        }
    }
    
    if let Ok(metadata) = fs::metadata(path) {
        if let Ok(modified) = metadata.modified() {
            let datetime: DateTime<Local> = modified.into();
            let year = datetime.format("%Y").to_string().parse::<i32>().unwrap_or(0);
            if year < 1990 {
                return DateInfo { source: "SOSPECHOSA".to_string(), value: Some(datetime.to_rfc3339()) };
            }
            return DateInfo { source: "FILESYSTEM".to_string(), value: Some(datetime.to_rfc3339()) };
        }
    }
    DateInfo { source: "NINGUNA".to_string(), value: None }
}

pub fn extract_music_tags(path: &str) -> MusicInfo {
    if let Ok(tag) = id3::Tag::read_from_path(path) {
        return MusicInfo {
            artist: Some(tag.artist().unwrap_or("Desconocido").to_string()),
            album: Some(tag.album().unwrap_or("Desconocido").to_string()),
        };
    }
    MusicInfo {
        artist: Some("Desconocido".to_string()),
        album: Some("Desconocido".to_string()),
    }
}
