use myhomelab_agent_prelude::collector::Collector;
use myhomelab_agent_prelude::reader::BuildContext;
use myhomelab_agent_prelude::reader::ReaderBuilder;

#[derive(Debug, Default, serde::Deserialize)]
pub struct ConfigWrapper<T> {
    pub enabled: bool,
    #[serde(default, flatten)]
    pub inner: T,
}

impl<T: ReaderBuilder> ConfigWrapper<T> {
    async fn build<C: Collector>(
        &self,
        ctx: &BuildContext<C>,
    ) -> anyhow::Result<Option<T::Output>> {
        if self.enabled {
            self.inner.build(ctx).await.map(Some)
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Default, serde::Deserialize)]
pub struct ManagerConfig {
    system: ConfigWrapper<myhomelab_agent_reader_system::SystemReaderConfig>,
    xiaomi_miflora: ConfigWrapper<myhomelab_agent_reader_xiaomi_miflora::MifloraReaderConfig>,
}

impl myhomelab_agent_prelude::reader::ReaderBuilder for ManagerConfig {
    type Output = Manager;

    async fn build<C: Collector>(&self, ctx: &BuildContext<C>) -> anyhow::Result<Self::Output> {
        Ok(Manager {
            system: self.system.build(ctx).await?,
            xiaomi_miflora: self.xiaomi_miflora.build(ctx).await?,
        })
    }
}

#[derive(Debug)]
pub struct Manager {
    system: Option<myhomelab_agent_reader_system::SystemReader>,
    xiaomi_miflora: Option<myhomelab_agent_reader_xiaomi_miflora::MifloraReader>,
}

impl myhomelab_agent_prelude::reader::Reader for Manager {
    async fn wait(self) -> anyhow::Result<()>
    where
        Self: Sized,
    {
        let mut errors = Vec::default();
        if let Some(system) = self.system {
            if let Err(err) = system.wait().await {
                errors.push(err);
            }
        }
        if let Some(xiaomi_miflora) = self.xiaomi_miflora {
            if let Err(err) = xiaomi_miflora.wait().await {
                errors.push(err);
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors
                .into_iter()
                .fold(anyhow::anyhow!("some reader failed"), |prev, err| {
                    prev.context(err)
                }))
        }
    }
}
