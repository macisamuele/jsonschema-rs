use std::{
    cmp::Ordering,
    convert::TryFrom,
    error::Error,
    fmt,
    hash::{Hash, Hasher},
    str::FromStr,
};

use serde_json::{Number, Value};

/// Perfectly represent the input number. It does so by using arbitrary arithmethic libraries
///
/// JSON Number are always read from a file/string/whatever limited stream of bytes so there is no concept
/// of precision loss due to math oprations. As this is the case we can transform whatever input number
/// into it's integer or rational form.
/// This allows us to be able to process numbers without any limts on their bit size of float approximations
/// NOTE: The linking of this enum into the project requires the usage of `serde_json` with `arbitrary_precision`
///     enabled in order to have `Value::Number(...).to_string()` represent exactly the content of the input
///     JSON and not the result of its processing
#[derive(Clone, Debug)]
pub enum PerfectPrecisionNumber {
    Integer(rug::Integer),
    IntegerFromFloat(rug::Integer),
    Rational(rug::Rational),
}

impl fmt::Display for PerfectPrecisionNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(integer) | Self::IntegerFromFloat(integer) => write!(f, "{}", integer),
            Self::Rational(rational) => write!(f, "{}", rational.to_f64()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum PerfectPrecisionNumberError {
    Invalid(&'static str),
}

impl fmt::Display for PerfectPrecisionNumberError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(human_readable_reason) => {
                write!(f, "Invalid value: {}", human_readable_reason)
            }
        }
    }
}

impl Error for PerfectPrecisionNumberError {}

impl FromStr for PerfectPrecisionNumber {
    type Err = PerfectPrecisionNumberError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if let Ok(integer) = value.parse::<rug::Integer>() {
            Ok(Self::Integer(integer))
        } else {
            let mut characters = value.chars().peekable();
            let mut found_decimal_point = false;
            let mut only_zeros_after_decimal_point = true;

            let mut numerator = rug::Integer::from(0);
            let mut denominator = if characters.peek() == Some(&'-') {
                let _ = characters.next(); // Consume the character from the iterator as it was a '-' sign
                rug::Integer::from(-1)
            } else {
                rug::Integer::from(1)
            };
            for char_ in characters {
                if char_ == '.' {
                    if found_decimal_point {
                        return Err(PerfectPrecisionNumberError::Invalid(
                            "Multiple decimal points in the input string",
                        ));
                    }
                    found_decimal_point = true;
                } else if let Some(digit) = char_.to_digit(10) {
                    numerator *= 10;
                    numerator += digit;
                    if found_decimal_point {
                        denominator *= 10;
                        if digit != 0 {
                            only_zeros_after_decimal_point = false;
                        }
                    }
                } else {
                    return Err(PerfectPrecisionNumberError::Invalid(
                        "Invalid digit (accepted digits in [0-9])",
                    ));
                }
            }
            if denominator.to_u8() == Some(1) {
                if found_decimal_point {
                    Ok(Self::IntegerFromFloat(numerator))
                } else {
                    Ok(Self::Integer(numerator))
                }
            } else if only_zeros_after_decimal_point {
                numerator /= denominator;
                Ok(Self::IntegerFromFloat(numerator))
            } else {
                Ok(Self::Rational(rug::Rational::from((
                    numerator,
                    denominator,
                ))))
            }
        }
    }
}

macro_rules! from_int {
    ($($t:ty),*$(,)*) => {
        $(impl From<$t> for PerfectPrecisionNumber {
            #[inline]
            fn from(value: $t) -> Self {
                Self::Integer(rug::Integer::from(value))
            }
        })*
    };
}
from_int!(i8, u8, i16, u16, i32, u32, i64, u64, i128, u128);

macro_rules! from_float {
    ($($t:ty),*$(,)?) => {
        $(paste::item! {
            impl TryFrom<$t> for PerfectPrecisionNumber {
                type Error = PerfectPrecisionNumberError;
                #[inline]
                fn try_from(value: $t) -> Result<Self, Self::Error> {
                    if let Some(value_rational) = rug::Rational::[<from_ $t>](value) {
                        if value.fract() == 0.0 {
                            Ok(Self::IntegerFromFloat(value_rational.numer().clone()))
                        } else {
                            Ok(Self::Rational(value_rational))
                        }
                    } else if value.is_infinite() {
                        Err(PerfectPrecisionNumberError::Invalid("Infinite numbers are not managed"))
                    } else {
                        Err(PerfectPrecisionNumberError::Invalid("NaN numbers are not managed"))
                    }
                }
            }
        })*
    };
}
from_float!(f32, f64);

impl TryFrom<&Value> for PerfectPrecisionNumber {
    type Error = PerfectPrecisionNumberError;

    #[inline]
    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        if let Value::Number(value_number) = value {
            value_number.to_string().parse()
        } else {
            Err(PerfectPrecisionNumberError::Invalid(
                "value is not a number",
            ))
        }
    }
}

impl From<&Number> for PerfectPrecisionNumber {
    #[inline]
    fn from(value: &Number) -> Self {
        value
            .to_string()
            .parse()
            .expect("A JSON number will always be representable as PefectPrecisionNumber")
    }
}

impl PartialEq<Self> for PerfectPrecisionNumber {
    #[must_use]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Integer(self_int), Self::Integer(other_int))
            | (Self::Integer(self_int), Self::IntegerFromFloat(other_int))
            | (Self::IntegerFromFloat(self_int), Self::Integer(other_int))
            | (Self::IntegerFromFloat(self_int), Self::IntegerFromFloat(other_int)) => {
                self_int == other_int
            }
            (Self::Rational(self_rational), Self::Rational(other_rational)) => {
                self_rational == other_rational
            }
            _ => false,
        }
    }
}

impl Eq for PerfectPrecisionNumber {}

impl PartialOrd<Self> for PerfectPrecisionNumber {
    #[must_use]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::Integer(self_int), Self::Integer(other_int))
            | (Self::IntegerFromFloat(self_int), Self::Integer(other_int))
            | (Self::Integer(self_int), Self::IntegerFromFloat(other_int))
            | (Self::IntegerFromFloat(self_int), Self::IntegerFromFloat(other_int)) => {
                Some(self_int.cmp(other_int))
            }
            (Self::Integer(self_int), Self::Rational(other_rational))
            | (Self::IntegerFromFloat(self_int), Self::Rational(other_rational)) => {
                // TODO: Can we remove Rational allocation?
                Some(rug::Rational::from(self_int).cmp(other_rational))
            }
            (Self::Rational(self_rational), Self::Rational(other_rational)) => {
                Some(self_rational.cmp(other_rational))
            }
            (Self::Rational(self_rational), Self::Integer(other_int))
            | (Self::Rational(self_rational), Self::IntegerFromFloat(other_int)) => {
                // TODO: Can we remove Rational allocation?
                Some(self_rational.cmp(&rug::Rational::from(other_int)))
            }
        }
    }
}

impl Ord for PerfectPrecisionNumber {
    #[must_use]
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other)
            .expect("The implementation is never returning None, so we're safe")
    }
}

impl Hash for PerfectPrecisionNumber {
    #[inline]
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        match self {
            Self::Integer(integer) => (0, integer).hash(hasher),
            Self::IntegerFromFloat(integer) => (1, integer).hash(hasher),
            Self::Rational(rational) => rational.hash(hasher),
        }
    }
}

impl PerfectPrecisionNumber {
    pub(crate) fn is_multiple_of(&self, number: &Self) -> bool {
        // Let's do some math `self` is multiple of `number` if `self`/`number` = integer
        // The different assumptions and checks will be done on case-per-case
        match (self, &number) {
            (Self::Integer(self_int), Self::Integer(number_int))
            | (Self::Integer(self_int), Self::IntegerFromFloat(number_int))
            | (Self::IntegerFromFloat(self_int), Self::Integer(number_int))
            | (Self::IntegerFromFloat(self_int), Self::IntegerFromFloat(number_int)) => {
                self_int.is_divisible(number_int)
            }
            (Self::Integer(self_int), Self::Rational(number_rational))
            | (Self::IntegerFromFloat(self_int), Self::Rational(number_rational)) => {
                // a = self_int         b/c = number_rational
                // a / (b/c) = ac / b
                // As we know that b/c was canonicalised then there are no common factors
                // so the only way to have ac / b as integer is if a is divisible by b
                self_int.is_divisible(number_rational.numer())
            }
            (Self::Rational(self_rational), Self::Integer(_))
            | (Self::Rational(self_rational), Self::IntegerFromFloat(_)) => {
                // Using assertion to ensure that PerfectPrecisionNumber::Rational
                // will have a denominator different than 1. It is guaranteed by the
                // FromStr and From<f(32|64)> methods. Using debug_assert to avoid
                // adding an overhead on release build
                debug_assert_ne!(self_rational.denom().to_u8(), Some(1));
                // As we know that a rational number (with denominator different than 1)
                // cannot be a multiple than an integer number then
                false
            }
            (Self::Rational(self_rational), Self::Rational(number_rational)) => {
                // a/b = self_rational  c/d = number_rational
                // (a/b) / (c/d) = ac / bd => it's divisible if ac/bd can be canonicalised as e/1
                // TODO: Is there a way to get it without initialising a new rational?
                rug::Rational::from(self_rational / number_rational)
                    .denom()
                    .to_u8()
                    == Some(1)
            }
        }
    }

    pub(crate) fn to_f64(&self) -> f64 {
        match self {
            Self::Integer(integer) | Self::IntegerFromFloat(integer) => integer.to_f64(),
            Self::Rational(rational) => rational.to_f64(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PerfectPrecisionNumber, PerfectPrecisionNumberError};
    use serde_json::{from_str, Value};
    use std::{cmp::Ordering, convert::TryInto, fmt::Debug};
    use test_case::test_case;

    #[test_case("1" => Ok(PerfectPrecisionNumber::Integer(rug::Integer::from(1))))]
    #[test_case("-2" => Ok(PerfectPrecisionNumber::Integer(rug::Integer::from(-2))))]
    #[test_case("3." => Ok(PerfectPrecisionNumber::IntegerFromFloat(rug::Integer::from(3))))]
    #[test_case("-4." => Ok(PerfectPrecisionNumber::IntegerFromFloat(rug::Integer::from(-4))))]
    #[test_case("5.000" => Ok(PerfectPrecisionNumber::IntegerFromFloat(rug::Integer::from(5))))]
    #[test_case("-6.000" => Ok(PerfectPrecisionNumber::IntegerFromFloat(rug::Integer::from(-6))))]
    #[test_case("0.7" => Ok(PerfectPrecisionNumber::Rational(rug::Rational::from((7, 10)))))]
    #[test_case("-0.8" => Ok(PerfectPrecisionNumber::Rational(rug::Rational::from((-4, 5)))))]
    #[test_case(".9" => Ok(PerfectPrecisionNumber::Rational(rug::Rational::from((9, 10)))))]
    #[test_case("-.11" => Ok(PerfectPrecisionNumber::Rational(rug::Rational::from((-11, 100)))))]
    #[test_case("F" => Err(PerfectPrecisionNumberError::Invalid("Invalid digit (accepted digits in [0-9])")))]
    #[test_case("0..5" => Err(PerfectPrecisionNumberError::Invalid("Multiple decimal points in the input string")))]
    fn test_parse_str(value: &str) -> Result<PerfectPrecisionNumber, PerfectPrecisionNumberError> {
        value.parse::<PerfectPrecisionNumber>()
    }

    #[test_case(f32::INFINITY => PerfectPrecisionNumberError::Invalid("Infinite numbers are not managed"))]
    #[test_case(f32::NAN => PerfectPrecisionNumberError::Invalid("NaN numbers are not managed"))]
    #[test_case(f64::INFINITY => PerfectPrecisionNumberError::Invalid("Infinite numbers are not managed"))]
    #[test_case(f64::NAN => PerfectPrecisionNumberError::Invalid("NaN numbers are not managed"))]
    fn test_try_from_float_edge_cases<PPN: TryInto<PerfectPrecisionNumber>>(
        value: PPN,
    ) -> PPN::Error {
        value.try_into().unwrap_err()
    }

    #[test_case(1, 2 => false)]
    #[test_case(2, 1 => true)]
    #[test_case(3, 2 => false)]
    #[test_case(6, 2 => true)]
    #[test_case(1, 0.5 => true)]
    #[test_case(1, 0.75 => false)]
    #[test_case(0.5, 1 => false)]
    #[test_case(4, 2 => true)]
    #[test_case(0.5, 0.75 => false)]
    #[test_case(1.5, 0.75 => true)]
    fn test_is_multiple_of<
        PPN1: TryInto<PerfectPrecisionNumber>,
        PPN2: TryInto<PerfectPrecisionNumber>,
    >(
        number: PPN1,
        multiple_of: PPN2,
    ) -> bool
    where
        PPN1::Error: Debug,
        PPN2::Error: Debug,
    {
        let number_: PerfectPrecisionNumber = number.try_into().unwrap();
        let multiple_of_: PerfectPrecisionNumber = multiple_of.try_into().unwrap();
        number_.is_multiple_of(&multiple_of_)
    }

    #[test_case("1" => PerfectPrecisionNumber::Integer(1.into()))]
    #[test_case("3.0" => PerfectPrecisionNumber::IntegerFromFloat(rug::Integer::from(3)))]
    // 2^200 = 1606938044258990275541962092341162602522202993782792835301376
    #[test_case("1606938044258990275541962092341162602522202993782792835301376" => PerfectPrecisionNumber::Integer(rug::Integer::from_str_radix("1606938044258990275541962092341162602522202993782792835301376", 10).unwrap()))]
    #[test_case("1.5" => PerfectPrecisionNumber::Rational(rug::Rational::from((3,2))))]
    #[test_case("1.234567890123456789012345678901234567890123456789" => PerfectPrecisionNumber::Rational(rug::Rational::from((
        rug::Integer::from_str_radix("1234567890123456789012345678901234567890123456789", 10).unwrap(), rug::Integer::from_str_radix("1000000000000000000000000000000000000000000000000", 10).unwrap()
    ))))]
    fn test_load_from_json_string(json_str: &str) -> PerfectPrecisionNumber {
        let json_value: Value = from_str(json_str).unwrap();
        (&json_value).try_into().unwrap()
    }

    #[test_case(1, 2 => Ordering::Less)]
    #[test_case(3, 3 => Ordering::Equal)]
    #[test_case(5, 4 => Ordering::Greater)]
    #[test_case(0.6, 0.7 => Ordering::Less)]
    #[test_case(0.8, 0.8 => Ordering::Equal)]
    #[test_case(0.11, 0.09 => Ordering::Greater)]
    #[test_case(0.9, 1 => Ordering::Less)]
    #[test_case(1.9, 1 => Ordering::Greater)]
    #[test_case(3, 3.1 => Ordering::Less)]
    #[test_case(4, 3.1 => Ordering::Greater)]
    fn test_ordering<PPN1: TryInto<PerfectPrecisionNumber>, PPN2: TryInto<PerfectPrecisionNumber>>(
        value1: PPN1,
        value2: PPN2,
    ) -> Ordering
    where
        PPN1::Error: Debug,
        PPN2::Error: Debug,
    {
        let value1_: PerfectPrecisionNumber = value1.try_into().unwrap();
        let value2_: PerfectPrecisionNumber = value2.try_into().unwrap();
        value1_.cmp(&value2_)
    }
}
