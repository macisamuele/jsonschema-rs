use crate::{
    compilation::JSONSchema, error::ValidationError, keywords::CompilationResult, schemas::Draft,
    validator::Validate,
};
use serde_json::{Map, Value};

/// General interface to implement user-defined `format` keyword validators
pub trait FormatValidator: Sync + Send {
    /// Instanciate a new instance of the Validator
    fn new() -> Self
    where
        Self: Sized;

    /// Determine whether the format should be applicable to the provided `draft_version`
    ///
    /// This hook is provided because not all the format handlers should be applicated
    /// to all the draft versions (for library provided ones) or on special-circumstances
    /// on user defined formats.
    #[inline]
    fn supported_for_draft(_draft_version: Draft) -> bool
    where
        Self: Sized,
    {
        true
    }

    /// Name of the format keyword to handle
    fn format_name(&self) -> &'static str;

    /// Check if the incoming array instance is valid according to the format
    fn check_array(&self, _: &[Value]) -> bool {
        true
    }
    /// Check if the incoming boolean instance is valid according to the format
    fn check_boolean(&self, _: bool) -> bool {
        true
    }
    /// Check if the incoming float instance is valid according to the format
    fn check_float(&self, _: f64) -> bool {
        true
    }
    /// Check if `Value::Null` is valid according to the format
    fn check_null(&self) -> bool {
        true
    }
    /// Check if the incoming object instance is valid according to the format
    fn check_object(&self, _: &Map<String, Value>) -> bool {
        true
    }
    /// Check if the incoming signed integer instance is valid according to the format
    fn check_signed_integer(&self, _: i64) -> bool {
        true
    }
    /// Check if the incoming string instance is valid according to the format
    fn check_string(&self, _: &str) -> bool {
        true
    }
    /// Check if the incoming unsigned integer instance is valid according to the format
    fn check_unsigned_integer(&self, _: u64) -> bool {
        true
    }

    /// Default implementaton of the `ToString::to_string` method
    #[inline]
    fn default_to_string(&self) -> String {
        format!("format: {}", self.format_name())
    }
}

pub(crate) trait FormatValidatorBuilder: FormatValidator + Sized {
    fn compile(draft_version: Draft) -> Option<CompilationResult>
    where
        Self: 'static + Validate,
    {
        if Self::supported_for_draft(draft_version) {
            Some(Ok(Box::new(Self::new())))
        } else {
            None
        }
    }
}

impl<T: FormatValidator> FormatValidatorBuilder for T {}

impl<T: FormatValidator + ToString> Validate for T {
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::format(instance, self.format_name())
    }

    #[inline]
    fn is_valid_array(&self, _: &JSONSchema, _: &Value, instance_value: &[Value]) -> bool {
        self.check_array(instance_value)
    }
    #[inline]
    fn is_valid_boolean(&self, _: &JSONSchema, _: &Value, instance_value: bool) -> bool {
        self.check_boolean(instance_value)
    }
    #[inline]
    fn is_valid_object(
        &self,
        _: &JSONSchema,
        _: &Value,
        instance_value: &Map<String, Value>,
    ) -> bool {
        self.check_object(instance_value)
    }
    #[inline]
    fn is_valid_null(&self, _: &JSONSchema, _: &Value, _: ()) -> bool {
        self.check_null()
    }
    #[inline]
    fn is_valid_number(&self, _: &JSONSchema, _: &Value, instance_value: f64) -> bool {
        self.check_float(instance_value)
    }
    #[inline]
    fn is_valid_signed_integer(&self, _: &JSONSchema, _: &Value, instance_value: i64) -> bool {
        self.check_signed_integer(instance_value)
    }
    #[inline]
    fn is_valid_string(&self, _: &JSONSchema, _: &Value, instance_value: &str) -> bool {
        self.check_string(instance_value)
    }
    #[inline]
    fn is_valid_unsigned_integer(&self, _: &JSONSchema, _: &Value, instance_value: u64) -> bool {
        self.check_unsigned_integer(instance_value)
    }
}
