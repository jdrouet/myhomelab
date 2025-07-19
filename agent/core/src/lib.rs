use myhomelab_agent_prelude::mpsc::Sender;
use myhomelab_agent_prelude::reader::BuildContext;
use myhomelab_agent_prelude::reader::ReaderBuilder;

#[derive(Debug, Default, serde::Deserialize)]
pub struct ConfigWrapper<T> {
    pub enabled: bool,
    #[serde(default, flatten)]
    pub inner: T,
}

#[derive(Debug, Default, serde::Deserialize)]
pub struct ManagerConfig {
    system: ConfigWrapper<myhomelab_agent_reader_system::SystemReaderConfig>,
}

impl ManagerConfig {
    async fn build_system<S: Sender>(
        &self,
        ctx: &BuildContext<S>,
    ) -> anyhow::Result<Option<myhomelab_agent_reader_system::SystemReader>> {
        if !self.system.enabled {
            return Ok(None);
        }
        self.system.inner.build(ctx).await.map(Some)
    }
}

impl myhomelab_agent_prelude::reader::ReaderBuilder for ManagerConfig {
    type Output = Manager;

    async fn build<S: myhomelab_agent_prelude::mpsc::Sender>(
        &self,
        ctx: &BuildContext<S>,
    ) -> anyhow::Result<Self::Output> {
        Ok(Manager {
            system: self.build_system(ctx).await?,
        })
    }
}

#[derive(Debug)]
pub struct Manager {
    system: Option<myhomelab_agent_reader_system::SystemReader>,
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
