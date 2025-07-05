use std::collections::HashMap;

use myhomelab_metric::query::{QueryExecutor, Request, RequestKind, Response, TimeRange};

mod scalar;
mod shared;
mod timeseries;

impl QueryExecutor for crate::Sqlite {
    async fn execute(
        &self,
        requests: HashMap<Box<str>, Request>,
        timerange: TimeRange,
    ) -> anyhow::Result<HashMap<Box<str>, Response>> {
        let mut res = HashMap::with_capacity(requests.len());
        for (name, req) in requests {
            match req.kind {
                RequestKind::Scalar => {
                    match scalar::fetch(self.as_ref(), &req.query, &timerange).await {
                        Ok(response) => {
                            res.insert(name, Response::Scalar(response));
                        }
                        Err(err) => eprintln!("something went wrong: {err:?}"),
                    }
                }
                RequestKind::Timeseries { period } => {
                    match timeseries::fetch(self.as_ref(), &req.query, &timerange, period).await {
                        Ok(response) => {
                            res.insert(name, Response::Timeseries(response));
                        }
                        Err(err) => eprintln!("something went wrong: {err:?}"),
                    }
                }
            }
        }
        Ok(res)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::collections::HashMap;

    use myhomelab_metric::entity::{MetricHeader, MetricTags};
    use myhomelab_metric::intake::Intake;
    use myhomelab_metric::metrics;
    use myhomelab_metric::query::{Query, QueryExecutor, Request, Response, TimeRange};

    pub(crate) async fn prepare_pool() -> anyhow::Result<crate::Sqlite> {
        let config = crate::SqliteConfig::default();
        let sqlite = config.build().await?;
        sqlite.prepare().await?;

        sqlite
            .ingest([metrics!("system.cpu", gauge, "host" => "raspberry", "location" => "FR", [(1, 80.0), (2, 90.0), (3, 50.0), (4, 20.0)]),
                metrics!("system.cpu", gauge, "host" => "raspberry", "location" => "ES", [(1, 10.0), (2, 30.0), (3, 40.0), (4, 30.0)]),
                metrics!("system.cpu", gauge, "host" => "macbook", "location" => "FR", [(1, 1.0), (2, 2.0), (3, 3.0), (4, 2.0)]),
                metrics!("system.reboot", counter, "host" => "macbook", "location" => "FR", [(2, 1), (5, 1)])].concat())
            .await?;

        Ok(sqlite)
    }

    #[tokio::test]
    async fn should_fetch_multiple_requests() {
        let sqlite = crate::query::tests::prepare_pool().await.unwrap();

        let mut reqs = HashMap::with_capacity(1);
        reqs.insert(
            Box::from("default"),
            Request::scalar(Query::sum(MetricHeader::new(
                "system.reboot",
                Default::default(),
            ))),
        );
        reqs.insert(
            Box::from("reboot-macbook"),
            Request::scalar(Query::sum(MetricHeader::new(
                "system.reboot",
                MetricTags::default().with_tag("host", "macbook"),
            ))),
        );
        reqs.insert(
            Box::from("reboot-raspberry"),
            Request::scalar(Query::sum(MetricHeader::new(
                "system.reboot",
                MetricTags::default().with_tag("host", "raspberry"),
            ))),
        );
        reqs.insert(
            Box::from("cpu-global"),
            Request::timeseries(
                3,
                Query::max(MetricHeader::new("system.cpu", MetricTags::default())),
            ),
        );
        reqs.insert(
            Box::from("cpu-raspberry"),
            Request::timeseries(
                3,
                Query::max(MetricHeader::new(
                    "system.cpu",
                    MetricTags::default().with_tag("host", "raspberry"),
                )),
            ),
        );

        let res = sqlite.execute(reqs, TimeRange::from(0)).await.unwrap();

        assert_eq!(res.len(), 5);
        assert!(matches!(res["default"], Response::Scalar(_)));
        assert!(matches!(res["reboot-macbook"], Response::Scalar(_)));
        assert!(matches!(res["reboot-raspberry"], Response::Scalar(_)));
        assert!(matches!(res["cpu-global"], Response::Timeseries(_)));
        assert!(matches!(res["cpu-raspberry"], Response::Timeseries(_)));
    }

    #[tokio::test]
    async fn scalar_should_return_none_for_missing_metric() {
        let sqlite = crate::query::tests::prepare_pool().await.unwrap();

        let mut reqs = HashMap::with_capacity(1);
        reqs.insert(
            Box::from("default"),
            Request::scalar(Query::sum(MetricHeader::new(
                "nonexistent.metric",
                Default::default(),
            ))),
        );
        let res = sqlite.execute(reqs, TimeRange::from(0)).await.unwrap();

        assert_eq!(res.len(), 1);
        if let Response::Scalar(val) = &res["default"] {
            assert!(val.is_empty(), "Expected none for missing metric");
        } else {
            panic!("Expected Scalar response");
        }
    }

    #[tokio::test]
    async fn scalar_should_filter_by_multiple_tags() {
        let sqlite = crate::query::tests::prepare_pool().await.unwrap();

        let tags = MetricTags::default()
            .with_tag("host", "raspberry")
            .with_tag("location", "FR");

        let mut reqs = HashMap::with_capacity(1);
        reqs.insert(
            Box::from("default"),
            Request::scalar(Query::sum(MetricHeader::new("system.cpu", tags))),
        );
        let res = sqlite.execute(reqs, TimeRange::from(0)).await.unwrap();

        assert_eq!(res.len(), 1);
        if let Response::Scalar(val) = &res["default"] {
            assert!(!val.is_empty(), "Expected Some value for filtered tags");
            // Optionally check the value if you know what it should be
        } else {
            panic!("Expected Scalar response");
        }
    }

    #[tokio::test]
    async fn scalar_should_support_different_aggregations() {
        let sqlite = crate::query::tests::prepare_pool().await.unwrap();

        let mut reqs = HashMap::with_capacity(3);
        reqs.insert(
            Box::from("sum_req"),
            Request::scalar(Query::sum(MetricHeader::new(
                "system.cpu",
                Default::default(),
            ))),
        );
        reqs.insert(
            Box::from("max_req"),
            Request::scalar(Query::max(MetricHeader::new(
                "system.cpu",
                Default::default(),
            ))),
        );
        reqs.insert(
            Box::from("min_req"),
            Request::scalar(Query::min(MetricHeader::new(
                "system.cpu",
                Default::default(),
            ))),
        );

        let res = sqlite.execute(reqs, TimeRange::from(0)).await.unwrap();

        assert_eq!(res.len(), 3);
        assert!(matches!(res["sum_req"], Response::Scalar(_)));
        assert!(matches!(res["max_req"], Response::Scalar(_)));
        assert!(matches!(res["min_req"], Response::Scalar(_)));
    }
}
