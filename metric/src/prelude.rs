use crate::entity::value::MetricValue;

pub trait MetricFacade: Send {
    fn name(&self) -> &str;
    fn tags(&self) -> &impl serde::Serialize;
    fn timestamp(&self) -> u64;
    fn value(&self) -> MetricValue;
}
