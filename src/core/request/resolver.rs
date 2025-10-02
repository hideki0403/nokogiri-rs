use anyhow::Result;
use hickory_resolver::{TokioResolver, config::LookupIpStrategy, lookup_ip::LookupIpIntoIter};
use once_cell::sync::OnceCell;
use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use std::{net::SocketAddr, sync::Arc};

#[derive(Debug, Default, Clone)]
pub struct CustomDnsResolver {
    state: Arc<OnceCell<TokioResolver>>,
}

struct SocketAddrs {
    iter: LookupIpIntoIter,
}

impl Iterator for SocketAddrs {
    type Item = SocketAddr;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|ip_addr| SocketAddr::new(ip_addr, 0))
    }
}

impl Resolve for CustomDnsResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let resolver = self.clone();
        Box::pin(async move {
            let resolver = resolver.state.get_or_try_init(create_resolver)?;
            let lookup = resolver.lookup_ip(name.as_str()).await?;
            let addrs: Addrs = Box::new(SocketAddrs { iter: lookup.into_iter() });

            Ok(addrs)
        })
    }
}

fn create_resolver() -> Result<TokioResolver> {
    let mut builder = TokioResolver::builder_tokio()?;
    builder.options_mut().ip_strategy = LookupIpStrategy::Ipv4AndIpv6;
    Ok(builder.build())
}
