import React from 'react'
import { Card, Row, Col, Typography, Progress, Statistic } from 'antd'
import { 
  BarChartOutlined,
  LineChartOutlined,
  PieChartOutlined
} from '@ant-design/icons'

const { Title } = Typography

const Analytics: React.FC = () => {
  return (
    <div style={{ padding: '24px' }}>
      <Title level={2}>
        <BarChartOutlined style={{ marginRight: '12px' }} />
        Analytics
      </Title>

      <Row gutter={[16, 16]}>
        <Col xs={24} lg={8}>
          <Card title="Query Performance">
            <Statistic title="Avg Response Time" value={23} suffix="ms" />
            <Progress percent={75} status="active" style={{ marginTop: '16px' }} />
          </Card>
        </Col>
        
        <Col xs={24} lg={8}>
          <Card title="Storage Metrics">
            <Statistic title="Total Storage" value={1.2} suffix="GB" />
            <Progress percent={45} status="active" style={{ marginTop: '16px' }} />
          </Card>
        </Col>
        
        <Col xs={24} lg={8}>
          <Card title="Network Activity">
            <Statistic title="Requests/min" value={342} />
            <Progress percent={85} status="active" style={{ marginTop: '16px' }} />
          </Card>
        </Col>
      </Row>

      <Row gutter={[16, 16]} style={{ marginTop: '16px' }}>
        <Col span={24}>
          <Card title="Performance Charts" style={{ height: '400px' }}>
            <div style={{ 
              display: 'flex', 
              justifyContent: 'center', 
              alignItems: 'center', 
              height: '300px',
              fontSize: '16px',
              color: '#8c8c8c'
            }}>
              <div style={{ textAlign: 'center' }}>
                <LineChartOutlined style={{ fontSize: '64px', marginBottom: '16px' }} />
                <div>Interactive charts and analytics dashboard</div>
                <div>Coming in future release</div>
              </div>
            </div>
          </Card>
        </Col>
      </Row>
    </div>
  )
}

export default Analytics
