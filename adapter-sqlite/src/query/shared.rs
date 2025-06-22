use maitryk::metric::tag::TagValue;
use maitryk::query::TimeRange;

pub(super) struct Wrapper<V>(pub(super) V);

impl<V> Wrapper<V> {
    pub(super) fn from_many(list: Vec<Self>) -> Vec<V> {
        list.into_iter().map(|Wrapper(inner)| inner).collect()
    }
}

pub(super) fn build_tags_attribute<'a>(
    qb: &mut sqlx::QueryBuilder<'a, sqlx::Sqlite>,
    tags: impl Iterator<Item = &'a Box<str>>,
) {
    qb.push(", json_object(");
    let mut sep = qb.separated(",");
    for name in tags {
        let path = format!("'$.{name}'");
        sep.push_bind(name)
            .push(",")
            .push("json_extract(tags,")
            .push_bind(path)
            .push(")");
    }
    sep.push_unseparated(") as tags");
}

pub(super) fn build_value_attribute(
    qb: &mut sqlx::QueryBuilder<'_, sqlx::Sqlite>,
    aggr: &maitryk::query::Aggregator,
) {
    match aggr {
        maitryk::query::Aggregator::Average => qb.push(", avg(value) as value"),
        maitryk::query::Aggregator::Max => qb.push(", max(value) as value"),
        maitryk::query::Aggregator::Min => qb.push(", min(value) as value"),
        maitryk::query::Aggregator::Sum => qb.push(", sum(value) as value"),
    };
}

pub(super) fn build_timerange_filter(
    qb: &mut sqlx::QueryBuilder<'_, sqlx::Sqlite>,
    timerange: &TimeRange,
) {
    qb.push(" and timestamp >= ").push_bind(timerange.start);
    if let Some(end) = timerange.end {
        qb.push(" and timestamp < ").push_bind(end);
    }
}

pub(super) fn build_tags_filter<'a>(
    qb: &mut sqlx::QueryBuilder<'a, sqlx::Sqlite>,
    tags: impl Iterator<Item = (&'a Box<str>, &'a TagValue)>,
) {
    for (name, value) in tags {
        let path = format!("$.{name}");
        qb.push(" and json_extract(tags,")
            .push_bind(path)
            .push(") = ");
        match value {
            TagValue::Text(text_value) => {
                qb.push_bind(text_value);
            }
            TagValue::Integer(int_value) => {
                qb.push_bind(int_value);
            }
            _ => {}
        }
    }
}
