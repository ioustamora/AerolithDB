import React, { useState, useEffect, useCallback } from 'react'
import { 
  Card, 
  Row, 
  Col, 
  Table, 
  Tag, 
  Progress, 
  Statistic, 
  Badge,
  Space,
  Button,
  Typography,
  Divider,
  Alert,
  notification
} from 'antd'
import { 
  GlobalOutlined,
  ThunderboltOutlined,
  DatabaseOutlined,
  ReloadOutlined,
  ClusterOutlined,
  WarningOutlined,
  CheckCircleOutlined,
  ExclamationCircleOutlined
} from '@ant-design/icons'

import { ApiClient } from '../services/ApiClient'
import webSocketManager, { WebSocketEvent, NetworkEvent, NodeUpdateEvent } from '../services/WebSocketManager'

const { Title, Text } = Typography

interface NetworkNode {
  id: string
  address: string
  port: number
  status: 'healthy' | 'warning' | 'error' | 'connecting'
  role: 'bootstrap' | 'regular'
  version: string
  uptime: string
  connections: number
  documents: number
  memoryUsage: number
  cpuUsage: number
  lastSeen: string
}

const NetworkExplorer: React.FC = () => {
  const [nodes, setNodes] = useState<NetworkNode[]>([])
  const [loading, setLoading] = useState(false)
  const [apiClient] = useState(() => new ApiClient())
  const [wsConnected, setWsConnected] = useState(false)
  const [lastUpdate, setLastUpdate] = useState<Date>(new Date())
    // Real-time network discovery and monitoring
  const discoverNetwork = async () => {
    setLoading(true)
    try {
      const discoveredNodes = await apiClient.discoverNetwork()
      setNodes(discoveredNodes)
      setLastUpdate(new Date())
      
      // Connect to WebSocket endpoints for real-time updates
      await webSocketManager.connectToCluster(discoveredNodes)
      setWsConnected(webSocketManager.isConnectedToCluster())
      
      notification.success({
        message: 'Network Discovery Complete',
        description: `Found ${discoveredNodes.length} nodes in the cluster`,
        duration: 3
      })
    } catch (error) {
      console.error('Failed to discover network:', error)
      notification.error({
        message: 'Network Discovery Failed',
        description: 'Could not connect to the network. Check if nodes are running.',
        duration: 5
      })
    } finally {
      setLoading(false)
    }
  }

  // Handle real-time WebSocket events
  const handleNetworkEvent = useCallback((event: WebSocketEvent) => {
    setLastUpdate(new Date())
    
    if (event.type === 'node_update') {
      const nodeEvent = event as NodeUpdateEvent
      setNodes(prevNodes => 
        prevNodes.map(node => 
          node.id === nodeEvent.data.nodeId 
            ? { 
                ...node, 
                status: nodeEvent.data.status,
                connections: nodeEvent.data.metrics.connections,
                documents: nodeEvent.data.metrics.documents,
                memoryUsage: nodeEvent.data.metrics.memoryUsage,
                cpuUsage: nodeEvent.data.metrics.cpuUsage,
                lastSeen: 'just now'
              }
            : node
        )
      )
    } else if (event.type === 'network_status') {
      const networkEvent = event as NetworkEvent
      // Update overall network health status
      console.log('Network status update:', networkEvent.data)
    }
  }, [])

  // Initialize network discovery and WebSocket subscriptions
  useEffect(() => {
    discoverNetwork()
    
    // Subscribe to WebSocket events
    const unsubscribeNodeUpdates = webSocketManager.subscribe('node_update', handleNetworkEvent)
    const unsubscribeNetworkStatus = webSocketManager.subscribe('network_status', handleNetworkEvent)
    
    // Auto-refresh network every 30 seconds
    const refreshInterval = setInterval(() => {
      if (!webSocketManager.isConnectedToCluster()) {
        discoverNetwork()
      }
    }, 30000)
    
    return () => {
      unsubscribeNodeUpdates()
      unsubscribeNetworkStatus()
      clearInterval(refreshInterval)
      webSocketManager.disconnect()
    }
  }, [handleNetworkEvent])

  const refreshNodes = () => {
    discoverNetwork()
  }

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'healthy': return 'green'
      case 'warning': return 'orange'
      case 'error': return 'red'
      case 'connecting': return 'blue'
      default: return 'gray'
    }
  }

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'warning': return <WarningOutlined />
      case 'error': return <WarningOutlined />
      default: return <ClusterOutlined />
    }
  }

  const columns = [
    {
      title: 'Node ID',
      dataIndex: 'id',
      key: 'id',
      render: (text: string, record: NetworkNode) => (
        <Space>
          {getStatusIcon(record.status)}
          <Text strong>{text}</Text>
          {record.role === 'bootstrap' && <Tag color="gold">Bootstrap</Tag>}
        </Space>
      ),
    },
    {
      title: 'Address',
      key: 'address',
      render: (record: NetworkNode) => `${record.address}:${record.port}`,
    },
    {
      title: 'Status',
      dataIndex: 'status',
      key: 'status',
      render: (status: string) => (
        <Badge 
          status={getStatusColor(status) as any} 
          text={status.charAt(0).toUpperCase() + status.slice(1)} 
        />
      ),
    },
    {
      title: 'Uptime',
      dataIndex: 'uptime',
      key: 'uptime',
    },
    {
      title: 'Connections',
      dataIndex: 'connections',
      key: 'connections',
      render: (connections: number) => (
        <Badge count={connections} showZero color="#1890ff" />
      ),
    },
    {
      title: 'Documents',
      dataIndex: 'documents',
      key: 'documents',
      render: (docs: number) => docs.toLocaleString(),
    },
    {
      title: 'Memory',
      dataIndex: 'memoryUsage',
      key: 'memoryUsage',
      render: (usage: number) => (
        <Progress 
          percent={usage} 
          size="small" 
          status={usage > 80 ? 'exception' : 'active'}
          format={() => `${usage}%`}
        />
      ),
    },
    {
      title: 'CPU',
      dataIndex: 'cpuUsage',
      key: 'cpuUsage',
      render: (usage: number) => (
        <Progress 
          percent={usage} 
          size="small" 
          status={usage > 70 ? 'exception' : 'active'}
          format={() => `${usage}%`}
        />
      ),
    },
    {
      title: 'Last Seen',
      dataIndex: 'lastSeen',
      key: 'lastSeen',
    },
  ]

  const healthyNodes = nodes.filter(n => n.status === 'healthy').length
  const totalNodes = nodes.length
  const totalConnections = nodes.reduce((sum, node) => sum + node.connections, 0)
  const totalDocuments = nodes.reduce((sum, node) => sum + node.documents, 0)

  return (
    <div style={{ padding: '24px' }}>      <div style={{ marginBottom: '24px', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div style={{ display: 'flex', alignItems: 'center' }}>
          <Title level={2} style={{ margin: 0, marginRight: '16px' }}>
            <GlobalOutlined style={{ marginRight: '12px' }} />
            Network Explorer
          </Title>
          <Badge 
            status={wsConnected ? "processing" : "error"} 
            text={wsConnected ? "Real-time Connected" : "Disconnected"}
            style={{ fontSize: '12px' }}
          />
          {lastUpdate && (
            <Text type="secondary" style={{ marginLeft: '16px', fontSize: '12px' }}>
              Last updated: {lastUpdate.toLocaleTimeString()}
            </Text>
          )}
        </div>
        <Space>
          <Button 
            type="primary" 
            icon={<ReloadOutlined />} 
            onClick={refreshNodes}
            loading={loading}
          >
            Refresh Network
          </Button>
        </Space>
      </div>

      <Row gutter={[16, 16]} style={{ marginBottom: '24px' }}>
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title="Network Health"
              value={`${healthyNodes}/${totalNodes}`}
              prefix={<ClusterOutlined />}
              valueStyle={{ 
                color: healthyNodes === totalNodes ? '#3f8600' : '#faad14' 
              }}
              suffix="nodes"
            />
          </Card>
        </Col>
        
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title="Total Connections"
              value={totalConnections}
              prefix={<ThunderboltOutlined />}
              valueStyle={{ color: '#1890ff' }}
            />
          </Card>
        </Col>
        
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title="Total Documents"
              value={totalDocuments}
              prefix={<DatabaseOutlined />}
              valueStyle={{ color: '#722ed1' }}
            />
          </Card>
        </Col>
        
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title="Network Topology"
              value="Mesh"
              prefix={<GlobalOutlined />}
              valueStyle={{ color: '#52c41a' }}
            />
          </Card>
        </Col>
      </Row>

      {nodes.some(n => n.status === 'warning' || n.status === 'error') && (
        <Alert
          message="Network Issues Detected"
          description="Some nodes are experiencing performance issues or connectivity problems. Check individual node status below."
          type="warning"
          showIcon
          style={{ marginBottom: '24px' }}
        />
      )}

      <Card 
        title="Cluster Nodes" 
        extra={
          <Space>
            <Text type="secondary">Last updated: {new Date().toLocaleTimeString()}</Text>
          </Space>
        }
      >
        <Table
          columns={columns}
          dataSource={nodes}
          rowKey="id"
          loading={loading}
          pagination={false}
          size="small"
        />
      </Card>

      <Divider />

      <Row gutter={[16, 16]} style={{ marginTop: '24px' }}>
        <Col xs={24} lg={12}>
          <Card title="Network Topology Visualization" style={{ height: '300px' }}>
            <div style={{ 
              display: 'flex', 
              justifyContent: 'center', 
              alignItems: 'center', 
              height: '200px',
              fontSize: '14px',
              color: '#8c8c8c'
            }}>
              <div style={{ textAlign: 'center' }}>
                <GlobalOutlined style={{ fontSize: '48px', marginBottom: '16px' }} />
                <div>Interactive network topology visualization</div>
                <div>Coming in future release</div>
              </div>
            </div>
          </Card>
        </Col>
        
        <Col xs={24} lg={12}>
          <Card title="Real-time Metrics" style={{ height: '300px' }}>
            <div style={{ 
              display: 'flex', 
              justifyContent: 'center', 
              alignItems: 'center', 
              height: '200px',
              fontSize: '14px',
              color: '#8c8c8c'
            }}>
              <div style={{ textAlign: 'center' }}>
                <ThunderboltOutlined style={{ fontSize: '48px', marginBottom: '16px' }} />
                <div>Real-time network performance charts</div>
                <div>Coming in future release</div>
              </div>
            </div>
          </Card>
        </Col>
      </Row>
    </div>
  )
}

export default NetworkExplorer
