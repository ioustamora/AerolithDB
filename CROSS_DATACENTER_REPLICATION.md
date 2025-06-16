# Cross-Datacenter Replication Configuration Guide

## Overview

aerolithsDB now supports cross-datacenter replication for global consistency and multi-region data synchronization. This guide explains how to configure and deploy cross-datacenter replication.

## Configuration

### Basic Configuration

```rust
use aerolithsdb_storage::{StorageConfig, DatacenterReplicationConfig, RemoteDatacenter, ReplicationMode};

let storage_config = StorageConfig {
    // ... other storage configuration
    datacenter_replication: Some(DatacenterReplicationConfig {
        enabled: true,
        local_datacenter_id: "us-east-1".to_string(),
        remote_datacenters: vec![
            RemoteDatacenter {
                datacenter_id: "us-west-2".to_string(),
                endpoints: vec![
                    "https://aerolithsdb-usw2-1.example.com:8443".to_string(),
                    "https://aerolithsdb-usw2-2.example.com:8443".to_string(),
                ],
                region: "us-west".to_string(),
                priority: 100,
                active: true,
            },
            RemoteDatacenter {
                datacenter_id: "eu-central-1".to_string(),
                endpoints: vec![
                    "https://aerolithsdb-euc1-1.example.com:8443".to_string(),
                ],
                region: "europe".to_string(),
                priority: 90,
                active: true,
            },
        ],
        default_replication_mode: ReplicationMode::Asynchronous { max_delay_ms: 1000 },
        max_replication_lag_ms: 5000,
        retry_attempts: 3,
        batch_size: 100,
        compression_enabled: true,
    }),
};
```

### JSON Configuration Example

```json
{
  "storage": {
    "datacenter_replication": {
      "enabled": true,
      "local_datacenter_id": "us-east-1",
      "remote_datacenters": [
        {
          "datacenter_id": "us-west-2",
          "endpoints": [
            "https://aerolithsdb-usw2-1.example.com:8443",
            "https://aerolithsdb-usw2-2.example.com:8443"
          ],
          "region": "us-west",
          "priority": 100,
          "active": true
        },
        {
          "datacenter_id": "eu-central-1",
          "endpoints": [
            "https://aerolithsdb-euc1-1.example.com:8443"
          ],
          "region": "europe",
          "priority": 90,
          "active": true
        }
      ],
      "default_replication_mode": {
        "Asynchronous": {
          "max_delay_ms": 1000
        }
      },
      "max_replication_lag_ms": 5000,
      "retry_attempts": 3,
      "batch_size": 100,
      "compression_enabled": true
    }
  }
}
```

## Replication Modes

### Synchronous Replication

Provides strong consistency across all datacenters but with higher latency:

```rust
ReplicationMode::Synchronous
```

### Asynchronous Replication

Provides eventual consistency with lower latency:

```rust
ReplicationMode::Asynchronous { max_delay_ms: 1000 }
```

### Hybrid Replication

Allows per-operation configuration:

```rust
ReplicationMode::Hybrid {
    default_mode: Box::new(ReplicationMode::Asynchronous { max_delay_ms: 1000 }),
    critical_synchronous: true,
}
```

## Deployment Architecture

### Multi-Region Setup

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   US-East-1     │    │   US-West-2     │    │   EU-Central-1  │
│                 │    │                 │    │                 │
│ ┌─────────────┐ │    │ ┌─────────────┐ │    │ ┌─────────────┐ │
│ │ aerolithsDB     │ │◄──►│ │ aerolithsDB     │ │◄──►│ │ aerolithsDB     │ │
│ │ Primary     │ │    │ │ Replica     │ │    │ │ Replica     │ │
│ └─────────────┘ │    │ └─────────────┘ │    │ └─────────────┘ │
│                 │    │                 │    │                 │
│ Application     │    │ Application     │    │ Application     │
│ Servers         │    │ Servers         │    │ Servers         │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Network Requirements

- **Bandwidth**: Minimum 100 Mbps between datacenters
- **Latency**: <200ms for optimal performance
- **Security**: TLS 1.3 encryption for all inter-datacenter communication
- **Ports**: 8443 (configurable) for replication traffic

## Monitoring and Observability

### Health Checks

```rust
// Check datacenter replication health
let health_report = storage_hierarchy.check_datacenter_health().await?;
if let Some(report) = health_report {
    println!("Overall health: {}", report.overall_health);
    println!("Healthy datacenters: {}/{}", 
             report.healthy_datacenters, 
             report.total_datacenters);
}
```

### Replication Statistics

```rust
// Get replication statistics
let stats = storage_hierarchy.get_datacenter_replication_stats().await;
if let Some(stats) = stats {
    println!("Total replications: {}", stats.total_replications);
    println!("Success rate: {:.2}%", 
             (stats.successful_replications as f64 / stats.total_replications as f64) * 100.0);
    println!("Average latency: {:.2}ms", stats.average_latency_ms);
}
```

### Metrics and Alerts

Monitor these key metrics:

- **Replication Lag**: Time delay between write and replication
- **Success Rate**: Percentage of successful replications
- **Network Latency**: Round-trip time between datacenters
- **Error Rate**: Failed replication attempts
- **Data Consistency**: Cross-datacenter data verification

## Conflict Resolution

### Vector Clock Ordering

aerolithsDB uses vector clocks to maintain causal consistency across datacenters:

```rust
// Vector clocks automatically track causality
let vector_clock = VectorClock::new();
vector_clock.increment("us-east-1");  // Increment for local datacenter
```

### Resolution Strategies

1. **Last Writer Wins**: Based on timestamp (default for async)
2. **Datacenter Priority**: Higher priority datacenter wins
3. **Vector Clock**: Causal ordering resolution (default for sync)
4. **Custom**: Application-defined resolution logic

## Security Considerations

### Network Security

- All inter-datacenter traffic is encrypted with TLS 1.3
- Mutual TLS authentication between datacenters
- Certificate-based datacenter identity verification

### Data Security

- End-to-end encryption for replicated data
- Integrity verification with checksums
- Audit logging for all replication operations

## Performance Optimization

### Batching

Configure batch size for optimal throughput:

```rust
batch_size: 100,  // Replicate 100 documents per batch
```

### Compression

Enable compression for bandwidth optimization:

```rust
compression_enabled: true,  // Compress replication data
```

### Connection Pooling

The system automatically manages connection pools to remote datacenters with:

- Maximum connection limits
- Connection timeout handling
- Automatic reconnection on failures

## Troubleshooting

### Common Issues

1. **High Replication Lag**
   - Check network bandwidth and latency
   - Increase batch size for better throughput
   - Verify datacenter health status

2. **Connection Failures**
   - Verify endpoint URLs and network connectivity
   - Check firewall rules and port accessibility
   - Validate TLS certificates

3. **Data Inconsistencies**
   - Monitor conflict resolution statistics
   - Review vector clock synchronization
   - Check for network partitions

### Debug Logging

Enable debug logging for replication:

```rust
RUST_LOG=aerolithsdb_storage::datacenter_replication=debug
```

## Best Practices

1. **Regional Proximity**: Deploy datacenters in geographically close regions for lower latency
2. **Monitoring**: Implement comprehensive monitoring for replication health
3. **Testing**: Regularly test failover and recovery procedures
4. **Capacity Planning**: Monitor bandwidth usage and plan for growth
5. **Security**: Regularly rotate TLS certificates and review access controls

## Future Enhancements

The cross-datacenter replication system is designed for future enhancements:

- **Multi-master writes**: Support for writes in multiple datacenters
- **Geo-routing**: Automatic routing based on client location
- **Disaster recovery**: Automated failover and recovery procedures
- **Conflict prevention**: Predictive conflict detection and prevention
- **Performance optimization**: ML-driven optimization of replication strategies
