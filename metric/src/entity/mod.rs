use std::{borrow::Cow, collections::BTreeMap};

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

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub struct MetricHeader {
    pub name: Cow<'static, str>,
    #[serde(default, skip_serializing_if = "MetricTags::is_empty")]
    pub tags: MetricTags,
}

impl MetricHeader {
    pub fn new(name: impl Into<Cow<'static, str>>, tags: MetricTags) -> Self {
        Self {
            name: name.into(),
            tags,
        }
    }

    pub fn tag(&self, name: &str) -> Option<&tag::TagValue> {
        self.tags.0.get(name)
    }

    pub fn iter_tags(&self) -> impl Iterator<Item = (&str, &tag::TagValue)> {
        self.tags.0.iter().map(|(key, value)| (key.as_ref(), value))
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Metric {
    #[serde(flatten)]
    pub header: MetricHeader,
    pub timestamp: u64,
    pub value: value::MetricValue,
}

impl Metric {
    pub fn as_counter(&self) -> Option<MetricRef<'_, value::CounterValue>> {
        match self.value {
            value::MetricValue::Counter(ref value) => Some(MetricRef {
                header: &self.header,
                timestamp: self.timestamp,
                value,
            }),
            _ => None,
        }
    }

    pub fn as_gauge(&self) -> Option<MetricRef<'_, value::GaugeValue>> {
        match self.value {
            value::MetricValue::Gauge(ref value) => Some(MetricRef {
                header: &self.header,
                timestamp: self.timestamp,
                value,
            }),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MetricRef<'a, V> {
    pub header: &'a MetricHeader,
    pub timestamp: u64,
    pub value: &'a V,
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
            let mut header = myhomelab_metric::entity::MetricHeader::new($name, tags);

            vec![
                $(
                    myhomelab_metric::entity::Metric {
                        header: header.clone(),
                        timestamp: $timestamp,
                        value: myhomelab_metric::entity::value::MetricValue::$val_ty($value),
                    }
                ),+
            ]
        }
    }};
}
