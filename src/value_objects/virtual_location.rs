//! Virtual location value objects including URLs and IP addresses

use cim_domain::{DomainError, DomainResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::net::IpAddr;
use std::str::FromStr;
use url::Url;

/// Enhanced virtual location with comprehensive online presence support
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VirtualLocation {
    /// Type of virtual location
    pub location_type: VirtualLocationType,

    /// Primary identifier for the location
    pub primary_identifier: String,

    /// URLs associated with this location
    pub urls: Vec<VirtualUrl>,

    /// IP addresses associated with this location
    pub ip_addresses: Vec<IpAddress>,

    /// Network information
    pub network_info: Option<NetworkInfo>,

    /// Platform-specific metadata
    pub metadata: HashMap<String, String>,
}

/// Types of virtual locations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VirtualLocationType {
    /// Website or web application
    Website,
    /// API endpoint
    ApiEndpoint,
    /// Cloud service (AWS, Azure, GCP, etc.)
    CloudService { provider: String, region: String },
    /// Container or pod
    Container {
        orchestrator: String,
        namespace: String,
    },
    /// Virtual machine
    VirtualMachine,
    /// Network device
    NetworkDevice,
    /// Online meeting room
    MeetingRoom { platform: String },
    /// Game server or virtual world
    GameServer,
    /// Blockchain address
    BlockchainAddress { chain: String },
    /// Email server
    EmailServer,
    /// Custom virtual location
    Custom(String),
}

/// URL with metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VirtualUrl {
    /// The URL itself
    pub url: String,

    /// Type of URL
    pub url_type: UrlType,

    /// Whether this URL is currently active
    pub is_active: bool,

    /// Priority (lower is higher priority)
    pub priority: u8,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Types of URLs
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UrlType {
    /// Primary website
    Primary,
    /// API endpoint
    Api,
    /// Documentation
    Documentation,
    /// Support/help URL
    Support,
    /// Status page
    Status,
    /// Webhook endpoint
    Webhook,
    /// CDN endpoint
    Cdn,
    /// Mirror/backup
    Mirror,
    /// Development/staging
    Development,
    /// Custom type
    Custom(String),
}

/// IP address with metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpAddress {
    /// The IP address
    pub address: IpAddr,

    /// Type of IP address usage
    pub ip_type: IpAddressType,

    /// Whether this IP is currently active
    pub is_active: bool,

    /// Port mappings
    pub ports: Vec<PortMapping>,

    /// Reverse DNS if available
    pub reverse_dns: Option<String>,
}

/// Types of IP address usage
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IpAddressType {
    /// Primary server IP
    Primary,
    /// Load balancer IP
    LoadBalancer,
    /// Failover/backup IP
    Failover,
    /// Internal/private IP
    Internal,
    /// VPN endpoint
    VpnEndpoint,
    /// NAT gateway
    NatGateway,
    /// Custom type
    Custom(String),
}

/// Port mapping information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortMapping {
    /// Port number
    pub port: u16,

    /// Protocol (TCP, UDP, etc.)
    pub protocol: String,

    /// Service name
    pub service: String,

    /// Whether encrypted (TLS/SSL)
    pub encrypted: bool,
}

/// Network information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkInfo {
    /// Autonomous System Number
    pub asn: Option<u32>,

    /// AS organization name
    pub as_org: Option<String>,

    /// Network CIDR blocks
    pub cidr_blocks: Vec<String>,

    /// BGP information
    pub bgp_info: Option<BgpInfo>,

    /// Latency measurements to common points
    pub latency_map: HashMap<String, f64>,
}

/// BGP routing information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BgpInfo {
    /// BGP communities
    pub communities: Vec<String>,

    /// Origin AS
    pub origin_as: u32,

    /// AS path
    pub as_path: Vec<u32>,
}

impl VirtualLocation {
    /// Create a website virtual location
    pub fn website(url: &str, name: String) -> DomainResult<Self> {
        let parsed_url = Url::parse(url)
            .map_err(|e| DomainError::ValidationError(format!("Invalid URL: {e}")))?;

        let virtual_url = VirtualUrl {
            url: url.to_string(),
            url_type: UrlType::Primary,
            is_active: true,
            priority: 0,
            metadata: HashMap::new(),
        };

        Ok(Self {
            location_type: VirtualLocationType::Website,
            primary_identifier: parsed_url.host_str().unwrap_or(&name).to_string(),
            urls: vec![virtual_url],
            ip_addresses: Vec::new(),
            network_info: None,
            metadata: HashMap::new(),
        })
    }

    /// Create an API endpoint virtual location
    pub fn api_endpoint(base_url: &str, name: String) -> DomainResult<Self> {
        let _parsed_url = Url::parse(base_url)
            .map_err(|e| DomainError::ValidationError(format!("Invalid URL: {e}")))?;

        let virtual_url = VirtualUrl {
            url: base_url.to_string(),
            url_type: UrlType::Api,
            is_active: true,
            priority: 0,
            metadata: HashMap::new(),
        };

        Ok(Self {
            location_type: VirtualLocationType::ApiEndpoint,
            primary_identifier: name,
            urls: vec![virtual_url],
            ip_addresses: Vec::new(),
            network_info: None,
            metadata: HashMap::new(),
        })
    }

    /// Create a cloud service virtual location
    pub fn cloud_service(
        provider: String,
        region: String,
        service_id: String,
    ) -> DomainResult<Self> {
        Ok(Self {
            location_type: VirtualLocationType::CloudService { provider, region },
            primary_identifier: service_id,
            urls: Vec::new(),
            ip_addresses: Vec::new(),
            network_info: None,
            metadata: HashMap::new(),
        })
    }

    /// Add a URL to this virtual location
    pub fn add_url(&mut self, url: VirtualUrl) -> DomainResult<()> {
        // Validate URL
        Url::parse(&url.url)
            .map_err(|e| DomainError::ValidationError(format!("Invalid URL: {e}")))?;

        self.urls.push(url);
        Ok(())
    }

    /// Add an IP address to this virtual location
    pub fn add_ip_address(&mut self, ip: IpAddress) -> DomainResult<()> {
        self.ip_addresses.push(ip);
        Ok(())
    }

    /// Get primary URL if available
    pub fn primary_url(&self) -> Option<&str> {
        self.urls
            .iter()
            .filter(|u| u.url_type == UrlType::Primary && u.is_active)
            .min_by_key(|u| u.priority)
            .map(|u| u.url.as_str())
    }

    /// Get all active URLs
    pub fn active_urls(&self) -> Vec<&VirtualUrl> {
        self.urls.iter().filter(|u| u.is_active).collect()
    }

    /// Get primary IP address if available
    pub fn primary_ip(&self) -> Option<&IpAddr> {
        self.ip_addresses
            .iter()
            .filter(|ip| ip.ip_type == IpAddressType::Primary && ip.is_active)
            .map(|ip| &ip.address)
            .next()
    }

    /// Check if this location has IPv6 support
    pub fn has_ipv6(&self) -> bool {
        self.ip_addresses
            .iter()
            .any(|ip| matches!(ip.address, IpAddr::V6(_)) && ip.is_active)
    }

    /// Get all active IP addresses
    pub fn active_ips(&self) -> Vec<&IpAddress> {
        self.ip_addresses.iter().filter(|ip| ip.is_active).collect()
    }
}

impl VirtualUrl {
    /// Create a new virtual URL
    pub fn new(url: String, url_type: UrlType) -> DomainResult<Self> {
        // Validate URL
        Url::parse(&url).map_err(|e| DomainError::ValidationError(format!("Invalid URL: {e}")))?;

        Ok(Self {
            url,
            url_type,
            is_active: true,
            priority: 0,
            metadata: HashMap::new(),
        })
    }

    /// Parse and get the domain from the URL
    pub fn domain(&self) -> Option<String> {
        Url::parse(&self.url)
            .ok()
            .and_then(|u| u.host_str().map(String::from))
    }

    /// Check if URL uses HTTPS
    pub fn is_secure(&self) -> bool {
        self.url.starts_with("https://") || self.url.starts_with("wss://")
    }
}

impl IpAddress {
    /// Create a new IP address entry
    pub fn new(address: &str, ip_type: IpAddressType) -> DomainResult<Self> {
        let addr = IpAddr::from_str(address)
            .map_err(|e| DomainError::ValidationError(format!("Invalid IP address: {e}")))?;

        Ok(Self {
            address: addr,
            ip_type,
            is_active: true,
            ports: Vec::new(),
            reverse_dns: None,
        })
    }

    /// Add a port mapping
    pub fn add_port(&mut self, port: PortMapping) {
        self.ports.push(port);
    }

    /// Check if this is a private IP address
    pub fn is_private(&self) -> bool {
        match self.address {
            IpAddr::V4(ipv4) => ipv4.is_private(),
            IpAddr::V6(ipv6) => ipv6.is_loopback() || ipv6.is_multicast(),
        }
    }

    /// Check if this is a loopback address
    pub fn is_loopback(&self) -> bool {
        match self.address {
            IpAddr::V4(ipv4) => ipv4.is_loopback(),
            IpAddr::V6(ipv6) => ipv6.is_loopback(),
        }
    }
}

impl PortMapping {
    /// Create a new port mapping
    pub fn new(port: u16, protocol: String, service: String) -> Self {
        Self {
            port,
            protocol,
            service,
            encrypted: false,
        }
    }

    /// Create a TLS/SSL encrypted port mapping
    pub fn new_encrypted(port: u16, protocol: String, service: String) -> Self {
        Self {
            port,
            protocol,
            service,
            encrypted: true,
        }
    }
}

impl fmt::Display for VirtualLocationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VirtualLocationType::Website => write!(f, "Website"),
            VirtualLocationType::ApiEndpoint => write!(f, "API Endpoint"),
            VirtualLocationType::CloudService { provider, region } => {
                write!(f, "Cloud Service ({}, {})", provider, region)
            }
            VirtualLocationType::Container { orchestrator, namespace } => {
                write!(f, "Container ({}/{})", orchestrator, namespace)
            }
            VirtualLocationType::VirtualMachine => write!(f, "Virtual Machine"),
            VirtualLocationType::NetworkDevice => write!(f, "Network Device"),
            VirtualLocationType::MeetingRoom { platform } => {
                write!(f, "Meeting Room ({})", platform)
            }
            VirtualLocationType::GameServer => write!(f, "Game Server"),
            VirtualLocationType::BlockchainAddress { chain } => {
                write!(f, "Blockchain Address ({})", chain)
            }
            VirtualLocationType::EmailServer => write!(f, "Email Server"),
            VirtualLocationType::Custom(desc) => write!(f, "Custom ({})", desc),
        }
    }
}

impl fmt::Display for UrlType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UrlType::Primary => write!(f, "Primary"),
            UrlType::Api => write!(f, "API"),
            UrlType::Documentation => write!(f, "Documentation"),
            UrlType::Support => write!(f, "Support"),
            UrlType::Status => write!(f, "Status"),
            UrlType::Webhook => write!(f, "Webhook"),
            UrlType::Cdn => write!(f, "CDN"),
            UrlType::Mirror => write!(f, "Mirror"),
            UrlType::Development => write!(f, "Development"),
            UrlType::Custom(desc) => write!(f, "Custom ({})", desc),
        }
    }
}

impl fmt::Display for IpAddressType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IpAddressType::Primary => write!(f, "Primary"),
            IpAddressType::LoadBalancer => write!(f, "Load Balancer"),
            IpAddressType::Failover => write!(f, "Failover"),
            IpAddressType::Internal => write!(f, "Internal"),
            IpAddressType::VpnEndpoint => write!(f, "VPN Endpoint"),
            IpAddressType::NatGateway => write!(f, "NAT Gateway"),
            IpAddressType::Custom(desc) => write!(f, "Custom ({})", desc),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_website_creation() {
        let website =
            VirtualLocation::website("https://example.com", "Example Website".to_string()).unwrap();

        assert_eq!(website.location_type, VirtualLocationType::Website);
        assert_eq!(website.primary_identifier, "example.com");
        assert_eq!(website.primary_url(), Some("https://example.com"));
    }

    #[test]
    fn test_ip_address_creation() {
        let ip = IpAddress::new("192.168.1.1", IpAddressType::Primary).unwrap();
        assert!(ip.is_private());
        assert!(!ip.is_loopback());

        let ip6 = IpAddress::new("2001:db8::1", IpAddressType::Primary).unwrap();
        assert_eq!(ip6.address, IpAddr::V6("2001:db8::1".parse().unwrap()));
    }

    #[test]
    fn test_url_validation() {
        let url = VirtualUrl::new("https://api.example.com/v1".to_string(), UrlType::Api).unwrap();

        assert!(url.is_secure());
        assert_eq!(url.domain(), Some("api.example.com".to_string()));
    }

    #[test]
    fn test_cloud_service_location() {
        let cloud = VirtualLocation::cloud_service(
            "AWS".to_string(),
            "us-east-1".to_string(),
            "i-1234567890abcdef0".to_string(),
        )
        .unwrap();

        match cloud.location_type {
            VirtualLocationType::CloudService { provider, region } => {
                assert_eq!(provider, "AWS");
                assert_eq!(region, "us-east-1");
            }
            _ => panic!("Wrong location type"),
        }
    }
}
