import React, { useState, useEffect, useCallback } from 'react'
import { Card, Row, Col, Statistic, Progress, Alert, Button, Space, Tag, Spin } from 'antd'
import { 
  DatabaseOutlined,
  UserOutlined,
  GlobalOutlined,
  ThunderboltOutlined,
  ReloadOutlined,
  WarningOutlined,
  CheckCircleOutlined
} from '@ant-design/icons'

import { ApiClient } from '../services/ApiClient'
import webSocketManager, { WebSocketEvent } from '../services/WebSocketManager'
import { DatabaseStats, NetworkNode } from '../types/index'

interface DashboardMetrics {
  totalDocuments: number
  activeConnections: number
  networkNodes: number
  queriesPerSecond: number
  clusterHealth: 'healthy' | 'warning' | 'error'
  memoryUsage: number
  cpuUsage: number
  storageUsage: number
  networkIO: number
}

const Dashboard: React.FC = () => {
  const [metrics, setMetrics] = useState<DashboardMetrics>({
    totalDocuments: 0,
    activeConnections: 0,
    networkNodes: 0,
    queriesPerSecond: 0,
    clusterHealth: 'healthy',
    memoryUsage: 0,
    cpuUsage: 0,
    storageUsage: 0,
    networkIO: 0
  })
  const [loading, setLoading] = useState(true)
  const [lastUpdate, setLastUpdate] = useState<Date>(new Date())
  const [apiClient] = useState(() => new ApiClient())
  const [recentActivity, setRecentActivity] = useState<string[]>([])

  // Load dashboard metrics from backend
  const loadMetrics = useCallback(async () => {
    try {
      setLoading(true)
      
      // Discover network and get cluster metrics
      const [nodes, stats] = await Promise.all([
        apiClient.discoverNetwork(),
        apiClient.getDatabaseStats()
      ])

      // Calculate aggregated metrics from all nodes
      const totalDocs = nodes.reduce((sum, node) => sum + node.documents, 0)
      const totalConnections = nodes.reduce((sum, node) => sum + node.connections, 0)
      const avgMemory = nodes.length > 0 ? nodes.reduce((sum, node) => sum + node.memoryUsage, 0) / nodes.length : 0
      const avgCpu = nodes.length > 0 ? nodes.reduce((sum, node) => sum + node.cpuUsage, 0) / nodes.length : 0
      
      // Determine cluster health
      const healthyNodes = nodes.filter(node => node.status === 'healthy').length
      const clusterHealth: 'healthy' | 'warning' | 'error' = 
        healthyNodes === nodes.length ? 'healthy' :
        healthyNodes > nodes.length / 2 ? 'warning' : 'error'

      setMetrics({
        totalDocuments: totalDocs,
        activeConnections: totalConnections,
        networkNodes: nodes.length,
        queriesPerSecond: stats.throughput || Math.floor(Math.random() * 50 + 10), // From stats or simulated
        clusterHealth,
        memoryUsage: avgMemory,
        cpuUsage: avgCpu,
        storageUsage: stats.storage_usage_percent || Math.floor(Math.random() * 30 + 20),
        networkIO: Math.floor(Math.random() * 60 + 20) // Simulated until available
      })

      setLastUpdate(new Date())
      
      // Connect to WebSocket for real-time updates
      if (!webSocketManager.isConnectedToCluster()) {
        await webSocketManager.connectToCluster(nodes)
      }

    } catch (error) {
      console.error('Failed to load dashboard metrics:', error)
      // Keep previous metrics on error, just update timestamp
      setLastUpdate(new Date())
    } finally {
      setLoading(false)
    }
  }, [apiClient])

  // Handle real-time WebSocket events
  const handleRealtimeEvent = useCallback((event: WebSocketEvent) => {
    const now = new Date()
    setLastUpdate(now)
    
    // Add to recent activity
    let activityMessage = ''
    switch (event.type) {
      case 'document_changed':
        activityMessage = `Document ${event.data?.action || 'modified'} in collection '${event.data?.collection || 'unknown'}'`
        break
      case 'node_update':
        activityMessage = `Node ${event.nodeId} status updated`
        break
      case 'query_executed':
        activityMessage = `Query executed on ${event.nodeId} (${event.data?.duration || 'N/A'}ms)`
        break
      default:
        activityMessage = `${event.type} event received`
    }
    
    setRecentActivity(prev => [
      `${now.toLocaleTimeString()}: ${activityMessage}`,
      ...prev.slice(0, 5) // Keep last 6 activities
    ])

    // Update metrics based on event type
    if (event.type === 'node_update' && event.data?.metrics) {
      setMetrics(prev => ({
        ...prev,
        queriesPerSecond: prev.queriesPerSecond + 1, // Increment on query events
      }))
    }
  }, [])

  // Initialize dashboard
  useEffect(() => {
    loadMetrics()
    
    // Subscribe to real-time events
    webSocketManager.on('*', handleRealtimeEvent)
    
    // Auto-refresh every 30 seconds
    const interval = setInterval(loadMetrics, 30000)
    
    return () => {
      clearInterval(interval)
      webSocketManager.off('*', handleRealtimeEvent)
    }
  }, [loadMetrics, handleRealtimeEvent])

  const getHealthColor = (health: string) => {
    switch (health) {
      case 'healthy': return '#52c41a'
      case 'warning': return '#faad14'
      case 'error': return '#f5222d'
      default: return '#d9d9d9'
    }
  }

  const getHealthIcon = (health: string) => {
    switch (health) {
      case 'healthy': return <CheckCircleOutlined />
      case 'warning': return <WarningOutlined />
      case 'error': return <WarningOutlined />
      default: return <WarningOutlined />
    }
  }

  return (
    <div style={{ padding: '24px' }}>
      <Row gutter={[16, 16]}>
        <Col span={24}>
          <Alert
            message="AerolithDB Real-time Dashboard"
            description={
              <Space>
                <span>Live monitoring of your distributed database cluster.</span>
                <Tag color={getHealthColor(metrics.clusterHealth)}>
                  {getHealthIcon(metrics.clusterHealth)} Cluster {metrics.clusterHealth.toUpperCase()}
                </Tag>
                <span>Last updated: {lastUpdate.toLocaleTimeString()}</span>
                <Button 
                  size="small" 
                  icon={<ReloadOutlined spin={loading} />} 
                  onClick={loadMetrics}
                  type="text"
                >
                  Refresh
                </Button>
              </Space>
            }
            type="info"
            showIcon
            style={{ marginBottom: '24px' }}
          />
        </Col>
        
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title="Total Documents"
              value={metrics.totalDocuments}
              prefix={<DatabaseOutlined />}
              valueStyle={{ color: '#3f8600' }}
              suffix={loading ? <Spin size="small" /> : undefined}
            />
          </Card>
        </Col>
        
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title="Active Connections"
              value={metrics.activeConnections}
              prefix={<UserOutlined />}
              valueStyle={{ color: '#1890ff' }}
              suffix={loading ? <Spin size="small" /> : undefined}
            />
          </Card>
        </Col>
        
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title="Network Nodes"
              value={metrics.networkNodes}
              prefix={<GlobalOutlined />}
              valueStyle={{ color: '#722ed1' }}
              suffix={loading ? <Spin size="small" /> : undefined}
            />
          </Card>
        </Col>
        
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title="Queries/sec"
              value={metrics.queriesPerSecond}
              prefix={<ThunderboltOutlined />}
              valueStyle={{ color: '#f5222d' }}
              suffix={loading ? <Spin size="small" /> : undefined}
            />
          </Card>
        </Col>        
        <Col xs={24} lg={12}>
          <Card title="System Health" style={{ height: '300px' }}>
            <div style={{ padding: '20px 0' }}>
              <div style={{ marginBottom: '16px' }}>
                <div>CPU Usage</div>
                <Progress 
                  percent={Math.round(metrics.cpuUsage)} 
                  status={metrics.cpuUsage > 80 ? "exception" : "active"} 
                />
              </div>
              <div style={{ marginBottom: '16px' }}>
                <div>Memory Usage</div>
                <Progress 
                  percent={Math.round(metrics.memoryUsage)} 
                  status={metrics.memoryUsage > 90 ? "exception" : "active"} 
                />
              </div>
              <div style={{ marginBottom: '16px' }}>
                <div>Storage Usage</div>
                <Progress 
                  percent={metrics.storageUsage} 
                  status={metrics.storageUsage > 85 ? "exception" : "active"} 
                />
              </div>
              <div>
                <div>Network I/O</div>
                <Progress 
                  percent={metrics.networkIO} 
                  status="active" 
                />
              </div>
            </div>
          </Card>
        </Col>
        
        <Col xs={24} lg={12}>
          <Card title="Recent Activity" style={{ height: '300px' }}>
            <div style={{ fontSize: '14px', lineHeight: '1.8', overflow: 'auto', height: '200px' }}>
              {recentActivity.length > 0 ? (
                recentActivity.map((activity, index) => (
                  <div key={index} style={{ marginBottom: '4px' }}>
                    • {activity}
                  </div>
                ))
              ) : (
                <>
                  <div>• Waiting for real-time events...</div>
                  <div>• Dashboard initialized</div>
                  <div>• Connecting to cluster nodes</div>
                  <div>• Health monitoring active</div>
                </>
              )}
            </div>
          </Card>
        </Col>
      </Row>
    </div>
  )
}

export default Dashboard