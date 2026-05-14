#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilePathClassification {
    Source,
    Infrastructure,
    Backup,
    Project,
}

impl FilePathClassification {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Source => "source",
            Self::Infrastructure => "infrastructure",
            Self::Backup => "backup",
            Self::Project => "project",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilePathClassificationFilter {
    All,
    Source,
    Infrastructure,
    Backup,
    Project,
}

impl FilePathClassificationFilter {
    pub fn parse(value: Option<&str>) -> Result<Self, String> {
        match value.unwrap_or("all").trim().to_ascii_lowercase().as_str() {
            "all" => Ok(Self::All),
            "source" => Ok(Self::Source),
            "infrastructure" => Ok(Self::Infrastructure),
            "backup" => Ok(Self::Backup),
            "project" => Ok(Self::Project),
            other => Err(format!(
                "path_classification must be one of: all, source, infrastructure, backup, project; got '{}'",
                other
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Source => "source",
            Self::Infrastructure => "infrastructure",
            Self::Backup => "backup",
            Self::Project => "project",
        }
    }

    pub fn matches(self, classification: FilePathClassification) -> bool {
        match self {
            Self::All => true,
            Self::Source => classification == FilePathClassification::Source,
            Self::Infrastructure => classification == FilePathClassification::Infrastructure,
            Self::Backup => classification == FilePathClassification::Backup,
            Self::Project => classification == FilePathClassification::Project,
        }
    }
}

const INFRASTRUCTURE_DIRS: &[&str] = &[".claude", ".amazonq", ".cursor", ".agents", ".zread"];
const SOURCE_EXTENSIONS: &[&str] = &[
    "rs", "py", "vue", "js", "jsx", "ts", "tsx", "java", "kt", "go", "c", "cc", "cpp", "h", "hpp",
    "cs", "rb", "php", "swift", "scala", "sh", "bash", "zsh", "fish", "sql",
];
const BACKUP_SUFFIXES: &[&str] = &["~", ".bak", ".backup", ".orig", ".rej", ".swp", ".tmp"];

pub fn classify_file_path(rel_path: &str) -> FilePathClassification {
    let normalized = rel_path.replace('\\', "/");
    let lower = normalized.to_ascii_lowercase();

    if normalized
        .split('/')
        .any(|segment| INFRASTRUCTURE_DIRS.contains(&segment))
    {
        return FilePathClassification::Infrastructure;
    }

    if BACKUP_SUFFIXES.iter().any(|suffix| lower.ends_with(suffix)) {
        return FilePathClassification::Backup;
    }

    match lower.rsplit_once('.') {
        Some((_, ext)) if SOURCE_EXTENSIONS.contains(&ext) => FilePathClassification::Source,
        _ => FilePathClassification::Project,
    }
}

#[cfg(test)]
mod tests {
    use super::{classify_file_path, FilePathClassification, FilePathClassificationFilter};

    #[test]
    fn classifies_ai_tool_infrastructure_paths() {
        for path in [
            ".claude/settings.local.json",
            ".amazonq/rules.md",
            ".cursor/rules/project.mdc",
            ".agents/prompts/review.md",
            ".zread/wiki/current/index.md",
        ] {
            assert_eq!(
                classify_file_path(path),
                FilePathClassification::Infrastructure,
                "{path}"
            );
        }
    }

    #[test]
    fn classifies_backup_patterns_without_hiding_source_files() {
        for path in ["src/main.py.bak", "notes/tmp.txt~", "lib/module.rs.orig"] {
            assert_eq!(
                classify_file_path(path),
                FilePathClassification::Backup,
                "{path}"
            );
        }

        assert_eq!(
            classify_file_path("src/main.py"),
            FilePathClassification::Source
        );
        assert_eq!(
            classify_file_path("web/frontend/src/App.vue"),
            FilePathClassification::Source
        );
    }

    #[test]
    fn parses_user_facing_classification_filter_values() {
        assert_eq!(
            FilePathClassificationFilter::parse(None).unwrap(),
            FilePathClassificationFilter::All
        );
        assert_eq!(
            FilePathClassificationFilter::parse(Some("SOURCE")).unwrap(),
            FilePathClassificationFilter::Source
        );
        assert_eq!(
            FilePathClassificationFilter::parse(Some(" infrastructure ")).unwrap(),
            FilePathClassificationFilter::Infrastructure
        );
        assert_eq!(
            FilePathClassificationFilter::parse(Some("backup")).unwrap(),
            FilePathClassificationFilter::Backup
        );
        assert_eq!(
            FilePathClassificationFilter::parse(Some("project")).unwrap(),
            FilePathClassificationFilter::Project
        );

        let error = FilePathClassificationFilter::parse(Some("docs")).unwrap_err();
        assert!(error.contains("path_classification must be one of"));
    }

    #[test]
    fn classification_filter_matches_expected_path_classes() {
        assert!(FilePathClassificationFilter::All.matches(FilePathClassification::Source));
        assert!(FilePathClassificationFilter::Source.matches(FilePathClassification::Source));
        assert!(!FilePathClassificationFilter::Source.matches(FilePathClassification::Project));
    }
}
