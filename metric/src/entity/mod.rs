use std::collections::BTreeMap;

pub mod tag;
pub mod value;

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub struct MetricHeader {
    pub name: Box<str>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub tags: BTreeMap<Box<str>, tag::TagValue>,
}

impl MetricHeader {
    pub fn new(name: impl Into<Box<str>>) -> Self {
        Self {
            name: name.into(),
            tags: Default::default(),
        }
    }

    pub fn maybe_set_tag<V: Into<tag::TagValue>>(
        &mut self,
        name: impl Into<Box<str>>,
        value: Option<V>,
    ) {
        if let Some(value) = value {
            self.tags.insert(name.into(), value.into());
        }
    }

    pub fn set_tag(&mut self, name: impl Into<Box<str>>, value: impl Into<tag::TagValue>) {
        self.tags.insert(name.into(), value.into());
    }

    pub fn with_tag(mut self, name: impl Into<Box<str>>, value: impl Into<tag::TagValue>) -> Self {
        self.set_tag(name, value);
        self
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
            let mut header = myhomelab_metric::entity::MetricHeader::new($name);
            $(
                header = header.with_tag($tag_key, $tag_val);
            )+

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
