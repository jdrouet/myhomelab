use std::collections::HashMap;

use myhomelab_metric::query::{QueryExecutor, Request, RequestKind, Response, TimeRange};

mod scalar;
mod shared;
mod timeseries;

impl QueryExecutor for crate::Sqlite {
    async fn execute(
        &self,
        requests: Vec<Request>,
        timerange: TimeRange,
    ) -> anyhow::Result<Vec<Response>> {
        let mut res = Vec::with_capacity(requests.len());
        for req in requests {
            let mut queries = HashMap::with_capacity(req.queries.len());
            match req.kind {
                RequestKind::Scalar => {
                    for (name, query) in req.queries.iter() {
                        match scalar::fetch(self.as_ref(), query, &timerange).await {
                            Ok(response) => {
                                queries.insert(name.clone(), response);
                            }
                            Err(err) => eprintln!("something went wrong: {err:?}"),
                        }
                    }
                }
                RequestKind::Timeseries { period } => {
                    for (name, query) in req.queries.iter() {
                        match timeseries::fetch(self.as_ref(), query, &timerange, period).await {
                            Ok(response) => {
                                queries.insert(name.clone(), response);
                            }
                            Err(err) => eprintln!("something went wrong: {err:?}"),
                        }
                    }
                }
            }
            res.push(Response {
                kind: req.kind,
                queries,
            });
        }
        Ok(res)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use myhomelab_metric::{
        entity::MetricHeader,
        intake::Intake,
        metrics,
        query::{Query, QueryExecutor, Request, RequestKind, TimeRange},
    };

    pub(crate) async fn prepare_pool() -> anyhow::Result<crate::Sqlite> {
        let config = crate::SqliteConfig::default();
        let sqlite = config.build().await?;
        sqlite.prepare().await?;

        sqlite
            .ingest(vec![
                metrics!("system.cpu", gauge, "host" => "raspberry", "location" => "FR", [(1, 80.0), (2, 90.0), (3, 50.0), (4, 20.0)]),
                metrics!("system.cpu", gauge, "host" => "raspberry", "location" => "ES", [(1, 10.0), (2, 30.0), (3, 40.0), (4, 30.0)]),
                metrics!("system.cpu", gauge, "host" => "macbook", "location" => "FR", [(1, 1.0), (2, 2.0), (3, 3.0), (4, 2.0)]),
                metrics!("system.reboot", counter, "host" => "macbook", "location" => "FR", [(2, 1), (5, 1)]),
            ].concat())
            .await?;

        Ok(sqlite)
    }

    #[tokio::test]
    async fn should_fetch_multiple_requests() -> anyhow::Result<()> {
        let sqlite = crate::query::tests::prepare_pool().await?;

        let res = sqlite
            .execute(
                vec![
                    Request::scalar()
                        .with_query("reboot-all", Query::sum(MetricHeader::new("system.reboot")))
                        .with_query(
                            "reboot-macbook",
                            Query::sum(
                                MetricHeader::new("system.reboot").with_tag("host", "macbook"),
                            ),
                        )
                        .with_query(
                            "reboot-raspberry",
                            Query::sum(
                                MetricHeader::new("system.reboot").with_tag("host", "raspberry"),
                            ),
                        ),
                    Request::timeseries(3)
                        .with_query("cpu", Query::max(MetricHeader::new("system.cpu")))
                        .with_query(
                            "cpu-raspberry",
                            Query::min(
                                MetricHeader::new("system.cpu").with_tag("host", "raspberry"),
                            ),
                        ),
                ],
                TimeRange::from(0),
            )
            .await?;

        assert_eq!(res.len(), 2);
        assert_eq!(res[0].kind, RequestKind::Scalar);
        assert_eq!(res[0].queries.len(), 3);
        assert_eq!(res[1].kind, RequestKind::Timeseries { period: 3 });
        assert_eq!(res[1].queries.len(), 2);

        Ok(())
    }
}
