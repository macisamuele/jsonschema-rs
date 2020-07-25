//! Validator for `format` keyword.
use crate::{
    compilation::context::CompilationContext,
    error::CompilationError,
    format::{jsonschema_formats::*, FormatValidatorBuilder},
    keywords::CompilationResult,
};
use serde_json::{Map, Value};

#[inline]
pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    if let Value::String(format) = schema {
        let draft_version = context.config.draft();
        match format.as_str() {
            "date-time" => DateTimeValidator::compile(draft_version),
            "date" => DateValidator::compile(draft_version),
            "email" => EmailValidator::compile(draft_version),
            "hostname" => HostnameValidator::compile(draft_version),
            "idn-email" => IDNEmailValidator::compile(draft_version),
            "idn-hostname" => IDNHostnameValidator::compile(draft_version),
            "ipv4" => IpV4Validator::compile(draft_version),
            "ipv6" => IpV6Validator::compile(draft_version),
            "iri-reference" => IRIReferenceValidator::compile(draft_version),
            "iri" => IRIValidator::compile(draft_version),
            "json-pointer" => JSONPointerValidator::compile(draft_version),
            "regex" => RegexValidator::compile(draft_version),
            "relative-json-pointer" => RelativeJSONPointerValidator::compile(draft_version),
            "time" => TimeValidator::compile(draft_version),
            "uri-reference" => URIReferenceValidator::compile(draft_version),
            "uri-template" => URITemplateValidator::compile(draft_version),
            "uri" => URIValidator::compile(draft_version),
            _ => None,
        }
    } else {
        Some(Err(CompilationError::SchemaError))
    }
}

#[cfg(test)]
mod tests {
    use crate::compilation::JSONSchema;
    use serde_json::json;

    #[test]
    fn ignored_format() {
        let schema = json!({"format": "custom", "type": "string"});
        let instance = json!("foo");
        let compiled = JSONSchema::compile(&schema).unwrap();
        assert!(compiled.is_valid(&instance))
    }
}
