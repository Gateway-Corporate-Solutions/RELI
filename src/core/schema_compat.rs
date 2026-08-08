use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::core::validation::ValidationIssue;

fn read_json(path: &Path) -> Result<Value, ValidationIssue> {
    let content = fs::read_to_string(path).map_err(|err| {
        ValidationIssue {
            target: path.display().to_string(),
            details: format!("read failed: {err}"),
        }
    })?;

    serde_json::from_str::<Value>(&content).map_err(|err| ValidationIssue {
        target: path.display().to_string(),
        details: format!("invalid json: {err}"),
    })
}

fn required_set(schema: &Value) -> Result<BTreeSet<String>, ValidationIssue> {
    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .ok_or_else(|| ValidationIssue {
            target: "schema.required".to_string(),
            details: "missing or invalid required array".to_string(),
        })?;

    let mut out = BTreeSet::new();
    for item in required {
        let name = item.as_str().ok_or_else(|| ValidationIssue {
            target: "schema.required".to_string(),
            details: "required entry must be string".to_string(),
        })?;
        out.insert(name.to_string());
    }

    Ok(out)
}

fn properties_map(schema: &Value) -> Result<BTreeMap<String, Value>, ValidationIssue> {
    let props = schema
        .get("properties")
        .and_then(Value::as_object)
        .ok_or_else(|| ValidationIssue {
            target: "schema.properties".to_string(),
            details: "missing or invalid properties object".to_string(),
        })?;

    let mut out = BTreeMap::new();
    for (k, v) in props {
        out.insert(k.clone(), v.clone());
    }

    Ok(out)
}

fn maybe_type(schema: &Value) -> Option<&str> {
    schema.get("type").and_then(Value::as_str)
}

fn enum_values(schema: &Value) -> Option<BTreeSet<String>> {
    let values = schema.get("enum")?.as_array()?;
    let mut out = BTreeSet::new();

    for value in values {
        let as_str = value.as_str()?;
        out.insert(as_str.to_string());
    }

    Some(out)
}

fn additional_properties(schema: &Value) -> Option<bool> {
    schema.get("additionalProperties").and_then(Value::as_bool)
}

fn assert_backward_compatible_schema_values(
    baseline: &Value,
    candidate: &Value,
) -> Result<(), ValidationIssue> {
    if maybe_type(baseline) != Some("object") || maybe_type(candidate) != Some("object") {
        return Err(ValidationIssue {
            target: "schema.type".to_string(),
            details: "both schemas must be object schemas".to_string(),
        });
    }

    let baseline_required = required_set(baseline)?;
    let candidate_required = required_set(candidate)?;

    let newly_required: Vec<String> = candidate_required
        .difference(&baseline_required)
        .cloned()
        .collect();
    if !newly_required.is_empty() {
        return Err(ValidationIssue {
            target: "schema.required".to_string(),
            details: format!(
                "candidate adds new required fields and breaks backward compatibility: {}",
                newly_required.join(", ")
            ),
        });
    }

    let baseline_props = properties_map(baseline)?;
    let candidate_props = properties_map(candidate)?;

    for (name, baseline_prop) in &baseline_props {
        let candidate_prop = candidate_props.get(name).ok_or_else(|| ValidationIssue {
            target: format!("schema.properties.{name}"),
            details: "candidate removed existing property".to_string(),
        })?;

        if maybe_type(candidate_prop) != maybe_type(baseline_prop) {
            return Err(ValidationIssue {
                target: format!("schema.properties.{name}.type"),
                details: "candidate changed property type".to_string(),
            });
        }

        if let (Some(base_enum), Some(candidate_enum)) =
            (enum_values(baseline_prop), enum_values(candidate_prop))
        {
            if !base_enum.is_subset(&candidate_enum) {
                return Err(ValidationIssue {
                    target: format!("schema.properties.{name}.enum"),
                    details: "candidate removed existing enum values".to_string(),
                });
            }
        }
    }

    if additional_properties(baseline) == Some(true) && additional_properties(candidate) == Some(false)
    {
        return Err(ValidationIssue {
            target: "schema.additionalProperties".to_string(),
            details: "candidate tightens additionalProperties from true to false".to_string(),
        });
    }

    Ok(())
}

pub fn assert_backward_compatible_schema(
    baseline_path: &Path,
    candidate_path: &Path,
) -> Result<(), ValidationIssue> {
    let baseline = read_json(baseline_path)?;
    let candidate = read_json(candidate_path)?;
    assert_backward_compatible_schema_values(&baseline, &candidate)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::assert_backward_compatible_schema;

    #[test]
    fn job_spec_compatible_candidate_passes() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let baseline = root.join("docs/schemas/v1/job_spec.schema.json");
        let candidate = root.join("docs/schemas/compat/job_spec.v1_compatible.schema.json");

        assert!(assert_backward_compatible_schema(&baseline, &candidate).is_ok());
    }

    #[test]
    fn job_spec_breaking_candidate_fails() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let baseline = root.join("docs/schemas/v1/job_spec.schema.json");
        let candidate = root.join("docs/schemas/compat/job_spec.v1_breaking.schema.json");

        let err = assert_backward_compatible_schema(&baseline, &candidate)
            .expect_err("breaking candidate should fail compatibility");
        assert!(err.details.contains("adds new required fields"));
    }
}