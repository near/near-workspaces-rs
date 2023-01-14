use std::future::{Future, IntoFuture};
use std::marker::PhantomData;
use std::path::PathBuf;

use crate::network::Sandbox;
use crate::{Network, Worker};

pub(crate) type BoxFuture<'a, T> = std::pin::Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// This trait provides a way to construct Networks out of a single builder. Currently
/// not planned to offer this trait outside, since the custom networks can just construct
/// themselves however they want utilizing `Worker::new` like so:
/// ```ignore
/// Worker::new(CustomNetwork {
///   ... // fields
/// })
/// ```
#[async_trait::async_trait]
pub(crate) trait FromNetworkBuilder: Sized {
    async fn from_builder<'a>(build: NetworkBuilder<'a, Self>) -> crate::result::Result<Self>;
}

/// Builder for Networks. Only usable with workspaces provided Networks.
// Note, this is currently the aggregated state for all network types you can have since
// I didn't want to add additional reading complexity with another trait that associates the
// Network state.
pub struct NetworkBuilder<'a, T> {
    pub(crate) name: &'a str,
    pub(crate) rpc_addr: Option<String>,
    pub(crate) home_dir: Option<PathBuf>,
    _network: PhantomData<T>,
}

impl<'a, T> IntoFuture for NetworkBuilder<'a, T>
where
    T: FromNetworkBuilder + Network + Send + 'a,
{
    type Output = crate::result::Result<Worker<T>>;
    type IntoFuture = BoxFuture<'a, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        let fut = async {
            let network = FromNetworkBuilder::from_builder(self).await?;
            Ok(Worker::new(network))
        };
        Box::pin(fut)
    }
}

impl<'a, T> NetworkBuilder<'a, T> {
    pub(crate) fn new(name: &'a str) -> Self {
        Self {
            name,
            rpc_addr: None,
            home_dir: None,
            _network: PhantomData,
        }
    }

    /// Sets the RPC addr for this network. Useful for setting the Url to a different RPC
    /// node than the default one provided by near.org. This enables certain features that
    /// the default node doesn't provide such as getting beyond the data cap when downloading
    /// state from the network.
    /// Example:
    /// ```
    /// async fn connect_custom() -> anyhow::Result<()> {
    ///     let worker = workspaces::testnet()
    ///         .rpc_addr("https://rpc.testnet.near.org")
    ///         .await?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// Note that, for sandbox, we also are required to specify `home_dir`
    pub fn rpc_addr(mut self, addr: &str) -> Self {
        self.rpc_addr = Some(addr.into());
        self
    }
}

// So far, only Sandbox makes use of home_dir.
impl NetworkBuilder<'_, Sandbox> {
    /// Specify at which location the home_dir of the manually spawned sandbox node is at.
    /// We are expected to init our own sandbox before running this builder. To learn more
    /// about initalizing and  starting our own sandbox, go to [near-sandbox](https://github.com/near/sandbox).
    /// Also required to set the home directory where all the chain data lives. This is
    /// the `my_home_folder` we passed into `near-sandbox --home {my_home_folder} init`.
    pub fn home_dir(mut self, home_dir: impl AsRef<std::path::Path>) -> Self {
        self.home_dir = Some(home_dir.as_ref().into());
        self
    }
}
