# AerolithDB Production Deployment Guide

[![Production Ready](https://img.shields.io/badge/status-production_ready-green.svg)](https://github.com/aerolithsdb/aerolithsdb)
[![Battle Tested](https://img.shields.io/badge/battle_tested-100%25_success-brightgreen.svg)](https://github.com/aerolithsdb/aerolithsdb)

## Table of Contents

- [Overview](#overview)
- [Prerequisites](#prerequisites)
- [Security Hardening](#security-hardening)
- [High Availability Setup](#high-availability-setup)
- [Performance Tuning](#performance-tuning)
- [Monitoring and Observability](#monitoring-and-observability)
- [Backup and Recovery](#backup-and-recovery)
- [Scaling Strategies](#scaling-strategies)
- [Operational Procedures](#operational-procedures)
- [Disaster Recovery](#disaster-recovery)
- [Compliance and Auditing](#compliance-and-auditing)

## Overview

This guide covers production deployment of AerolithDB, including security hardening, high availability configuration, performance optimization, and operational best practices for enterprise environments.

### Deployment Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   Load Balancer                            │
│                (HAProxy/NGINX)                             │
└─────────────────┬───────────────┬───────────────────────────┘
                  │               │
          ┌───────▼─────┐ ┌───────▼─────┐ ┌─────────────┐
          │ AerolithDB  │ │ AerolithDB  │ │ AerolithDB  │
          │   Node 1    │ │   Node 2    │ │   Node 3    │
          │ (Primary)   │ │ (Secondary) │ │ (Secondary) │
          └─────────────┘ └─────────────┘ └─────────────┘
                  │               │               │
          ┌───────▼───────────────▼───────────────▼─────┐
          │         Storage Layer (SAN/NFS)             │
          └─────────────────────────────────────────────┘
```

## Prerequisites

### Hardware Requirements

**Minimum Production Setup (3-Node Cluster):**

- **CPU**: 8 cores per node (24 cores total)
- **RAM**: 32GB per node (96GB total)
- **Storage**: 1TB NVMe SSD + 4TB HDD per node
- **Network**: 10Gbps network interface

**Recommended Production Setup:**

- **CPU**: 16 cores per node
- **RAM**: 64GB per node
- **Storage**: 2TB NVMe SSD + 8TB HDD + Object storage
- **Network**: 25Gbps network interface with redundancy

### Operating System

**Supported Platforms:**

- Ubuntu 22.04 LTS (Recommended)
- CentOS Stream 9
- RHEL 9
- Amazon Linux 2023
- Docker (Kubernetes-ready)

### Network Requirements

- **Bandwidth**: Minimum 1Gbps, recommended 10Gbps+
- **Latency**: <5ms between nodes in same datacenter
- **Ports**: 8080 (REST), 8082 (gRPC), 8083 (WebSocket), 7946 (P2P)
- **Firewall**: Configure according to security requirements

## Security Hardening

### SSL/TLS Configuration

**Generate Production Certificates:**

```bash
# Using Let's Encrypt
certbot certonly --standalone -d aerolithdb.yourdomain.com

# Or enterprise CA
openssl req -new -x509 -days 365 -nodes \
  -out aerolithdb.crt \
  -keyout aerolithdb.key \
  -config ssl.conf
```

**Production Security Configuration:**

```yaml
# production-config.yaml
security:
  zero_trust: true
  tls_enabled: true
  tls_cert_path: "/etc/aerolithdb/certs/server.crt"
  tls_key_path: "/etc/aerolithdb/certs/server.key"
  tls_ca_path: "/etc/aerolithdb/certs/ca.crt"
  
  authentication:
    method: "jwt"
    jwt_secret_path: "/etc/aerolithdb/secrets/jwt.key"
    token_expiry: "24h"
    
  authorization:
    enabled: true
    default_policy: "deny"
    admin_users: ["admin@company.com"]
    
  encryption:
    at_rest: true
    algorithm: "AES-256-GCM"
    key_rotation_days: 90
    
  audit:
    enabled: true
    level: "comprehensive"
    retention_days: 365
    export_format: "json"
    siem_integration: true
```

### Firewall Configuration

**UFW (Ubuntu) Example:**

```bash
# Reset firewall
ufw --force reset

# Default policies
ufw default deny incoming
ufw default allow outgoing

# SSH access (restrict to management network)
ufw allow from 10.0.1.0/24 to any port 22

# AerolithDB ports (restrict to application network)
ufw allow from 10.0.2.0/24 to any port 8080  # REST API
ufw allow from 10.0.2.0/24 to any port 8082  # gRPC API
ufw allow from 10.0.2.0/24 to any port 8083  # WebSocket

# Inter-node communication
ufw allow from 10.0.3.0/24 to any port 7946  # P2P

# Enable firewall
ufw enable
```

### User and Permission Management

```bash
# Create dedicated user
useradd -r -s /bin/false aerolithdb
mkdir -p /var/lib/aerolithdb /var/log/aerolithdb /etc/aerolithdb
chown aerolithdb:aerolithdb /var/lib/aerolithdb /var/log/aerolithdb
chmod 750 /var/lib/aerolithdb /var/log/aerolithdb /etc/aerolithdb

# Set up sudo access for operations
echo "aerolithdb ALL=(ALL) NOPASSWD: /usr/local/bin/aerolithdb-cli" >> /etc/sudoers.d/aerolithdb
```

## High Availability Setup

### Multi-Node Configuration

**Bootstrap Node Configuration:**

```yaml
# bootstrap-node.yaml
node:
  node_id: "prod-bootstrap-01"
  bind_address: "10.0.3.10"
  port: 8080
  is_bootstrap: true
  datacenter: "primary"
  rack: "rack-01"

network:
  cluster_name: "production-cluster"
  bootstrap_nodes: []
  discovery_enabled: true
  mesh_networking: true
  heartbeat_interval: "5s"
  failure_timeout: "30s"

storage:
  replication_factor: 3
  consistency_level: "strong"
  data_dir: "/var/lib/aerolithdb/data"
  
  tiers:
    memory:
      size_limit: "16GB"
      eviction_policy: "lru"
    ssd:
      path: "/mnt/nvme/aerolithdb"
      size_limit: "1TB"
    hdd:
      path: "/mnt/hdd/aerolithdb" 
      size_limit: "4TB"
    archive:
      backend: "s3"
      bucket: "aerolithdb-archive"
      region: "us-east-1"

consensus:
  algorithm: "pbft"
  byzantine_tolerance: true
  min_nodes_for_consensus: 2
  election_timeout: "10s"
  
monitoring:
  metrics_enabled: true
  prometheus_port: 9090
  health_check_interval: "10s"
  
logging:
  level: "info"
  format: "json"
  output: "/var/log/aerolithdb/aerolithdb.log"
  rotation:
    enabled: true
    max_size: "100MB"
    max_files: 10
```

**Secondary Nodes Configuration:**

```yaml
# secondary-node-N.yaml
node:
  node_id: "prod-secondary-02"  # Increment for each node
  bind_address: "10.0.3.11"     # Unique IP for each node
  port: 8080

network:
  cluster_name: "production-cluster"
  bootstrap_nodes: ["10.0.3.10:8080"]  # Bootstrap node
  discovery_enabled: true

# ... rest similar to bootstrap node
```

### Load Balancer Configuration

**HAProxy Configuration:**

```haproxy
# /etc/haproxy/haproxy.cfg
global
    daemon
    maxconn 4096
    log 127.0.0.1:514 local0
    
defaults
    mode http
    timeout connect 5000ms
    timeout client 50000ms
    timeout server 50000ms
    option httplog
    
frontend aerolithdb_frontend
    bind *:80
    bind *:443 ssl crt /etc/ssl/certs/aerolithdb.pem
    redirect scheme https if !{ ssl_fc }
    
    # Health checks
    monitor-uri /health
    
    default_backend aerolithdb_nodes

backend aerolithdb_nodes
    balance roundrobin
    option httpchk GET /api/v1/health
    
    server node1 10.0.3.10:8080 check inter 5s
    server node2 10.0.3.11:8080 check inter 5s
    server node3 10.0.3.12:8080 check inter 5s
    
# gRPC load balancing
frontend grpc_frontend
    bind *:8082
    mode tcp
    default_backend grpc_nodes
    
backend grpc_nodes
    mode tcp
    balance roundrobin
    server node1 10.0.3.10:8082 check
    server node2 10.0.3.11:8082 check
    server node3 10.0.3.12:8082 check
```

### Service Management

**Systemd Service File:**

```ini
# /etc/systemd/system/aerolithdb.service
[Unit]
Description=AerolithDB Distributed Database
After=network.target
Wants=network-online.target

[Service]
Type=exec
User=aerolithdb
Group=aerolithdb
WorkingDirectory=/var/lib/aerolithdb
ExecStart=/usr/local/bin/aerolithsdb --config /etc/aerolithdb/production-config.yaml
ExecReload=/bin/kill -HUP $MAINPID
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
SyslogIdentifier=aerolithdb

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/aerolithdb /var/log/aerolithdb

# Resource limits
LimitNOFILE=65536
LimitNPROC=32768

[Install]
WantedBy=multi-user.target
```

**Service Management Commands:**

```bash
# Install and start service
systemctl daemon-reload
systemctl enable aerolithdb
systemctl start aerolithdb

# Check status
systemctl status aerolithdb
journalctl -u aerolithdb -f

# Performance monitoring
systemctl show aerolithdb --property=CPUUsageNSec
systemctl show aerolithdb --property=MemoryCurrent
```

## Performance Tuning

### System-Level Optimization

**Kernel Parameters:**

```bash
# /etc/sysctl.conf
# Network optimization
net.core.rmem_max = 134217728
net.core.wmem_max = 134217728
net.ipv4.tcp_rmem = 4096 65536 134217728
net.ipv4.tcp_wmem = 4096 65536 134217728
net.core.netdev_max_backlog = 5000

# File system optimization
fs.file-max = 1000000
fs.inotify.max_user_watches = 1048576

# Memory management
vm.swappiness = 10
vm.dirty_ratio = 15
vm.dirty_background_ratio = 5

# Apply changes
sysctl -p
```

**Storage Optimization:**

```bash
# NVMe SSD optimization
echo mq-deadline > /sys/block/nvme0n1/queue/scheduler
echo 1 > /sys/block/nvme0n1/queue/nomerges
echo 1024 > /sys/block/nvme0n1/queue/nr_requests

# HDD optimization  
echo cfq > /sys/block/sda/queue/scheduler
echo 0 > /sys/block/sda/queue/nomerges
echo 128 > /sys/block/sda/queue/nr_requests
```

### Application-Level Tuning

**Performance Configuration:**

```yaml
# performance-config.yaml
performance:
  # Memory settings
  memory_pool_size: "24GB"
  cache_size_ratio: 0.7
  buffer_pool_size: "8GB"
  
  # Storage settings
  storage_threads: 16
  compaction_threads: 4
  flush_threshold: "64MB"
  
  # Network settings
  connection_pool_size: 1000
  max_concurrent_requests: 10000
  keepalive_timeout: "60s"
  
  # Query optimization
  query_cache_size: "2GB"
  index_cache_size: "4GB"
  query_timeout: "30s"
  
  # Consensus optimization
  batch_size: 1000
  batch_timeout: "10ms"
  pipeline_depth: 10
```

## Monitoring and Observability

### Prometheus Integration

**Metrics Configuration:**

```yaml
# monitoring.yaml
monitoring:
  prometheus:
    enabled: true
    port: 9090
    endpoint: "/metrics"
    
  metrics:
    - name: "request_duration"
      type: "histogram"
      buckets: [0.001, 0.01, 0.1, 1.0, 10.0]
      
    - name: "active_connections"
      type: "gauge"
      
    - name: "storage_usage"
      type: "gauge"
      labels: ["tier", "node"]
      
    - name: "consensus_operations"
      type: "counter"
      labels: ["operation", "status"]
```

**Grafana Dashboard Configuration:**

```json
{
  "dashboard": {
    "title": "AerolithDB Production Dashboard",
    "panels": [
      {
        "title": "Request Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(aerolithdb_requests_total[5m])",
            "legendFormat": "{{method}} {{status}}"
          }
        ]
      },
      {
        "title": "Response Time",
        "type": "graph", 
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(aerolithdb_request_duration_seconds_bucket[5m]))",
            "legendFormat": "95th percentile"
          }
        ]
      },
      {
        "title": "Storage Usage",
        "type": "graph",
        "targets": [
          {
            "expr": "aerolithdb_storage_usage_bytes / aerolithdb_storage_capacity_bytes * 100",
            "legendFormat": "{{tier}} usage %"
          }
        ]
      }
    ]
  }
}
```

### Log Management

**Structured Logging Configuration:**

```yaml
logging:
  structured: true
  format: "json"
  level: "info"
  
  outputs:
    - type: "file"
      path: "/var/log/aerolithdb/application.log"
      rotation:
        size: "100MB"
        count: 10
        
    - type: "syslog"
      facility: "local0"
      tag: "aerolithdb"
      
    - type: "elasticsearch"
      hosts: ["elasticsearch-01:9200", "elasticsearch-02:9200"]
      index: "aerolithdb-logs"
      
  fields:
    service: "aerolithdb"
    environment: "production"
    datacenter: "primary"
```

### Alerting Rules

**Prometheus Alert Rules:**

```yaml
# alerts.yaml
groups:
  - name: aerolithdb
    rules:
      - alert: HighRequestLatency
        expr: histogram_quantile(0.95, rate(aerolithdb_request_duration_seconds_bucket[5m])) > 1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High request latency detected"
          
      - alert: NodeDown
        expr: up{job="aerolithdb"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "AerolithDB node is down"
          
      - alert: StorageUsageHigh
        expr: aerolithdb_storage_usage_ratio > 0.85
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Storage usage is high"
          
      - alert: ConsensusFailure
        expr: increase(aerolithdb_consensus_failures_total[5m]) > 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Consensus failures detected"
```

## Backup and Recovery

### Automated Backup Strategy

**Backup Configuration:**

```yaml
# backup-config.yaml
backup:
  strategy: "continuous"
  
  schedules:
    - name: "hourly-incremental"
      cron: "0 * * * *"
      type: "incremental"
      retention: "7d"
      
    - name: "daily-full"
      cron: "0 2 * * *" 
      type: "full"
      retention: "30d"
      
    - name: "weekly-archive"
      cron: "0 3 * * 0"
      type: "archive"
      retention: "1y"
      
  destinations:
    - type: "s3"
      bucket: "aerolithdb-backups"
      encryption: true
      compression: true
      
    - type: "tape"
      library: "/dev/nst0"
      encryption: true
```

**Backup Scripts:**

```bash
#!/bin/bash
# backup-script.sh

BACKUP_DIR="/backup/aerolithdb"
S3_BUCKET="aerolithdb-backups"
DATE=$(date +%Y%m%d_%H%M%S)

# Create backup
aerolithdb-cli backup create \
  --type full \
  --output "$BACKUP_DIR/full_$DATE.tar.gz" \
  --compression gzip \
  --encryption aes256

# Upload to S3
aws s3 cp "$BACKUP_DIR/full_$DATE.tar.gz" \
  "s3://$S3_BUCKET/full/$DATE/" \
  --storage-class GLACIER

# Cleanup old backups (keep 30 days)
find "$BACKUP_DIR" -name "full_*.tar.gz" -mtime +30 -delete

# Verify backup integrity
aerolithdb-cli backup verify \
  --backup "$BACKUP_DIR/full_$DATE.tar.gz"
```

### Disaster Recovery Procedures

**Recovery Scenarios:**

1. **Single Node Failure:**
   ```bash
   # Replace failed node
   ./scripts/replace-node.sh --old-node node-02 --new-node node-02-new
   
   # Automatic data rebalancing will occur
   aerolithdb-cli cluster rebalance --auto
   ```

2. **Data Corruption:**
   ```bash
   # Stop affected node
   systemctl stop aerolithdb
   
   # Restore from backup
   aerolithdb-cli restore \
     --backup s3://aerolithdb-backups/full/latest \
     --target /var/lib/aerolithdb/data
     
   # Start node and verify integrity
   systemctl start aerolithdb
   aerolithdb-cli verify --comprehensive
   ```

3. **Complete Cluster Failure:**
   ```bash
   # Bootstrap new cluster from backup
   aerolithdb-cli cluster bootstrap \
     --backup s3://aerolithdb-backups/full/latest \
     --nodes node-01,node-02,node-03
   ```

## Scaling Strategies

### Horizontal Scaling

**Adding New Nodes:**

```bash
# Prepare new node
./scripts/prepare-node.sh --node new-node-04 --ip 10.0.3.13

# Join to cluster
aerolithdb-cli cluster join \
  --node-id prod-node-04 \
  --bootstrap-node 10.0.3.10:8080 \
  --auto-rebalance

# Monitor rebalancing
aerolithdb-cli cluster status --verbose
```

**Auto-scaling Configuration:**

```yaml
# autoscaling.yaml
autoscaling:
  enabled: true
  
  triggers:
    - metric: "cpu_usage"
      threshold: 80
      action: "scale_up"
      cooldown: "10m"
      
    - metric: "storage_usage"
      threshold: 85
      action: "scale_up_storage"
      cooldown: "30m"
      
    - metric: "request_latency_p95"
      threshold: 1000  # 1 second
      action: "scale_up"
      cooldown: "5m"
      
  policies:
    min_nodes: 3
    max_nodes: 20
    scale_increment: 1
```

### Vertical Scaling

**Resource Scaling Procedures:**

```bash
# Memory scaling
./scripts/scale-memory.sh --node node-01 --memory 128GB

# Storage scaling  
./scripts/scale-storage.sh --node node-01 --ssd-size 4TB

# CPU scaling
./scripts/scale-cpu.sh --node node-01 --cores 32
```

## Operational Procedures

### Health Check Procedures

**Automated Health Checks:**

```bash
#!/bin/bash
# health-check.sh

# API health
curl -f http://localhost:8080/api/v1/health || exit 1

# Storage health
aerolithdb-cli storage health --all-tiers || exit 1

# Consensus health
aerolithdb-cli consensus status | grep -q "healthy" || exit 1

# Network health
aerolithdb-cli network peers | grep -q "connected" || exit 1

echo "All health checks passed"
```

### Maintenance Procedures

**Rolling Updates:**

```bash
#!/bin/bash
# rolling-update.sh

NODES=("node-01" "node-02" "node-03")

for node in "${NODES[@]}"; do
    echo "Updating $node..."
    
    # Drain traffic
    haproxy-admin disable server aerolithdb_nodes/$node
    
    # Wait for connections to drain
    sleep 30
    
    # Stop service
    ssh $node "systemctl stop aerolithdb"
    
    # Update binary
    scp aerolithsdb-new $node:/usr/local/bin/aerolithsdb
    
    # Start service
    ssh $node "systemctl start aerolithdb"
    
    # Wait for node to become healthy
    while ! curl -f http://$node:8080/api/v1/health; do
        sleep 5
    done
    
    # Re-enable traffic
    haproxy-admin enable server aerolithdb_nodes/$node
    
    echo "$node updated successfully"
done
```

### Performance Monitoring

**Daily Performance Report:**

```bash
#!/bin/bash
# performance-report.sh

DATE=$(date +%Y-%m-%d)
REPORT="/var/log/aerolithdb/performance-$DATE.txt"

echo "AerolithDB Performance Report - $DATE" > $REPORT
echo "=======================================" >> $REPORT

# Request metrics
echo "Request Metrics:" >> $REPORT
curl -s http://localhost:9090/api/v1/query?query=rate\(aerolithdb_requests_total\[24h\]\) >> $REPORT

# Latency metrics
echo "Latency Metrics:" >> $REPORT
curl -s http://localhost:9090/api/v1/query?query=histogram_quantile\(0.95,rate\(aerolithdb_request_duration_seconds_bucket\[24h\]\)\) >> $REPORT

# Storage metrics
echo "Storage Metrics:" >> $REPORT
aerolithdb-cli storage stats >> $REPORT

# Send report
mail -s "AerolithDB Performance Report - $DATE" ops-team@company.com < $REPORT
```

## Compliance and Auditing

### Audit Configuration

**Comprehensive Audit Logging:**

```yaml
# audit-config.yaml
audit:
  enabled: true
  level: "comprehensive"
  
  events:
    - "authentication"
    - "authorization" 
    - "data_access"
    - "data_modification"
    - "configuration_changes"
    - "system_events"
    
  outputs:
    - type: "file"
      path: "/var/log/aerolithdb/audit.log"
      encryption: true
      signing: true
      
    - type: "siem"
      endpoint: "https://siem.company.com/api/events"
      format: "cef"
      
  retention:
    period: "7y"
    immutable: true
    
  integrity:
    checksums: true
    digital_signatures: true
    tamper_detection: true
```

### Compliance Reports

**SOC 2 Compliance Script:**

```bash
#!/bin/bash
# soc2-compliance-report.sh

# Generate compliance report
aerolithdb-cli audit report \
  --type soc2 \
  --period "2024-01-01:2024-12-31" \
  --output "/tmp/soc2-report.pdf"

# Verify audit trail integrity
aerolithdb-cli audit verify \
  --period "2024-01-01:2024-12-31" \
  --include-checksums

# Generate security metrics
aerolithdb-cli security metrics \
  --period "2024-01-01:2024-12-31" \
  --format json \
  --output "/tmp/security-metrics.json"
```

### Data Privacy (GDPR/CCPA)

**Data Privacy Configuration:**

```yaml
# privacy-config.yaml
privacy:
  gdpr_compliance: true
  ccpa_compliance: true
  
  data_retention:
    default_period: "2y"
    automatic_deletion: true
    
  data_classification:
    pii_detection: true
    sensitive_data_encryption: true
    
  right_to_erasure:
    enabled: true
    verification_required: true
    audit_trail: true
    
  data_portability:
    export_formats: ["json", "csv", "xml"]
    encryption_required: true
```

## Troubleshooting Guide

### Common Issues and Solutions

**Issue: High Memory Usage**

```bash
# Diagnosis
aerolithdb-cli memory analyze --detailed

# Solutions
# 1. Adjust cache sizes
# 2. Enable memory compression
# 3. Tune garbage collection
```

**Issue: Slow Query Performance**

```bash
# Diagnosis
aerolithdb-cli query analyze --slow-queries

# Solutions
# 1. Add appropriate indexes
# 2. Optimize query patterns
# 3. Increase query cache
```

**Issue: Consensus Failures**

```bash
# Diagnosis
aerolithdb-cli consensus debug --detailed

# Solutions
# 1. Check network connectivity
# 2. Verify time synchronization
# 3. Check node configuration consistency
```

---

This production deployment guide provides comprehensive information for deploying AerolithDB in enterprise environments. For additional support, consult the [Developer Guide](DEVELOPER_GUIDE.md) and [Getting Started Guide](GETTING_STARTED.md).
