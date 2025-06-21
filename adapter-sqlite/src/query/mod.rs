use std::collections::HashMap;

use maitryk::query::{QueryResponse, RequestKind, Response, TimeRange};

mod scalar;

impl maitryk::query::QueryExecutor for crate::Sqlite {
    async fn execute(
        &self,
        requests: &[maitryk::query::Request],
        timerange: TimeRange,
    ) -> anyhow::Result<Vec<Response>> {
        let mut res = Vec::with_capacity(requests.len());
        for req in requests {
            match req.kind {
                RequestKind::Scalar => {
                    let mut queries = HashMap::with_capacity(req.queries.len());
                    for (name, query) in req.queries.iter() {
                        match scalar::fetch(self.as_ref(), query, &timerange).await {
                            Ok(response) => {
                                queries.insert(name.clone(), response);
                            }
                            Err(err) => eprintln!("something went wrong: {err:?}"),
                        }
                    }
                    res.push(Response {
                        kind: req.kind,
                        queries,
                    });
                }
                RequestKind::Timeseries => {
                    let mut queries = HashMap::with_capacity(req.queries.len());
                    for (name, _query) in req.queries.iter() {
                        queries.insert(name.clone(), QueryResponse::Timeseries);
                    }
                    res.push(Response {
                        kind: req.kind,
                        queries,
                    });
                }
            }
        }
        Ok(res)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use maitryk::{
        intake::Intake,
        metric::{Metric, MetricHeader, value::MetricValue},
        metrics,
    };

    pub(crate) async fn prepare_pool() -> anyhow::Result<crate::Sqlite> {
        // let config = crate::SqliteConfig {
        //     url: Some("./storage.db".into()),
        // };
        let config = crate::SqliteConfig::default();
        let sqlite = config.build().await?;
        sqlite.prepare().await?;

        sqlite
            .ingest(&[
                metrics!("system.cpu", gauge, "host" => "raspberry", "location" => "FR", [(1, 80.0), (2, 90.0), (3, 50.0), (4, 20.0)]),
                metrics!("system.cpu", gauge, "host" => "raspberry", "location" => "ES", [(1, 10.0), (2, 30.0), (3, 40.0), (4, 30.0)]),
                metrics!("system.cpu", gauge, "host" => "macbook", "location" => "FR", [(1, 1.0), (2, 2.0), (3, 3.0), (4, 2.0)]),
                metrics!("system.reboot", counter, "host" => "macbook", "location" => "FR", [(2, 1), (5, 1)]),
            ].concat())
            .await?;

        Ok(sqlite)
    }
}
