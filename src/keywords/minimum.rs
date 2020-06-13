#[cfg(feature = "perfect_precision")]
use crate::perfect_precision_number::PerfectPrecisionNumber;
use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::{no_error, CompilationError, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    validator::Validate,
};
#[cfg(not(feature = "perfect_precision"))]
use num_cmp::NumCmp;
use serde_json::{Map, Value};
#[cfg(feature = "perfect_precision")]
use std::convert::TryFrom;

#[cfg(feature = "perfect_precision")]
pub struct MinimumValidator {
    limit: PerfectPrecisionNumber,
}
#[cfg(not(feature = "perfect_precision"))]
pub struct MinimumU64Validator {
    limit: u64,
}
#[cfg(not(feature = "perfect_precision"))]
pub struct MinimumI64Validator {
    limit: i64,
}
#[cfg(not(feature = "perfect_precision"))]
pub struct MinimumF64Validator {
    limit: f64,
}

macro_rules! validate {
    ($validator: ty) => {
        impl Validate for $validator {
            #[inline]
            fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
                #[cfg(feature = "perfect_precision")]
                {
                    ValidationError::minimum(instance, self.limit.to_f64())
                }
                #[cfg(not(feature = "perfect_precision"))]
                {
                    #[allow(trivial_numeric_casts)]
                    ValidationError::minimum(instance, self.limit as f64)
                }
            }

            fn name(&self) -> String {
                format!("exclusiveMinimum: {}", self.limit)
            }

            #[inline]
            fn is_valid_number(&self, _: &JSONSchema, _: &Value, instance_value: f64) -> bool {
                #[cfg(feature = "perfect_precision")]
                {
                    &PerfectPrecisionNumber::try_from(instance_value)
                        .expect("A JSON float will always be a valid PerfectPrecisionNumber")
                        >= &self.limit
                }
                #[cfg(not(feature = "perfect_precision"))]
                {
                    NumCmp::num_ge(instance_value, self.limit)
                }
            }
            #[inline]
            fn is_valid_signed_integer(
                &self,
                _: &JSONSchema,
                _: &Value,
                instance_value: i64,
            ) -> bool {
                #[cfg(feature = "perfect_precision")]
                {
                    &PerfectPrecisionNumber::from(instance_value) >= &self.limit
                }
                #[cfg(not(feature = "perfect_precision"))]
                {
                    NumCmp::num_ge(instance_value, self.limit)
                }
            }
            #[inline]
            fn is_valid_unsigned_integer(
                &self,
                _: &JSONSchema,
                _: &Value,
                instance_value: u64,
            ) -> bool {
                #[cfg(feature = "perfect_precision")]
                {
                    &PerfectPrecisionNumber::from(instance_value) >= &self.limit
                }
                #[cfg(not(feature = "perfect_precision"))]
                {
                    NumCmp::num_ge(instance_value, self.limit)
                }
            }
            #[cfg(feature = "perfect_precision")]
            #[inline]
            fn is_valid_perfect_precision_number(
                &self,
                _: &JSONSchema,
                _: &Value,
                instance_value: &PerfectPrecisionNumber,
            ) -> bool {
                instance_value >= &self.limit
            }
            #[cfg(feature = "perfect_precision")]
            #[inline]
            fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
                if let Value::Number(instance_number) = instance {
                    self.is_valid_perfect_precision_number(
                        schema,
                        instance,
                        &instance_number.into(),
                    )
                } else {
                    true
                }
            }
            #[cfg(not(feature = "perfect_precision"))]
            #[inline]
            fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
                if let Some(instance_value) = instance.as_u64() {
                    self.is_valid_unsigned_integer(schema, instance, instance_value)
                } else if let Some(instance_value) = instance.as_i64() {
                    self.is_valid_signed_integer(schema, instance, instance_value)
                } else if let Some(instance_value) = instance.as_f64() {
                    self.is_valid_number(schema, instance, instance_value)
                } else {
                    true
                }
            }

            #[cfg(feature = "perfect_precision")]
            #[inline]
            fn validate<'a>(
                &self,
                schema: &'a JSONSchema,
                instance: &'a Value,
            ) -> ErrorIterator<'a> {
                if let Value::Number(instance_number) = instance {
                    self.validate_perfect_precision_number(
                        schema,
                        instance,
                        &instance_number.into(),
                    )
                } else {
                    no_error()
                }
            }
            #[cfg(not(feature = "perfect_precision"))]
            #[inline]
            fn validate<'a>(
                &self,
                schema: &'a JSONSchema,
                instance: &'a Value,
            ) -> ErrorIterator<'a> {
                if let Value::Number(instance_number) = instance {
                    if let Some(instance_unsigned_integer) = instance_number.as_u64() {
                        self.validate_unsigned_integer(schema, instance, instance_unsigned_integer)
                    } else if let Some(instance_signed_integer) = instance_number.as_i64() {
                        self.validate_signed_integer(schema, instance, instance_signed_integer)
                    } else {
                        self.validate_number(
                            schema,
                            instance,
                            instance_number
                                .as_f64()
                                .expect("A JSON number will always be representable as f64"),
                        )
                    }
                } else {
                    no_error()
                }
            }
        }
    };
}

#[cfg(feature = "perfect_precision")]
validate!(MinimumValidator);
#[cfg(not(feature = "perfect_precision"))]
validate!(MinimumU64Validator);
#[cfg(not(feature = "perfect_precision"))]
validate!(MinimumI64Validator);
#[cfg(not(feature = "perfect_precision"))]
validate!(MinimumF64Validator);

#[inline]
pub fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    #[cfg(feature = "perfect_precision")]
    {
        if let Value::Number(limit) = schema {
            return Some(Ok(Box::new(MinimumValidator {
                limit: limit.into(),
            })));
        }
    }
    #[cfg(not(feature = "perfect_precision"))]
    {
        if let Value::Number(limit) = schema {
            return if let Some(limit) = limit.as_u64() {
                Some(Ok(Box::new(MinimumU64Validator { limit })))
            } else if let Some(limit) = limit.as_i64() {
                Some(Ok(Box::new(MinimumI64Validator { limit })))
            } else {
                let limit = limit.as_f64().expect("Always valid");
                Some(Ok(Box::new(MinimumF64Validator { limit })))
            };
        }
    }
    Some(Err(CompilationError::SchemaError))
}

#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(json!({"minimum": 1u64 << 54}), json!(1u64 << 54 - 1))]
    #[test_case(json!({"minimum": 1i64 << 54}), json!(1i64 << 54 - 1))]
    fn is_not_valid(schema: Value, instance: Value) {
        tests_util::is_not_valid(schema, instance)
    }
}
