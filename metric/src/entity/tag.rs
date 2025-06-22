#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum TagValueArray {
    Text(Box<str>),
    Integer(i64),
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum TagValue {
    Text(Box<str>),
    Integer(i64),
    Array(Box<[TagValueArray]>),
}

impl From<&str> for TagValue {
    fn from(value: &str) -> Self {
        Self::Text(value.into())
    }
}

impl From<String> for TagValue {
    fn from(value: String) -> Self {
        Self::Text(value.into())
    }
}

impl From<i64> for TagValue {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::entity::tag::TagValue;

    #[test]
    fn should_compile() {
        // text
        assert!(matches!(TagValue::from("foo"), TagValue::Text(_)));
        assert!(matches!(
            TagValue::from("foo".to_string()),
            TagValue::Text(_)
        ));
        // number
        assert!(matches!(TagValue::from(42i64), TagValue::Integer(_)));
    }
}
