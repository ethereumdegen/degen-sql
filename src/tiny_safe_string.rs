use serde::{Deserialize, Serialize, Serializer, Deserializer};
use std::fmt;
use std::str::FromStr;
use serde::de::{self, Visitor};
use std::ops::Deref;

#[cfg_attr(feature = "utoipa-schema", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TinySafeString(String);

impl TinySafeString {
    /// Creates a new TinySafeString if the input string only contains alphanumeric characters
    pub fn new(s: &str) -> Result<Self, String> {
        if s.chars().all(|c| c.is_alphanumeric() || c == '_') {
            Ok(TinySafeString(s.to_string()))
        } else {
            Err(format!("Invalid string: '{}'. Only alphanumeric characters and underscores are allowed.", s))
        }
    }

    /// Get the inner string value
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validates if a string meets the safety requirements
    pub fn is_valid(s: &str) -> bool {
        s.chars().all(|c| c.is_alphanumeric() || c == '_')
    }

    /// Convert to SQL-safe column name
    pub fn to_sql_string(&self) -> &str {
        &self.0
    }
}

impl Deref for TinySafeString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for TinySafeString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl FromStr for TinySafeString {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        TinySafeString::new(s)
    }
}

/*
impl<'a> TryFrom<&'a str> for TinySafeString {
    type Error = String;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        TinySafeString::new(s)
    }
}*/

impl TryFrom<String> for TinySafeString {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        TinySafeString::new(&s)
    }
}

impl From<TinySafeString> for String {
    fn from(value: TinySafeString) -> Self {
        value.0
    }
}

// Custom serialization for TinySafeString
impl Serialize for TinySafeString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

// Custom deserialization for TinySafeString
impl<'de> Deserialize<'de> for TinySafeString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TinySafeStringVisitor;
        
        impl<'de> Visitor<'de> for TinySafeStringVisitor {
            type Value = TinySafeString;
            
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string containing only alphanumeric characters and underscores")
            }
            
            fn visit_str<E>(self, value: &str) -> Result<TinySafeString, E>
            where
                E: de::Error,
            {
                TinySafeString::new(value).map_err(E::custom)
            }
        }
        
        deserializer.deserialize_str(TinySafeStringVisitor)
    }
}

// Implement Display for TinySafeString
impl fmt::Display for TinySafeString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Implement From<&str> for easy conversion in code, but with a panic for invalid strings
// Use TryFrom for safe conversion instead
impl From<&str> for TinySafeString {
    fn from(s: &str) -> Self {
        Self::new(s).unwrap_or_else(|e| panic!("{}", e))
    }
}

#[cfg(test)]
mod tests {
    use crate::pagination::PaginationData;
use crate::pagination::ColumnSortDir;
use super::*;
    use serde_json::json;

    #[test]
    fn test_valid_strings() {
        assert!(TryInto::<TinySafeString>::try_into("hello").is_ok() );
        assert!(TinySafeString::new("hello123").is_ok());
        assert!(TinySafeString::new("HELLO").is_ok());
        assert!(TinySafeString::new("hello_world").is_ok());
        assert!(TinySafeString::new("_underscore").is_ok());
    }

    #[test]
    fn test_invalid_strings() {
        assert!(TinySafeString::new("hello world").is_err());
        assert!(TinySafeString::new("hello-world").is_err());
        assert!(TinySafeString::new("hello;drop table").is_err());
        assert!(TinySafeString::new("SELECT * FROM").is_err());
        assert!(TinySafeString::new("hello'world").is_err());
    }

    #[test]
    fn test_serialization() {
        let safe_str = TinySafeString::new("hello123").unwrap();
        let serialized = serde_json::to_string(&safe_str).unwrap();
        assert_eq!(serialized, "\"hello123\"");
    }

    #[test]
    fn test_deserialization() {
        let json_str = "\"hello123\"";
        let safe_str: TinySafeString = serde_json::from_str(json_str).unwrap();
        assert_eq!(safe_str.as_str(), "hello123");
    }

    #[test]
    fn test_invalid_deserialization() {
        let json_str = "\"hello world\"";
        let result: Result<TinySafeString, _> = serde_json::from_str(json_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_pagination_data() {
        let json_data = json!({
            "page": 2,
            "page_size": 20,
            "sort_by": "created_at",
            "sort_dir": "asc"
        });

        let pagination_data: PaginationData = serde_json::from_value(json_data).unwrap();
        assert_eq!(pagination_data.page, Some(2));
        assert_eq!(pagination_data.page_size, Some(20));
        assert_eq!(pagination_data.sort_by.as_ref().unwrap().as_str(), "created_at");
        assert!(matches!(pagination_data.sort_dir.as_ref().unwrap(), ColumnSortDir::Asc));
    }
}