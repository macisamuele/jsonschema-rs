#[cfg(feature = "perfect_precision")]
use crate::perfect_precision_number::PerfectPrecisionNumber;
use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::{no_error, CompilationError, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    validator::Validate,
};
use serde_json::{Map, Value};
#[cfg(feature = "perfect_precision")]
use std::convert::TryFrom;
#[cfg(not(feature = "perfect_precision"))]
use std::f64::EPSILON;

#[cfg(feature = "perfect_precision")]
pub struct MultipleOfValidator {
    multiple_of: PerfectPrecisionNumber,
}
#[cfg(feature = "perfect_precision")]
impl MultipleOfValidator {
    #[inline]
    pub(crate) fn compile(multiple_of: PerfectPrecisionNumber) -> CompilationResult {
        Ok(Box::new(MultipleOfValidator { multiple_of }))
    }
}
#[cfg(feature = "perfect_precision")]
impl Validate for MultipleOfValidator {
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::multiple_of(instance, self.multiple_of.to_f64())
    }

    fn name(&self) -> String {
        format!("multipleOf: {}", self.multiple_of)
    }

    #[inline]
    fn is_valid_number(&self, schema: &JSONSchema, instance: &Value, instance_value: f64) -> bool {
        self.is_valid_perfect_precision_number(
            schema,
            instance,
            &PerfectPrecisionNumber::try_from(instance_value)
                .expect("A JSON float will always be a valid PerfectPrecisionNumber"),
        )
    }
    #[inline]
    fn is_valid_signed_integer(
        &self,
        schema: &JSONSchema,
        instance: &Value,
        instance_value: i64,
    ) -> bool {
        self.is_valid_perfect_precision_number(
            schema,
            instance,
            &PerfectPrecisionNumber::from(instance_value),
        )
    }
    #[inline]
    fn is_valid_unsigned_integer(
        &self,
        schema: &JSONSchema,
        instance: &Value,
        instance_value: u64,
    ) -> bool {
        self.is_valid_perfect_precision_number(
            schema,
            instance,
            &PerfectPrecisionNumber::from(instance_value),
        )
    }
    #[inline]
    fn is_valid_perfect_precision_number(
        &self,
        _: &JSONSchema,
        _: &Value,
        instance_value: &PerfectPrecisionNumber,
    ) -> bool {
        instance_value.is_multiple_of(&self.multiple_of)
    }
    #[inline]
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Number(instance_number) = instance {
            self.is_valid_perfect_precision_number(schema, instance, &instance_number.into())
        } else {
            true
        }
    }

    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Number(instance_number) = instance {
            self.validate_perfect_precision_number(schema, instance, &instance_number.into())
        } else {
            no_error()
        }
    }
}

#[cfg(not(feature = "perfect_precision"))]
pub struct MultipleOfFloatValidator {
    multiple_of: f64,
}
#[cfg(not(feature = "perfect_precision"))]
impl MultipleOfFloatValidator {
    #[inline]
    pub(crate) fn compile(multiple_of: f64) -> CompilationResult {
        Ok(Box::new(MultipleOfFloatValidator { multiple_of }))
    }
}
#[cfg(not(feature = "perfect_precision"))]
impl Validate for MultipleOfFloatValidator {
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::multiple_of(instance, self.multiple_of)
    }

    fn name(&self) -> String {
        format!("multipleOf: {}", self.multiple_of)
    }

    #[inline]
    fn is_valid_number(&self, _: &JSONSchema, _: &Value, instance_value: f64) -> bool {
        let remainder = (instance_value / self.multiple_of) % 1.;
        remainder < EPSILON && remainder < (1. - EPSILON)
    }
    #[inline]
    fn is_valid_signed_integer(
        &self,
        schema: &JSONSchema,
        instance: &Value,
        instance_value: i64,
    ) -> bool {
        #[allow(clippy::cast_precision_loss)]
        self.is_valid_number(schema, instance, instance_value as f64)
    }
    #[inline]
    fn is_valid_unsigned_integer(
        &self,
        schema: &JSONSchema,
        instance: &Value,
        instance_value: u64,
    ) -> bool {
        #[allow(clippy::cast_precision_loss)]
        self.is_valid_number(schema, instance, instance_value as f64)
    }
    #[inline]
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Some(instance_value) = instance.as_f64() {
            self.is_valid_number(schema, instance, instance_value)
        } else {
            true
        }
    }

    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Some(instance_value) = instance.as_f64() {
            self.validate_number(schema, instance, instance_value)
        } else {
            no_error()
        }
    }
}

#[cfg(not(feature = "perfect_precision"))]
pub struct MultipleOfIntegerValidator {
    multiple_of: f64,
}

#[cfg(not(feature = "perfect_precision"))]
impl MultipleOfIntegerValidator {
    #[inline]
    pub(crate) fn compile(multiple_of: f64) -> CompilationResult {
        Ok(Box::new(MultipleOfIntegerValidator { multiple_of }))
    }
}

#[cfg(not(feature = "perfect_precision"))]
impl Validate for MultipleOfIntegerValidator {
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::multiple_of(instance, self.multiple_of)
    }

    fn name(&self) -> String {
        format!("multipleOf: {}", self.multiple_of)
    }

    #[inline]
    fn is_valid_number(&self, _: &JSONSchema, _: &Value, instance_value: f64) -> bool {
        if instance_value.fract() == 0. {
            (instance_value % self.multiple_of) == 0.
        } else {
            let remainder = (instance_value / self.multiple_of) % 1.;
            remainder < EPSILON && remainder < (1. - EPSILON)
        }
    }
    #[inline]
    fn is_valid_signed_integer(
        &self,
        schema: &JSONSchema,
        instance: &Value,
        instance_value: i64,
    ) -> bool {
        #[allow(clippy::cast_precision_loss)]
        self.is_valid_number(schema, instance, instance_value as f64)
    }
    #[inline]
    fn is_valid_unsigned_integer(
        &self,
        schema: &JSONSchema,
        instance: &Value,
        instance_value: u64,
    ) -> bool {
        #[allow(clippy::cast_precision_loss)]
        self.is_valid_number(schema, instance, instance_value as f64)
    }
    #[inline]
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Some(instance_value) = instance.as_f64() {
            self.is_valid_number(schema, instance, instance_value)
        } else {
            true
        }
    }

    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Some(instance_value) = instance.as_f64() {
            self.validate_number(schema, instance, instance_value)
        } else {
            no_error()
        }
    }
}

#[inline]
pub fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    if let Value::Number(multiple_of) = schema {
        #[cfg(feature = "perfect_precision")]
        {
            return Some(MultipleOfValidator::compile(multiple_of.into()));
        }
        #[cfg(not(feature = "perfect_precision"))]
        {
            let multiple_of = multiple_of.as_f64().expect("Always valid");
            return if multiple_of.fract() == 0. {
                Some(MultipleOfIntegerValidator::compile(multiple_of))
            } else {
                Some(MultipleOfFloatValidator::compile(multiple_of))
            };
        }
    }
    Some(Err(CompilationError::SchemaError))
}
