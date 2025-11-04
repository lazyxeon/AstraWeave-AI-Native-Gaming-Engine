/// Phase 7: Enhanced LLM Plan Parser with JSON Validation and Hallucination Detection
///
/// This module provides robust parsing of LLM responses into validated PlanIntent structures.
/// Key improvements over legacy parser:
/// - Tool hallucination detection (reject unknown tools)
/// - JSON schema validation against tool vocabulary
/// - Multi-stage extraction (fenced code, envelopes, objects)
/// - Detailed error reporting with recovery suggestions
/// - Metrics tracking for parse success rates
use anyhow::{anyhow, bail, Context, Result};
use astraweave_core::{ActionStep, PlanIntent, ToolRegistry};
use serde_json::Value;
use std::collections::HashSet;
use tracing::debug;

/// Parse result with metadata
#[derive(Debug, Clone)]
pub struct ParseResult {
    pub plan: PlanIntent,
    pub extraction_method: ExtractionMethod,
    pub validation_warnings: Vec<String>,
}

/// How the JSON was extracted from the response
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractionMethod {
    Direct,           // Direct JSON parse
    CodeFence,        // Extracted from ```json ... ```
    Envelope,         // Extracted from message.content or response field
    ObjectExtraction, // Regex-based object extraction
    Tolerant,         // Fallback with key normalization
}

impl ExtractionMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::CodeFence => "code_fence",
            Self::Envelope => "envelope",
            Self::ObjectExtraction => "object_extraction",
            Self::Tolerant => "tolerant",
        }
    }
}

/// Parse LLM response into validated PlanIntent
///
/// # Arguments
/// * `json_text` - Raw LLM response (may contain markdown, envelopes, etc.)
/// * `reg` - Tool registry for validation
///
/// # Returns
/// * `Ok(ParseResult)` - Successfully parsed and validated plan
/// * `Err(anyhow::Error)` - Parse failure with detailed error message
///
/// # Example
/// ```
/// use astraweave_llm::plan_parser::{parse_llm_response, ExtractionMethod};
/// use astraweave_core::default_tool_registry;
///
/// # fn example() -> anyhow::Result<()> {
/// let registry = default_tool_registry();
/// let llm_output = r#"{"plan_id": "test-123", "steps": [{"Wait": {"ticks": 5}}]}"#;
/// let result = parse_llm_response(llm_output, &registry)?;
///
/// assert_eq!(result.extraction_method, ExtractionMethod::Direct);
/// assert_eq!(result.plan.steps.len(), 1);
/// # Ok(())
/// # }
/// ```
pub fn parse_llm_response(json_text: &str, reg: &ToolRegistry) -> Result<ParseResult> {
    // ═══ PHASE 7 DEBUG LOGGING ═══
    eprintln!("\n╔═══════════════════════════════════════════════════════════════╗");
    eprintln!("║              PARSING LLM RESPONSE (5 Stages)                 ║");
    eprintln!("╠═══════════════════════════════════════════════════════════════╣");
    eprintln!("Input Length: {} chars", json_text.len());
    eprintln!("Input Preview: {}", truncate(json_text, 150));
    eprintln!("╚═══════════════════════════════════════════════════════════════╝\n");

    let mut warnings = Vec::new();

    // Stage 1: Try direct parse
    eprintln!("  [Stage 1/5] Attempting Direct JSON Parse...");
    if let Ok(plan) = try_direct_parse(json_text, reg, &mut warnings) {
        eprintln!(
            "  ✅ SUCCESS via Direct Parse! Plan has {} steps",
            plan.steps.len()
        );
        return Ok(ParseResult {
            plan,
            extraction_method: ExtractionMethod::Direct,
            validation_warnings: warnings,
        });
    }
    eprintln!("  ❌ Stage 1 failed");

    // Stage 2: Try code fence extraction
    eprintln!("  [Stage 2/5] Attempting Code Fence Extraction...");
    if let Ok(plan) = try_code_fence_parse(json_text, reg, &mut warnings) {
        eprintln!(
            "  ✅ SUCCESS via Code Fence! Plan has {} steps",
            plan.steps.len()
        );
        return Ok(ParseResult {
            plan,
            extraction_method: ExtractionMethod::CodeFence,
            validation_warnings: warnings,
        });
    }
    eprintln!("  ❌ Stage 2 failed");

    // Stage 3: Try envelope extraction (message.content, response fields)
    eprintln!("  [Stage 3/5] Attempting Envelope Extraction...");
    if let Ok(plan) = try_envelope_parse(json_text, reg, &mut warnings) {
        eprintln!(
            "  ✅ SUCCESS via Envelope! Plan has {} steps",
            plan.steps.len()
        );
        return Ok(ParseResult {
            plan,
            extraction_method: ExtractionMethod::Envelope,
            validation_warnings: warnings,
        });
    }
    eprintln!("  ❌ Stage 3 failed");

    // Stage 4: Try regex-based object extraction
    eprintln!("  [Stage 4/5] Attempting Object Extraction...");
    if let Ok(plan) = try_object_extraction(json_text, reg, &mut warnings) {
        eprintln!(
            "  ✅ SUCCESS via Object Extraction! Plan has {} steps",
            plan.steps.len()
        );
        return Ok(ParseResult {
            plan,
            extraction_method: ExtractionMethod::ObjectExtraction,
            validation_warnings: warnings,
        });
    }
    eprintln!("  ❌ Stage 4 failed");

    // Stage 5: Tolerant parsing with key normalization
    eprintln!("  [Stage 5/5] Attempting Tolerant Parse...");
    if let Ok(plan) = try_tolerant_parse(json_text, reg, &mut warnings) {
        eprintln!(
            "  ✅ SUCCESS via Tolerant Parse! Plan has {} steps",
            plan.steps.len()
        );
        return Ok(ParseResult {
            plan,
            extraction_method: ExtractionMethod::Tolerant,
            validation_warnings: warnings,
        });
    }
    eprintln!("  ❌ Stage 5 failed");

    // All stages failed
    eprintln!("\n╔═══════════════════════════════════════════════════════════════╗");
    eprintln!("║                 ❌ ALL 5 PARSING STAGES FAILED                ║");
    eprintln!("╠═══════════════════════════════════════════════════════════════╣");
    eprintln!("Response text (first 500 chars):");
    eprintln!("{}", truncate(json_text, 500));
    eprintln!("╚═══════════════════════════════════════════════════════════════╝\n");

    bail!(
        "Failed to parse LLM response after all extraction methods. Response preview: {}",
        truncate(json_text, 200)
    );
}

// ============================================================================
// Stage 1: Direct Parse
// ============================================================================

fn try_direct_parse(
    text: &str,
    reg: &ToolRegistry,
    warnings: &mut Vec<String>,
) -> Result<PlanIntent> {
    let plan: PlanIntent = serde_json::from_str(text.trim()).context("Direct JSON parse failed")?;

    validate_plan(&plan, reg, warnings)?;
    debug!("Successfully parsed plan via direct method");
    Ok(plan)
}

// ============================================================================
// Stage 2: Code Fence Extraction
// ============================================================================

fn try_code_fence_parse(
    text: &str,
    reg: &ToolRegistry,
    warnings: &mut Vec<String>,
) -> Result<PlanIntent> {
    // Match ```json ... ``` or ``` ... ```
    let fenced = extract_code_fence(text)?;
    let cleaned = clean_json(fenced); // Clean trailing commas
    let plan: PlanIntent =
        serde_json::from_str(cleaned.trim()).context("Code fence JSON parse failed")?;

    validate_plan(&plan, reg, warnings)?;
    debug!("Successfully parsed plan via code fence");
    Ok(plan)
}

fn extract_code_fence(text: &str) -> Result<&str> {
    // Look for ```json ... ``` or ``` ... ```
    if let Some(start) = text.find("```json") {
        let after_marker = &text[start + 7..]; // Skip ```json
        if let Some(end) = after_marker.find("```") {
            return Ok(after_marker[..end].trim());
        }
    }

    if let Some(start) = text.find("```") {
        let after_marker = &text[start + 3..];
        if let Some(end) = after_marker.find("```") {
            return Ok(after_marker[..end].trim());
        }
    }

    bail!("No code fence found")
}

/// Clean common JSON issues (trailing commas, extra fields)
fn clean_json(text: &str) -> String {
    // Remove trailing commas before closing brackets/braces
    let cleaned = text
        .replace(",\n  ]", "\n  ]")
        .replace(", ]", "]")
        .replace(",]", "]")
        .replace(",\n}", "\n}")
        .replace(", }", "}")
        .replace(",}", "}");

    cleaned
}

// ============================================================================
// Stage 3: Envelope Extraction
// ============================================================================

fn try_envelope_parse(
    text: &str,
    reg: &ToolRegistry,
    warnings: &mut Vec<String>,
) -> Result<PlanIntent> {
    let value: Value =
        serde_json::from_str(text).context("Cannot parse as JSON for envelope extraction")?;

    // Try message.content
    if let Some(content) = value.pointer("/message/content").and_then(|v| v.as_str()) {
        if let Ok(plan) = serde_json::from_str::<PlanIntent>(content.trim()) {
            validate_plan(&plan, reg, warnings)?;
            debug!("Successfully parsed plan from message.content envelope");
            return Ok(plan);
        }

        // Try extracting JSON from content string
        if let Ok(fenced) = extract_code_fence(content) {
            if let Ok(plan) = serde_json::from_str::<PlanIntent>(fenced) {
                validate_plan(&plan, reg, warnings)?;
                debug!("Successfully parsed plan from fenced JSON in message.content");
                return Ok(plan);
            }
        }
    }

    // Try response field
    if let Some(response) = value.get("response").and_then(|v| v.as_str()) {
        if let Ok(plan) = serde_json::from_str::<PlanIntent>(response.trim()) {
            validate_plan(&plan, reg, warnings)?;
            debug!("Successfully parsed plan from response field");
            return Ok(plan);
        }
    }

    bail!("No valid plan found in envelope fields")
}

// ============================================================================
// Stage 4: Object Extraction
// ============================================================================

fn try_object_extraction(
    text: &str,
    reg: &ToolRegistry,
    warnings: &mut Vec<String>,
) -> Result<PlanIntent> {
    // Find first {... } balanced object
    if let Some(obj_str) = extract_json_object(text) {
        eprintln!(
            "  📦 Extracted object ({} chars): {}",
            obj_str.len(),
            if obj_str.len() > 200 {
                &obj_str[..200]
            } else {
                obj_str
            }
        );

        let cleaned = clean_json(obj_str); // Clean trailing commas
        eprintln!(
            "  🧹 After cleaning ({} chars): {}",
            cleaned.len(),
            if cleaned.len() > 200 {
                &cleaned[..200]
            } else {
                &cleaned
            }
        );

        match serde_json::from_str::<PlanIntent>(cleaned.trim()) {
            Ok(plan) => {
                validate_plan(&plan, reg, warnings)?;
                debug!("Successfully parsed plan via object extraction");
                return Ok(plan);
            }
            Err(e) => {
                eprintln!("  ❌ Parse error: {}", e);
            }
        }
    }

    bail!("No valid JSON object found")
}

fn extract_json_object(text: &str) -> Option<&str> {
    let start = text.find('{')?;
    let mut depth = 0;
    let mut in_string = false;
    let mut escape_next = false;

    for (i, ch) in text[start..].char_indices() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match ch {
            '\\' if in_string => escape_next = true,
            '"' => in_string = !in_string,
            '{' if !in_string => depth += 1,
            '}' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    return Some(&text[start..=start + i]);
                }
            }
            _ => {}
        }
    }

    None
}

// ============================================================================
// Stage 5: Tolerant Parsing
// ============================================================================

fn try_tolerant_parse(
    text: &str,
    reg: &ToolRegistry,
    warnings: &mut Vec<String>,
) -> Result<PlanIntent> {
    // Try to extract JSON object first
    let obj_str = extract_json_object(text)
        .ok_or_else(|| anyhow!("No JSON object found for tolerant parsing"))?;

    let cleaned = clean_json(obj_str); // Clean trailing commas
    let value: Value =
        serde_json::from_str(&cleaned).context("Cannot parse extracted object as JSON")?;

    // Extract plan_id with normalization
    let plan_id = extract_plan_id(&value)?;

    // Extract steps array
    let steps_value = value
        .get("steps")
        .ok_or_else(|| anyhow!("Missing 'steps' field"))?;

    let steps: Vec<ActionStep> =
        serde_json::from_value(steps_value.clone()).context("Cannot parse steps array")?;

    let plan = PlanIntent { plan_id, steps };
    validate_plan(&plan, reg, warnings)?;

    warnings.push("Used tolerant parsing - plan_id key may have been normalized".to_string());
    debug!("Successfully parsed plan via tolerant method");
    Ok(plan)
}

fn extract_plan_id(value: &Value) -> Result<String> {
    // Try common variations
    let candidates = [
        "plan_id",
        "plan_eid",
        "id",
        "plan_no",
        "plan_num",
        "planNumber",
        "plan_n",
        "planId",
        "planID",
    ];

    for &key in &candidates {
        if let Some(id) = value.get(key).and_then(|v| v.as_str()) {
            return Ok(id.to_string());
        }
    }

    // Try normalized key matching
    if let Some(obj) = value.as_object() {
        for (k, v) in obj.iter() {
            let normalized: String = k
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
                .to_lowercase();

            if normalized.contains("plan") && normalized.contains("id") {
                if let Some(id) = v.as_str() {
                    return Ok(id.to_string());
                }
            }
        }
    }

    bail!(
        "Cannot find plan_id field (tried {} variations)",
        candidates.len()
    )
}

// ============================================================================
// Validation
// ============================================================================

/// Validate plan against tool registry and check for hallucinations
fn validate_plan(plan: &PlanIntent, reg: &ToolRegistry, warnings: &mut Vec<String>) -> Result<()> {
    // Check for empty plans
    if plan.steps.is_empty() {
        warnings.push("Plan has no steps".to_string());
        return Ok(()); // Not an error, just unusual
    }

    // Build set of allowed tool names from registry
    let allowed_tools: HashSet<String> = reg.tools.iter().map(|t| t.name.clone()).collect();

    // Check each step for hallucinated tools
    for (i, step) in plan.steps.iter().enumerate() {
        let tool_name = action_step_to_tool_name(step);

        if !allowed_tools.contains(tool_name) {
            bail!(
                "Hallucinated tool at step {}: '{}' is not in registry (allowed: {})",
                i + 1,
                tool_name,
                allowed_tools
                    .iter()
                    .take(5)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }

    debug!(
        "Plan validation passed: {} steps, all tools valid",
        plan.steps.len()
    );
    Ok(())
}

/// Map ActionStep to tool name for validation
/// MUST match ToolRegistry names EXACTLY (PascalCase from hello_companion)
fn action_step_to_tool_name(step: &ActionStep) -> &str {
    match step {
        ActionStep::MoveTo { .. } => "MoveTo",
        ActionStep::Approach { .. } => "Approach",
        ActionStep::Retreat { .. } => "Retreat",
        ActionStep::TakeCover { .. } => "TakeCover",
        ActionStep::Strafe { .. } => "Strafe",
        ActionStep::Patrol { .. } => "Patrol",
        ActionStep::Attack { .. } => "Attack",
        ActionStep::AimedShot { .. } => "AimedShot",
        ActionStep::QuickAttack { .. } => "QuickAttack",
        ActionStep::HeavyAttack { .. } => "HeavyAttack",
        ActionStep::AoEAttack { .. } => "AoEAttack",
        ActionStep::ThrowExplosive { .. } => "ThrowExplosive",
        ActionStep::CoverFire { .. } => "CoverFire",
        ActionStep::Charge { .. } => "Charge",
        ActionStep::Block { .. } => "Block",
        ActionStep::Dodge { .. } => "Dodge",
        ActionStep::Parry { .. } => "Parry",
        ActionStep::ThrowSmoke { .. } => "ThrowSmoke",
        ActionStep::Heal { .. } => "Heal",
        ActionStep::UseDefensiveAbility { .. } => "UseDefensiveAbility",
        ActionStep::EquipWeapon { .. } => "EquipWeapon",
        ActionStep::SwitchWeapon { .. } => "SwitchWeapon",
        ActionStep::Reload => "Reload",
        ActionStep::UseItem { .. } => "UseItem",
        ActionStep::DropItem { .. } => "DropItem",
        ActionStep::CallReinforcements { .. } => "CallReinforcements",
        ActionStep::MarkTarget { .. } => "MarkTarget",
        ActionStep::RequestCover { .. } => "RequestCover",
        ActionStep::CoordinateAttack { .. } => "CoordinateAttack",
        ActionStep::SetAmbush { .. } => "SetAmbush",
        ActionStep::Distract { .. } => "Distract",
        ActionStep::Regroup { .. } => "Regroup",
        ActionStep::Scan { .. } => "Scan",
        ActionStep::Wait { .. } => "Wait",
        ActionStep::Interact { .. } => "Interact",
        ActionStep::UseAbility { .. } => "UseAbility",
        ActionStep::Taunt { .. } => "Taunt",
        ActionStep::Throw { .. } => "Throw",   // Legacy
        ActionStep::Revive { .. } => "Revive", // Legacy
    }
}

// ============================================================================
// Utilities
// ============================================================================

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use astraweave_core::{Constraints, ToolSpec};
    use std::collections::BTreeMap;

    fn create_test_registry() -> ToolRegistry {
        ToolRegistry {
            tools: vec![
                ToolSpec {
                    name: "MoveTo".to_string(),
                    args: BTreeMap::new(),
                },
                ToolSpec {
                    name: "Attack".to_string(),
                    args: BTreeMap::new(),
                },
                ToolSpec {
                    name: "Heal".to_string(),
                    args: BTreeMap::new(),
                },
                ToolSpec {
                    name: "ThrowSmoke".to_string(),
                    args: BTreeMap::new(),
                },
            ],
            constraints: Constraints {
                enforce_cooldowns: false,
                enforce_los: false,
                enforce_stamina: false,
            },
        }
    }

    #[test]
    fn test_direct_parse_valid_json() {
        let reg = create_test_registry();
        let json = r#"{"plan_id": "test-1", "steps": [{"act": "MoveTo", "x": 5, "y": 10}]}"#;

        let result = parse_llm_response(json, &reg).unwrap();
        assert_eq!(result.extraction_method, ExtractionMethod::Direct);
        assert_eq!(result.plan.plan_id, "test-1");
        assert_eq!(result.plan.steps.len(), 1);
    }

    #[test]
    fn test_code_fence_extraction() {
        let reg = create_test_registry();
        let text = r#"
Here's my plan:
```json
{"plan_id": "test-2", "steps": [{"act": "Attack", "target_id": 42}]}
```
Hope that works!
"#;

        let result = parse_llm_response(text, &reg).unwrap();
        assert_eq!(result.extraction_method, ExtractionMethod::CodeFence);
        assert_eq!(result.plan.plan_id, "test-2");
    }

    #[test]
    fn test_envelope_extraction() {
        let reg = create_test_registry();
        let json = r#"{"message": {"content": "{\"plan_id\": \"test-3\", \"steps\": [{\"act\": \"Heal\"}]}"}}"#;

        let result = parse_llm_response(json, &reg).unwrap();
        assert_eq!(result.extraction_method, ExtractionMethod::Envelope);
        assert_eq!(result.plan.plan_id, "test-3");
    }

    #[test]
    fn test_hallucination_detection() {
        let reg = create_test_registry();
        let json = r#"{"plan_id": "test-bad", "steps": [{"act": "FlyToMoon", "altitude": 9999}]}"#;

        let result = parse_llm_response(json, &reg);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("unknown variant") || err_msg.contains("act"));
    }

    #[test]
    fn test_object_extraction() {
        let reg = create_test_registry();
        let text = r#"
Some text before
{"plan_id": "test-4", "steps": [{"act": "Attack", "target_id": 123}]}
Some text after
"#;

        let result = parse_llm_response(text, &reg).unwrap();
        assert_eq!(result.extraction_method, ExtractionMethod::ObjectExtraction);
        assert_eq!(result.plan.plan_id, "test-4");
    }

    #[test]
    fn test_tolerant_plan_id_variations() {
        let reg = create_test_registry();

        // Try alternative key names
        let variations = vec![
            r#"{"planId": "test-5", "steps": []}"#,
            r#"{"plan_no": "test-6", "steps": []}"#,
            r#"{"id": "test-7", "steps": []}"#,
        ];

        for json in variations {
            let result = parse_llm_response(json, &reg);
            assert!(result.is_ok(), "Failed to parse: {}", json);
        }
    }

    #[test]
    fn test_empty_steps_warning() {
        let reg = create_test_registry();
        let json = r#"{"plan_id": "test-empty", "steps": []}"#;

        let result = parse_llm_response(json, &reg).unwrap();
        assert_eq!(result.plan.steps.len(), 0);
        assert!(!result.validation_warnings.is_empty());
        assert!(result.validation_warnings[0].contains("no steps"));
    }

    #[test]
    fn test_malformed_json_fails() {
        let reg = create_test_registry();
        let bad_json = r#"{"plan_id": "test-bad", steps": [} // malformed"#;

        let result = parse_llm_response(bad_json, &reg);
        assert!(result.is_err());
    }

    #[test]
    fn test_non_json_text_fails() {
        let reg = create_test_registry();
        let text = "I can't generate a plan right now, sorry!";

        let result = parse_llm_response(text, &reg);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Failed to parse"));
    }
}
