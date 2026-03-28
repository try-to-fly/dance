#[cfg(test)]
mod analysis_contract_tests {
    use crate::analysis::contract::{
        AnalysisDiagnostic, AnalysisDiagnosticCode, AnalysisDiagnosticSeverity, AnalysisStatus,
        AnalysisSubtype, ANALYSIS_CONTRACT_VERSION, TEXT_ANALYSIS_VERSION,
    };
    use crate::analysis::TextAnalysisService;
    use serde_json::Value;

    #[test]
    fn test_text_analysis_contract_preserves_supported_subtypes() {
        let service = TextAnalysisService::new();

        let cases = vec![
            ("https://example.com/path?foo=bar", AnalysisSubtype::Url),
            (r#"{"hello":"world","count":2}"#, AnalysisSubtype::Json),
            (
                "function greet(name) {\n  return name;\n}",
                AnalysisSubtype::Code,
            ),
            ("git status --short", AnalysisSubtype::Command),
            ("#ff0000", AnalysisSubtype::Color),
            ("# Title\n\n- item", AnalysisSubtype::Markdown),
            ("user@example.com", AnalysisSubtype::Email),
            ("192.168.1.1", AnalysisSubtype::IpAddress),
            ("1640995200000", AnalysisSubtype::Timestamp),
            ("aGVsbG8gd29ybGQ=", AnalysisSubtype::Base64),
            ("ordinary clipboard note", AnalysisSubtype::PlainText),
        ];

        for (input, expected_subtype) in cases {
            let snapshot = service.analyze(input);

            assert_eq!(
                snapshot.subtype, expected_subtype,
                "expected {:?} for input {}",
                expected_subtype, input
            );
            assert_eq!(snapshot.status, AnalysisStatus::Matched);
            assert_eq!(snapshot.contract_version, ANALYSIS_CONTRACT_VERSION);
            assert_eq!(snapshot.analysis_version, TEXT_ANALYSIS_VERSION);
            assert!(
                snapshot.diagnostics.is_empty(),
                "matched snapshot should not carry diagnostics"
            );
        }
    }

    #[test]
    fn test_text_analysis_contract_preserves_precedence_for_ambiguous_inputs() {
        let service = TextAnalysisService::new();

        let json_with_url = service.analyze(r#"{"url":"https://example.com/api"}"#);
        assert_eq!(json_with_url.subtype, AnalysisSubtype::Json);

        let markdown_with_url = service.analyze("# Docs\n\n[site](https://example.com)");
        assert_eq!(markdown_with_url.subtype, AnalysisSubtype::Markdown);

        let shell_command = service.analyze("npm run dev");
        assert_eq!(shell_command.subtype, AnalysisSubtype::Command);
    }

    #[test]
    fn test_text_analysis_contract_failure_fallback_serializes_diagnostics() {
        let service = TextAnalysisService::new();
        let snapshot = service.fallback_plain_text(
            "{broken-json",
            vec![
                AnalysisDiagnostic::new(
                    AnalysisDiagnosticCode::JsonMalformed,
                    AnalysisDiagnosticSeverity::Error,
                    "json parse failed",
                ),
                AnalysisDiagnostic::new(
                    AnalysisDiagnosticCode::HeuristicFallback,
                    AnalysisDiagnosticSeverity::Warning,
                    "degraded to plain text",
                ),
            ],
        );

        assert_eq!(snapshot.status, AnalysisStatus::Fallback);
        assert_eq!(snapshot.subtype, AnalysisSubtype::PlainText);
        assert_eq!(snapshot.contract_version, ANALYSIS_CONTRACT_VERSION);
        assert_eq!(snapshot.analysis_version, TEXT_ANALYSIS_VERSION);
        assert_eq!(snapshot.diagnostics.len(), 2);

        let serialized = serde_json::to_value(&snapshot).expect("snapshot should serialize");
        assert_eq!(serialized["status"], Value::String("fallback".to_string()));
        assert_eq!(
            serialized["subtype"],
            Value::String("plain_text".to_string())
        );
        assert_eq!(
            serialized["contract_version"],
            Value::Number(ANALYSIS_CONTRACT_VERSION.into())
        );
        assert_eq!(
            serialized["analysis_version"],
            Value::Number(TEXT_ANALYSIS_VERSION.into())
        );
        assert_eq!(
            serialized["diagnostics"][0]["code"],
            Value::String("json_malformed".to_string())
        );
        assert_eq!(
            serialized["diagnostics"][0]["severity"],
            Value::String("error".to_string())
        );
        assert_eq!(
            serialized["diagnostics"][1]["code"],
            Value::String("heuristic_fallback".to_string())
        );
    }
}
