use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[repr(transparent)]
pub struct InferenceOutput(pub Vec<InferenceOutputUnit>);

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct InferenceOutputUnit {
    pub start: u32,
    pub end: u32,
    pub suggestion: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_parse_inference_output() {
        let inference_output_str = r#"[{"start": 30, "end": 60, "suggestion": "sorrow"}, {"start": 120, "end": 150, "suggestion": "anger"}]"#;
        let inference_output: InferenceOutput = serde_json::from_str(inference_output_str).unwrap();

        assert_eq!(
            &inference_output.0[0],
            &InferenceOutputUnit {
                start: 30,
                end: 60,
                suggestion: String::from("sorrow"),
            }
        );
        assert_eq!(
            &inference_output.0[1],
            &InferenceOutputUnit {
                start: 120,
                end: 150,
                suggestion: String::from("anger"),
            }
        );
    }
}
