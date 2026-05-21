use crate::config::CONFIG;
use anyhow::{Result, anyhow};
use axum::http::Extensions;
use reqwest::{Request, Response};
use reqwest_middleware::{Error as ReqwestMiddlewareError, Middleware, Next};
use std::{
    error::Error,
    fmt,
    net::{IpAddr, Ipv4Addr},
};
use url::{Host, Url};

use super::resolver;

#[derive(Debug, Default)]
pub struct BlockNonGlobalIpMiddleware;

#[derive(Debug)]
pub struct BlockNonGlobalIpError {
    pub url: String,
    pub blocked_ip: IpAddr,
    pub resolved_from: Option<String>,
}

impl BlockNonGlobalIpError {
    fn new(url: &Url, blocked_ip: IpAddr, resolved_from: Option<&str>) -> Self {
        Self {
            url: url.to_string(),
            blocked_ip,
            resolved_from: resolved_from.map(str::to_string),
        }
    }
}

impl fmt::Display for BlockNonGlobalIpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(resolved_from) = &self.resolved_from {
            write!(
                f,
                "Blocked by BlockNonGlobalIpMiddleware: non-global IP {} (resolved from {}) for URL {}",
                self.blocked_ip, resolved_from, self.url
            )
        } else {
            write!(
                f,
                "Blocked by BlockNonGlobalIpMiddleware: non-global IP {} for URL {}",
                self.blocked_ip, self.url
            )
        }
    }
}

impl Error for BlockNonGlobalIpError {}

#[async_trait::async_trait]
impl Middleware for BlockNonGlobalIpMiddleware {
    async fn handle(&self, req: Request, extensions: &mut Extensions, next: Next<'_>) -> reqwest_middleware::Result<Response> {
        if let Err(err) = enforce_non_global_ip_block(req.url()).await {
            return Err(ReqwestMiddlewareError::Middleware(err));
        }

        next.run(req, extensions).await
    }
}

fn is_non_global_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            ip.is_private() ||
                is_cgnat_ipv4(ip) ||
                ip.is_loopback() ||
                ip.is_link_local() ||
                ip.is_broadcast() ||
                ip.is_unspecified() ||
                ip.is_multicast()
        }
        IpAddr::V6(ip) => ip.is_loopback() || ip.is_unique_local() || ip.is_unicast_link_local() || ip.is_unspecified() || ip.is_multicast(),
    }
}

// Block Carrier-Grade NAT IPv4 addresses
// for Tailscale, Cloudflare Warp, ...etc
fn is_cgnat_ipv4(ip: Ipv4Addr) -> bool {
    let [a, b, _, _] = ip.octets();
    a == 100 && (64..=127).contains(&b)
}

async fn enforce_non_global_ip_block(url: &Url) -> Result<()> {
    if !CONFIG.security.block_non_global_ips {
        return Ok(());
    }

    let host = url.host().ok_or_else(|| anyhow!("URL has no host: {}", url))?;

    match host {
        Host::Ipv4(ip) => {
            let ip = IpAddr::V4(ip);
            if is_non_global_ip(ip) {
                return Err(BlockNonGlobalIpError::new(url, ip, None).into());
            }
        }
        Host::Ipv6(ip) => {
            let ip = IpAddr::V6(ip);
            if is_non_global_ip(ip) {
                return Err(BlockNonGlobalIpError::new(url, ip, None).into());
            }
        }
        Host::Domain(domain) => {
            let ips = resolver::lookup_ips(domain).await?;
            tracing::debug!("Resolved domain '{}' to IPs: {:?}", domain, ips);
            if let Some(ip) = ips.iter().copied().find(|ip| is_non_global_ip(*ip)) {
                return Err(BlockNonGlobalIpError::new(url, ip, Some(domain)).into());
            }
        }
    }

    Ok(())
}
