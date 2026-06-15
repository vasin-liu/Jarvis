use std::fs;
use std::path::Path;

use crate::types::Skill;

pub fn load_skills_from_dir(dir: &Path) -> Vec<Skill> {
    if !dir.is_dir() {
        return Vec::new();
    }
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        if let Ok(skill) = parse_skill_file(&path) {
            out.push(skill);
        }
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

fn parse_skill_file(path: &Path) -> std::io::Result<Skill> {
    let raw = fs::read_to_string(path)?;
    let id = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("skill")
        .to_string();

    let mut name = id.clone();
    let mut description = String::new();
    let mut content = raw.clone();

    if raw.starts_with("---") {
        if let Some(end) = raw[3..].find("\n---") {
            let front = &raw[3..3 + end];
            content = raw[3 + end + 4..].trim_start().to_string();
            for line in front.lines() {
                if let Some((k, v)) = line.split_once(':') {
                    match k.trim() {
                        "name" => name = v.trim().to_string(),
                        "description" => description = v.trim().to_string(),
                        _ => {}
                    }
                }
            }
        }
    } else if let Some(first) = content.lines().next() {
        if let Some(title) = first.strip_prefix("# ") {
            name = title.trim().to_string();
            content = content.lines().skip(1).collect::<Vec<_>>().join("\n");
        }
    }

    if description.is_empty() {
        description = content.lines().next().unwrap_or("").chars().take(80).collect();
    }

    Ok(Skill {
        id,
        name,
        description,
        content: content.trim().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_frontmatter_skill() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("helper.md");
        fs::write(
            &path,
            "---\nname: Helper\ndescription: Helps search\n---\nDo better search.",
        )
        .unwrap();
        let skills = load_skills_from_dir(dir.path());
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "Helper");
    }
}
