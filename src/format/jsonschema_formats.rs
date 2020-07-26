use crate::{
    format::{FormatValidator, FormatValidatorBuilder},
    keywords::CompilationResult,
    Draft,
};
use chrono::{DateTime, NaiveDate};
use regex::Regex;
use std::{collections::HashMap, net::IpAddr, str::FromStr};
use url::Url;

pub(crate) type FormatHandlerType = fn(Draft) -> Option<CompilationResult>;

lazy_static::lazy_static! {
    static ref IRI_REFERENCE_RE: Regex =
        Regex::new(r"^(\w+:(/?/?))?[^#\\\s]*(#[^\\\s]*)?\z").expect("Is a valid regex");
    static ref JSON_POINTER_RE: Regex = Regex::new(r"^(/(([^/~])|(~[01]))*)*\z").expect("Is a valid regex");
    static ref RELATIVE_JSON_POINTER_RE: Regex =
        Regex::new(r"^(?:0|[1-9][0-9]*)(?:#|(?:/(?:[^~/]|~0|~1)*)*)\z").expect("Is a valid regex");
    static ref TIME_RE: Regex =
        Regex::new(
        r"^([01][0-9]|2[0-3]):([0-5][0-9]):([0-5][0-9])(\.[0-9]{6})?(([Zz])|([+|\-]([01][0-9]|2[0-3]):[0-5][0-9]))\z",
    ).expect("Is a valid regex");
    static ref URI_REFERENCE_RE: Regex =
        Regex::new(r"^(\w+:(/?/?))?[^#\\\s]*(#[^\\\s]*)?\z").expect("Is a valid regex");
    static ref URI_TEMPLATE_RE: Regex = Regex::new(
        r#"^(?:(?:[^\x00-\x20"'<>%\\^`{|}]|%[0-9a-f]{2})|\{[+#./;?&=,!@|]?(?:[a-z0-9_]|%[0-9a-f]{2})+(?::[1-9][0-9]{0,3}|\*)?(?:,(?:[a-z0-9_]|%[0-9a-f]{2})+(?::[1-9][0-9]{0,3}|\*)?)*})*\z"#
    )
    .expect("Is a valid regex");
}

macro_rules! impl_string_formatter {
    ($validator:ident, $format_name:tt, $check:expr) => {
        impl_string_formatter![$validator, $format_name, $check, |_| true];
    };

    ($validator:ident, $format_name:tt, $check:expr, $supported_draft_check:expr) => {
        pub(crate) struct $validator;
        impl FormatValidator for $validator {
            fn new() -> Self {
                Self
            }

            #[inline]
            fn supported_for_draft(draft_version: Draft) -> bool
            where
                Self: Sized,
            {
                $supported_draft_check(draft_version)
            }

            #[inline]
            fn format_name(&self) -> &'static str {
                $format_name
            }

            fn check_string(&self, instance_value: &str) -> bool {
                $check(instance_value)
            }
        }
        impl ToString for $validator {
            #[inline]
            fn to_string(&self) -> String {
                self.default_to_string()
            }
        }
    };
}

#[inline]
fn is_valid_email(string: &str) -> bool {
    string.contains('@')
}
#[inline]
fn is_valid_hostname(string: &str) -> bool {
    !(string.ends_with('-')
        || string.starts_with('-')
        || string.is_empty()
        || string.chars().count() > 255
        || string
            .chars()
            .any(|c| !(c.is_alphanumeric() || c == '-' || c == '.'))
        || string.split('.').any(|part| part.chars().count() > 63))
}
#[inline]
fn is_valid_idn_hostname(string: &str) -> bool {
    is_valid_hostname(string) && idna::domain_to_unicode(string).1.is_ok()
}

impl_string_formatter!(DateTimeValidator, "date-time", |instance_string| {
    DateTime::parse_from_rfc3339(instance_string).is_ok()
});
impl_string_formatter!(DateValidator, "date", |instance_string| {
    NaiveDate::parse_from_str(instance_string, "%Y-%m-%d").is_ok()
});
impl_string_formatter!(EmailValidator, "email", is_valid_email);
impl_string_formatter!(HostnameValidator, "hostname", is_valid_hostname);
impl_string_formatter!(IDNEmailValidator, "idn-email", is_valid_email);
impl_string_formatter!(
    IDNHostnameValidator,
    "idn-hostname",
    is_valid_idn_hostname,
    |draft_version| draft_version >= Draft::Draft7
);
impl_string_formatter!(IpV4Validator, "ipv4", |instance_string| {
    if let Ok(IpAddr::V4(_)) = IpAddr::from_str(instance_string) {
        true
    } else {
        false
    }
});
impl_string_formatter!(IpV6Validator, "ipv6", |instance_string| {
    if let Ok(IpAddr::V6(_)) = IpAddr::from_str(instance_string) {
        true
    } else {
        false
    }
});
impl_string_formatter!(
    IRIValidator,
    "iri",
    |instance_string| Url::from_str(instance_string).is_ok(),
    |draft_version| draft_version >= Draft::Draft7
);
impl_string_formatter!(
    IRIReferenceValidator,
    "iri-reference",
    |instance_value| IRI_REFERENCE_RE.is_match(instance_value),
    |draft_version| draft_version >= Draft::Draft7
);
impl_string_formatter!(
    JSONPointerValidator,
    "json-pointer",
    |instance_value| JSON_POINTER_RE.is_match(instance_value),
    |draft_version| draft_version >= Draft::Draft6
);
impl_string_formatter!(RegexValidator, "regex", |instance_value| {
    Regex::new(instance_value).is_ok()
});
impl_string_formatter!(
    RelativeJSONPointerValidator,
    "relative-json-pointer",
    |instance_value| RELATIVE_JSON_POINTER_RE.is_match(instance_value),
    |draft_version| draft_version >= Draft::Draft7
);
impl_string_formatter!(TimeValidator, "time", |instance_value| {
    TIME_RE.is_match(instance_value)
});
impl_string_formatter!(URIValidator, "uri", |instance_string| {
    Url::from_str(instance_string).is_ok()
});
impl_string_formatter!(
    URIReferenceValidator,
    "uri-reference",
    |instance_value| URI_REFERENCE_RE.is_match(instance_value),
    |draft_version| draft_version >= Draft::Draft6
);
impl_string_formatter!(
    URITemplateValidator,
    "uri-template",
    |instance_value| URI_TEMPLATE_RE.is_match(instance_value),
    |draft_version| draft_version >= Draft::Draft6
);

lazy_static::lazy_static! {
    pub(crate) static ref DEFAULT_FORMAT_HANDLERS: HashMap<&'static str, FormatHandlerType> = {
        let mut map: HashMap<&'static str, FormatHandlerType> = HashMap::with_capacity(17);
        map.insert("date", DateValidator::compile);
        map.insert("date-time", DateTimeValidator::compile);
        map.insert("email", EmailValidator::compile);
        map.insert("hostname", HostnameValidator::compile);
        map.insert("idn-email", IDNEmailValidator::compile);
        map.insert("idn-hostname", IDNHostnameValidator::compile);
        map.insert("ipv4", IpV4Validator::compile);
        map.insert("ipv6", IpV6Validator::compile);
        map.insert("iri", IRIValidator::compile);
        map.insert("iri-reference", IRIReferenceValidator::compile);
        map.insert("json-pointer", JSONPointerValidator::compile);
        map.insert("regex", RegexValidator::compile);
        map.insert("relative-json-pointer", RelativeJSONPointerValidator::compile);
        map.insert("time", TimeValidator::compile);
        map.insert("uri", URIValidator::compile);
        map.insert("uri-reference", URIReferenceValidator::compile);
        map.insert("uri-template", URITemplateValidator::compile);
        map
    };
}
