use std::path::Path;
use anyhow::{Context, Result};
use super::pose::CameraPose;

pub fn load_from_json(path: impl AsRef<Path>) -> Result<Vec<CameraPose>> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let poses: Vec<CameraPose> = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON from {}", path.display()))?;
    Ok(poses)
}

pub fn load_from_csv(path: impl AsRef<Path>) -> Result<Vec<CameraPose>> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let mut poses = Vec::new();
    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("timestamp") {
            continue;
        }
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 8 {
            anyhow::bail!("Line {}: expected 8 fields, got {}", line_num + 1, parts.len());
        }
        let parse = |s: &str, field: &str| -> Result<f64> {
            s.trim().parse::<f64>().with_context(|| format!("Line {}: bad {}", line_num + 1, field))
        };
        poses.push(CameraPose {
            timestamp: parse(parts[0], "timestamp")?,
            position: [
                parse(parts[1], "px")? as f32,
                parse(parts[2], "py")? as f32,
                parse(parts[3], "pz")? as f32,
            ],
            orientation: [
                parse(parts[4], "qx")? as f32,
                parse(parts[5], "qy")? as f32,
                parse(parts[6], "qz")? as f32,
                parse(parts[7], "qw")? as f32,
            ],
        });
    }
    Ok(poses)
}

pub fn load_poses(path: impl AsRef<Path>) -> Result<Vec<CameraPose>> {
    let path = path.as_ref();
    match path.extension().and_then(|e| e.to_str()) {
        Some("json") => load_from_json(path),
        Some("csv") => load_from_csv(path),
        _ => {
            // Try JSON first, then CSV
            load_from_json(path).or_else(|_| load_from_csv(path))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_parse() {
        let json = r#"[
            {"timestamp": 0.0, "position": [0.0, 0.0, 0.0], "orientation": [0.0, 0.0, 0.0, 1.0]},
            {"timestamp": 0.033, "position": [0.05, 0.0, 0.1], "orientation": [0.0, 0.02, 0.0, 0.9998]}
        ]"#;
        let poses: Vec<CameraPose> = serde_json::from_str(json).unwrap();
        assert_eq!(poses.len(), 2);
        assert_eq!(poses[0].position, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_csv_parse() {
        let csv = "0.0,0.0,0.0,0.0,0.0,0.0,0.0,1.0\n0.033,0.05,0.0,0.1,0.0,0.02,0.0,0.9998\n";
        let tmp = std::env::temp_dir().join("test_poses.csv");
        std::fs::write(&tmp, csv).unwrap();
        let poses = load_from_csv(&tmp).unwrap();
        assert_eq!(poses.len(), 2);
        std::fs::remove_file(tmp).ok();
    }
}
