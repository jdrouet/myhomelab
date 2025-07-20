use std::borrow::Cow;
use std::collections::BTreeMap;

use tag::TagValue;
use value::MetricValue;

pub mod tag;
pub mod value;

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct MetricTags(BTreeMap<Cow<'static, str>, tag::TagValue>);

impl MetricTags {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get(&self, name: &str) -> Option<&TagValue> {
        self.0.get(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &TagValue)> {
        self.0.iter().map(|(key, value)| (key.as_ref(), value))
    }

    pub fn set_tag<N, V>(&mut self, name: N, value: V)
    where
        N: Into<Cow<'static, str>>,
        V: Into<tag::TagValue>,
    {
        self.0.insert(name.into(), value.into());
    }

    pub fn maybe_set_tag<N, V>(&mut self, name: N, value: Option<V>)
    where
        N: Into<Cow<'static, str>>,
        V: Into<tag::TagValue>,
    {
        if let Some(value) = value {
            self.0.insert(name.into(), value.into());
        }
    }

    pub fn with_tag<N, V>(mut self, name: N, value: V) -> Self
    where
        N: Into<Cow<'static, str>>,
        V: Into<tag::TagValue>,
    {
        self.set_tag(name, value);
        self
    }

    pub fn maybe_with_tag<N, V>(self, name: N, value: Option<V>) -> Self
    where
        N: Into<Cow<'static, str>>,
        V: Into<tag::TagValue>,
    {
        if let Some(value) = value {
            self.with_tag(name, value)
        } else {
            self
        }
    }
}

impl std::fmt::Display for MetricTags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, (name, value)) in self.0.iter().enumerate() {
            if index > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{name}={value}")?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MetricHeader<'h> {
    pub name: &'h str,
    pub tags: &'h MetricTags,
}

impl<'h> MetricHeader<'h> {
    pub fn new(name: &'h str, tags: &'h MetricTags) -> Self {
        Self { name, tags }
    }

    pub fn tag(&self, name: &str) -> Option<&tag::TagValue> {
        self.tags.0.get(name)
    }

    pub fn iter_tags(&self) -> impl Iterator<Item = (&str, &tag::TagValue)> {
        self.tags.0.iter().map(|(key, value)| (key.as_ref(), value))
    }
}

impl<'h> std::fmt::Display for MetricHeader<'h> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{{{}}}", self.name, self.tags)
    }
}

#[derive(Clone, Debug)]
pub struct Metric<'a, V = MetricValue> {
    pub name: Cow<'a, str>,
    pub tags: Cow<'a, MetricTags>,
    pub timestamp: u64,
    pub value: V,
}

impl<'a> Metric<'a> {
    pub fn as_borrowed(&'a self) -> Metric<'a> {
        Metric {
            name: Cow::Borrowed(&self.name),
            tags: Cow::Borrowed(&self.tags),
            timestamp: self.timestamp,
            value: self.value,
        }
    }
}

impl std::fmt::Display for Metric<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            MetricHeader::new(&self.name, &self.tags),
            self.timestamp,
            self.value
        )
    }
}

#[cfg(feature = "macros")]
#[macro_export]
macro_rules! metrics {
    (
        $name:expr,
        $val_ty:ident,
        $( $tag_key:literal => $tag_val:expr ),+,
        [ $( ($timestamp:expr, $value:expr) ),+ $(,)? ]
    ) => {{
        {
            let mut tags = myhomelab_metric::entity::MetricTags::default();
            $(
                tags.set_tag($tag_key, $tag_val);
            )+
            vec![
                $(
                    myhomelab_metric::entity::Metric {
                        name: $name.into(),
                        tags: std::borrow::Cow::Owned(tags.clone()),
                        timestamp: $timestamp,
                        value: myhomelab_metric::entity::value::MetricValue::$val_ty($value),
                    }
                ),+
            ]
        }
    }};
}

#[cfg(test)]
mod tests {
    use crate::entity::{MetricHeader, MetricTags};

    #[test]
    fn should_format_metric_headers() {
        assert_eq!(
            MetricHeader::new("foo", &Default::default()).to_string(),
            "foo{}"
        );
        assert_eq!(
            MetricHeader::new("foo", &MetricTags::default().with_tag("hello", "world")).to_string(),
            "foo{hello=\"world\"}"
        );
        assert_eq!(
            MetricHeader::new(
                "foo",
                &MetricTags::default()
                    .with_tag("hello", 42i64)
                    .with_tag("world", "bar")
            )
            .to_string(),
            "foo{hello=42, world=\"bar\"}"
        );
    }
}
