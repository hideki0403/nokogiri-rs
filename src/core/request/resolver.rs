use anyhow::Result;
use hickory_resolver::{TokioResolver, config::LookupIpStrategy};
use once_cell::sync::OnceCell;
use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use std::net::{IpAddr, SocketAddr};

#[derive(Debug, Default, Clone)]
pub struct CustomDnsResolver;

static RESOLVER: OnceCell<TokioResolver> = OnceCell::new();

impl Resolve for CustomDnsResolver {
    fn resolve(&self, name: Name) -> Resolving {
        Box::pin(async move {
            let ips = lookup_ips(name.as_str()).await?;
            let addrs: Addrs = Box::new(ips.into_iter().map(|ip_addr| SocketAddr::new(ip_addr, 0)));
            Ok(addrs)
        })
    }
}

pub async fn lookup_ips(host: &str) -> Result<Vec<IpAddr>> {
    let resolver = get_resolver()?;
    let lookup = resolver.lookup_ip(host).await?;
    Ok(lookup.iter().collect())
}

fn get_resolver() -> Result<&'static TokioResolver> {
    RESOLVER.get_or_try_init(create_resolver)
}

fn create_resolver() -> Result<TokioResolver> {
    let mut builder = TokioResolver::builder_tokio()?;
    builder.options_mut().ip_strategy = LookupIpStrategy::Ipv4AndIpv6;
    Ok(builder.build()?)
}
