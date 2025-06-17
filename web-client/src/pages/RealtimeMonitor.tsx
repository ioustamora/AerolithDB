import React from 'react'
import { Card, Row, Col, Typography, Tag, List } from 'antd'
import { 
  EyeOutlined,
  ThunderboltOutlined,
  DatabaseOutlined,
  GlobalOutlined
} from '@ant-design/icons'

const { Title, Text } = Typography

const RealtimeMonitor: React.FC = () => {
  const recentEvents = [
    { id: 1, type: 'document_created', message: 'Document created in users collection', timestamp: new Date() },
    { id: 2, type: 'query_executed', message: 'Query executed on node-2', timestamp: new Date(Date.now() - 5000) },
    { id: 3, type: 'node_joined', message: 'New node joined the cluster', timestamp: new Date(Date.now() - 10000) },
    { id: 4, type: 'replication_completed', message: 'Data replicated to node-4', timestamp: new Date(Date.now() - 15000) },
  ]

  return (
    <div style={{ padding: '24px' }}>
      <Title level={2}>
        <EyeOutlined style={{ marginRight: '12px' }} />
        Real-time Monitor
      </Title>

      <Row gutter={[16, 16]}>
        <Col span={24}>
          <Card title="Live Activity Feed">
            <List
              dataSource={recentEvents}
              renderItem={(item) => (
                <List.Item>
                  <div style={{ display: 'flex', justifyContent: 'space-between', width: '100%' }}>
                    <div>
                      <Tag color={item.type === 'document_created' ? 'green' : 'blue'}>
                        {item.type.replace('_', ' ').toUpperCase()}
                      </Tag>
                      <Text>{item.message}</Text>
                    </div>
                    <Text type="secondary">{item.timestamp.toLocaleTimeString()}</Text>
                  </div>
                </List.Item>
              )}
            />
          </Card>
        </Col>
      </Row>
    </div>
  )
}

export default RealtimeMonitor
