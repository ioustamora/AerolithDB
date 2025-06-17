import React from 'react'
import { Card, Row, Col, Typography, Form, Input, Button, Switch, Select, Table } from 'antd'
import { 
  SettingOutlined,
  UserOutlined,
  DatabaseOutlined,
  SecurityScanOutlined
} from '@ant-design/icons'

const { Title } = Typography
const { Option } = Select

const Administration: React.FC = () => {
  const userColumns = [
    { title: 'Username', dataIndex: 'username', key: 'username' },
    { title: 'Role', dataIndex: 'role', key: 'role' },
    { title: 'Last Login', dataIndex: 'lastLogin', key: 'lastLogin' },
    { title: 'Status', dataIndex: 'status', key: 'status' },
  ]

  const userData = [
    { key: '1', username: 'admin', role: 'Administrator', lastLogin: '2024-01-20 10:30', status: 'Active' },
    { key: '2', username: 'analyst', role: 'Analyst', lastLogin: '2024-01-20 09:15', status: 'Active' },
  ]

  return (
    <div style={{ padding: '24px' }}>
      <Title level={2}>
        <SettingOutlined style={{ marginRight: '12px' }} />
        Administration
      </Title>

      <Row gutter={[16, 16]}>
        <Col xs={24} lg={12}>
          <Card title="System Configuration" icon={<DatabaseOutlined />}>
            <Form layout="vertical">
              <Form.Item label="Max Connections">
                <Input placeholder="1000" />
              </Form.Item>
              <Form.Item label="Query Timeout (ms)">
                <Input placeholder="30000" />
              </Form.Item>
              <Form.Item label="Enable Auto-backup">
                <Switch defaultChecked />
              </Form.Item>
              <Form.Item label="Log Level">
                <Select defaultValue="info">
                  <Option value="debug">Debug</Option>
                  <Option value="info">Info</Option>
                  <Option value="warn">Warning</Option>
                  <Option value="error">Error</Option>
                </Select>
              </Form.Item>
              <Form.Item>
                <Button type="primary">Save Configuration</Button>
              </Form.Item>
            </Form>
          </Card>
        </Col>

        <Col xs={24} lg={12}>
          <Card title="Security Settings" icon={<SecurityScanOutlined />}>
            <Form layout="vertical">
              <Form.Item label="Enable Authentication">
                <Switch defaultChecked />
              </Form.Item>
              <Form.Item label="Session Timeout (hours)">
                <Input placeholder="24" />
              </Form.Item>
              <Form.Item label="Encryption Level">
                <Select defaultValue="aes256">
                  <Option value="aes128">AES-128</Option>
                  <Option value="aes256">AES-256</Option>
                </Select>
              </Form.Item>
              <Form.Item label="Enable Audit Logging">
                <Switch defaultChecked />
              </Form.Item>
              <Form.Item>
                <Button type="primary">Update Security</Button>
              </Form.Item>
            </Form>
          </Card>
        </Col>

        <Col span={24}>
          <Card title="User Management" icon={<UserOutlined />}>
            <Table 
              columns={userColumns} 
              dataSource={userData} 
              pagination={false}
              style={{ marginBottom: '16px' }}
            />
            <Button type="primary">Add User</Button>
          </Card>
        </Col>
      </Row>
    </div>
  )
}

export default Administration
