use crate::analysis::AnalysisSnapshot;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ClipboardEntry {
    pub id: String,
    pub content_hash: String,
    pub content_type: String,
    pub content_data: Option<String>,
    pub source_app: Option<String>,
    pub created_at: i64,
    pub copy_count: i32,
    pub file_path: Option<String>,
    pub is_favorite: bool,
    pub content_subtype: Option<String>,
    pub metadata: Option<String>,
    pub app_bundle_id: Option<String>,
    #[sqlx(skip)]
    pub analysis: Option<AnalysisSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentType {
    Text,
    Image,
    Unknown,
}

impl ContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::Text => "text",
            ContentType::Image => "image",
            ContentType::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statistics {
    pub total_entries: i64,
    pub total_copies: i64,
    pub most_copied: Vec<ClipboardEntry>,
    pub recent_apps: Vec<AppUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUsage {
    pub app_name: String,
    pub count: i64,
}

impl ClipboardEntry {
    pub fn new(
        content_type: ContentType,
        content_data: Option<String>,
        content_hash: String,
        source_app: Option<String>,
        file_path: Option<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            content_hash,
            content_type: content_type.as_str().to_string(),
            content_data,
            source_app,
            created_at: Utc::now().timestamp_millis(),
            copy_count: 1,
            file_path,
            is_favorite: false,
            content_subtype: None,
            metadata: None,
            app_bundle_id: None,
            analysis: None,
        }
    }

    pub fn attach_analysis(&mut self, analysis: AnalysisSnapshot) {
        self.content_subtype = Some(analysis.subtype.as_str().to_string());
        self.metadata = analysis.metadata.to_legacy_metadata_json();
        self.analysis = Some(analysis);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_clipboard_entry_new_text() {
        let content_data = Some("Hello, world!".to_string());
        let content_hash = "test_hash".to_string();
        let source_app = Some("TestApp".to_string());

        let entry = ClipboardEntry::new(
            ContentType::Text,
            content_data.clone(),
            content_hash.clone(),
            source_app.clone(),
            None,
        );

        assert!(!entry.id.is_empty());
        assert_eq!(entry.content_type, "text");
        assert_eq!(entry.content_data, content_data);
        assert_eq!(entry.content_hash, content_hash);
        assert_eq!(entry.source_app, source_app);
        assert!(entry.created_at > 0);
        assert_eq!(entry.copy_count, 1);
        assert_eq!(entry.file_path, None);
        assert!(!entry.is_favorite);
        assert_eq!(entry.content_subtype, None);
        assert_eq!(entry.metadata, None);
        assert_eq!(entry.app_bundle_id, None);
        assert_eq!(entry.analysis, None);

        // Test UUID format
        assert!(uuid::Uuid::parse_str(&entry.id).is_ok());
    }

    #[test]
    fn test_clipboard_entry_new_image() {
        let content_hash = "image_hash_123".to_string();
        let file_path = Some("imgs/image.png".to_string());

        let entry = ClipboardEntry::new(
            ContentType::Image,
            None,
            content_hash.clone(),
            Some("ImageApp".to_string()),
            file_path.clone(),
        );

        assert_eq!(entry.content_type, "image");
        assert_eq!(entry.content_data, None);
        assert_eq!(entry.content_hash, content_hash);
        assert_eq!(entry.file_path, file_path);
        assert_eq!(entry.source_app, Some("ImageApp".to_string()));
    }

    #[test]
    fn test_clipboard_entry_with_metadata() {
        let mut entry = ClipboardEntry::new(
            ContentType::Text,
            Some("https://example.com".to_string()),
            "url_hash".to_string(),
            Some("Browser".to_string()),
            None,
        );

        // Set content subtype
        entry.content_subtype = Some("url".to_string());

        // Set metadata as JSON string
        let metadata = json!({
            "url_parts": {
                "protocol": "https",
                "host": "example.com",
                "path": "/",
                "query_params": []
            }
        });
        entry.metadata = Some(metadata.to_string());

        // Set app bundle ID
        entry.app_bundle_id = Some("com.company.browser".to_string());

        assert_eq!(entry.content_subtype, Some("url".to_string()));
        assert!(entry.metadata.is_some());
        assert_eq!(entry.app_bundle_id, Some("com.company.browser".to_string()));

        // Test metadata parsing
        let parsed_metadata: serde_json::Value =
            serde_json::from_str(&entry.metadata.unwrap()).unwrap();
        assert_eq!(parsed_metadata["url_parts"]["protocol"], "https");
        assert_eq!(parsed_metadata["url_parts"]["host"], "example.com");
    }

    #[test]
    fn test_clipboard_entry_analysis_fields_default_to_none() {
        let entry = ClipboardEntry::new(
            ContentType::Text,
            Some("analysis test".to_string()),
            "analysis_hash".to_string(),
            Some("AnalysisApp".to_string()),
            None,
        );

        assert!(entry.analysis.is_none());
        assert!(entry.content_subtype.is_none());
        assert!(entry.metadata.is_none());
    }

    #[test]
    fn test_clipboard_entry_empty_content() {
        let entry = ClipboardEntry::new(
            ContentType::Text,
            Some("".to_string()),
            "empty_hash".to_string(),
            None,
            None,
        );

        assert_eq!(entry.content_data, Some("".to_string()));
        assert_eq!(entry.source_app, None);
        assert!(!entry.id.is_empty());
    }

    #[test]
    fn test_clipboard_entry_none_content() {
        let entry = ClipboardEntry::new(
            ContentType::Unknown,
            None,
            "unknown_hash".to_string(),
            Some("UnknownApp".to_string()),
            None,
        );

        assert_eq!(entry.content_type, "unknown");
        assert_eq!(entry.content_data, None);
        assert_eq!(entry.source_app, Some("UnknownApp".to_string()));
    }

    #[test]
    fn test_clipboard_entry_unicode_content() {
        let unicode_content = "Hello 世界! 🌍 Привет мир! مرحبا بالعالم";
        let entry = ClipboardEntry::new(
            ContentType::Text,
            Some(unicode_content.to_string()),
            "unicode_hash".to_string(),
            Some("UnicodeApp".to_string()),
            None,
        );

        assert_eq!(entry.content_data, Some(unicode_content.to_string()));
        assert_eq!(entry.content_type, "text");
    }

    #[test]
    fn test_clipboard_entry_large_content() {
        let large_content = "A".repeat(100000); // 100KB of text
        let entry = ClipboardEntry::new(
            ContentType::Text,
            Some(large_content.clone()),
            "large_hash".to_string(),
            Some("LargeContentApp".to_string()),
            None,
        );

        assert_eq!(entry.content_data, Some(large_content));
        assert!(entry.content_data.as_ref().unwrap().len() == 100000);
    }

    #[test]
    fn test_clipboard_entry_special_characters() {
        let special_content = "!@#$%^&*()_+-=[]{}|;':\",./<>?`~\n\t\r";
        let entry = ClipboardEntry::new(
            ContentType::Text,
            Some(special_content.to_string()),
            "special_hash".to_string(),
            Some("SpecialApp".to_string()),
            None,
        );

        assert_eq!(entry.content_data, Some(special_content.to_string()));
    }

    #[test]
    fn test_clipboard_entry_json_metadata_serialization() {
        let mut entry = ClipboardEntry::new(
            ContentType::Text,
            Some(r#"{"key": "value"}"#.to_string()),
            "json_hash".to_string(),
            Some("JSONApp".to_string()),
            None,
        );

        // Test various metadata structures
        let complex_metadata = json!({
            "detected_language": "javascript",
            "content_subtype": "code",
            "code_analysis": {
                "functions": ["main", "helper"],
                "variables": ["count", "data"],
                "complexity": 3
            },
            "statistics": {
                "lines": 45,
                "characters": 1200,
                "words": 180
            }
        });

        entry.metadata = Some(complex_metadata.to_string());
        entry.content_subtype = Some("code".to_string());

        // Test serialization/deserialization
        let serialized = serde_json::to_string(&entry).unwrap();
        let deserialized: ClipboardEntry = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.content_subtype, Some("code".to_string()));
        assert!(deserialized.metadata.is_some());

        let parsed_metadata: serde_json::Value =
            serde_json::from_str(&deserialized.metadata.unwrap()).unwrap();
        assert_eq!(parsed_metadata["detected_language"], "javascript");
        assert_eq!(parsed_metadata["code_analysis"]["complexity"], 3);
    }

    #[test]
    fn test_clipboard_entry_timestamp_validation() {
        let before = Utc::now().timestamp_millis();
        let entry = ClipboardEntry::new(
            ContentType::Text,
            Some("test".to_string()),
            "test_hash".to_string(),
            None,
            None,
        );
        let after = Utc::now().timestamp_millis();

        assert!(entry.created_at >= before);
        assert!(entry.created_at <= after);
    }

    #[test]
    fn test_clipboard_entry_unique_ids() {
        let entries: Vec<ClipboardEntry> = (0..100)
            .map(|i| {
                ClipboardEntry::new(
                    ContentType::Text,
                    Some(format!("content_{}", i)),
                    format!("hash_{}", i),
                    Some(format!("app_{}", i)),
                    None,
                )
            })
            .collect();

        // Check all IDs are unique
        let mut ids = std::collections::HashSet::new();
        for entry in &entries {
            assert!(
                ids.insert(entry.id.clone()),
                "Duplicate ID found: {}",
                entry.id
            );
        }
        assert_eq!(ids.len(), 100);
    }

    #[test]
    fn test_content_type_as_str() {
        assert_eq!(ContentType::Text.as_str(), "text");
        assert_eq!(ContentType::Image.as_str(), "image");
        assert_eq!(ContentType::Unknown.as_str(), "unknown");
    }

    #[test]
    fn test_clipboard_entry_edge_cases() {
        // Test with maximum length strings
        let _max_id = "a".repeat(255);
        let max_hash = "b".repeat(255);
        let max_app_name = "c".repeat(255);

        let mut entry = ClipboardEntry::new(
            ContentType::Text,
            Some("test".to_string()),
            max_hash.clone(),
            Some(max_app_name.clone()),
            None,
        );

        // Manually set long values for testing
        entry.app_bundle_id = Some("com.very.long.bundle.identifier.for.testing".to_string());

        assert_eq!(entry.content_hash, max_hash);
        assert_eq!(entry.source_app, Some(max_app_name));
        assert!(entry.app_bundle_id.as_ref().unwrap().len() > 40);
    }

    #[test]
    fn test_clipboard_entry_file_path_variations() {
        let file_paths = vec![
            Some("imgs/image.png".to_string()),
            Some("/absolute/path/to/file.jpg".to_string()),
            Some("relative/path/document.pdf".to_string()),
            Some("file.txt".to_string()),
            None,
        ];

        for file_path in file_paths {
            let entry = ClipboardEntry::new(
                ContentType::Image,
                None,
                "test_hash".to_string(),
                Some("TestApp".to_string()),
                file_path.clone(),
            );

            assert_eq!(entry.file_path, file_path);
        }
    }

    #[test]
    fn test_clipboard_entry_all_content_subtypes() {
        let subtypes = vec![
            "plain_text",
            "url",
            "ip_address",
            "email",
            "color",
            "code",
            "command",
            "timestamp",
            "json",
            "markdown",
            "base64",
        ];

        for subtype in subtypes {
            let mut entry = ClipboardEntry::new(
                ContentType::Text,
                Some("test content".to_string()),
                "test_hash".to_string(),
                Some("TestApp".to_string()),
                None,
            );

            entry.content_subtype = Some(subtype.to_string());
            assert_eq!(entry.content_subtype, Some(subtype.to_string()));
        }
    }

    #[test]
    fn test_clipboard_entry_copy_count_operations() {
        let mut entry = ClipboardEntry::new(
            ContentType::Text,
            Some("popular content".to_string()),
            "popular_hash".to_string(),
            Some("PopularApp".to_string()),
            None,
        );

        // Test initial copy count
        assert_eq!(entry.copy_count, 1);

        // Simulate multiple copies
        entry.copy_count += 1;
        assert_eq!(entry.copy_count, 2);

        entry.copy_count += 10;
        assert_eq!(entry.copy_count, 12);

        // Test boundary values
        entry.copy_count = i32::MAX;
        assert_eq!(entry.copy_count, i32::MAX);

        entry.copy_count = 0;
        assert_eq!(entry.copy_count, 0);
    }

    #[test]
    fn test_clipboard_entry_favorite_operations() {
        let mut entry = ClipboardEntry::new(
            ContentType::Text,
            Some("favorite content".to_string()),
            "fav_hash".to_string(),
            Some("FavApp".to_string()),
            None,
        );

        // Test initial favorite state
        assert!(!entry.is_favorite);

        // Toggle favorite
        entry.is_favorite = true;
        assert!(entry.is_favorite);

        entry.is_favorite = false;
        assert!(!entry.is_favorite);
    }
}
