//! Validator for `format` keyword.
use crate::{
    compilation::context::CompilationContext, error::CompilationError, keywords::CompilationResult,
};
use serde_json::{Map, Value};

#[inline]
pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    if let Value::String(format) = schema {
        if let Some(handler) = context.config.format_handler(format) {
            handler(context.config.draft())
        } else {
            None
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
