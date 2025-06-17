import React, { useState } from 'react'
import { 
  Card, 
  Table, 
  Button, 
  Space, 
  Typography, 
  Tree, 
  Modal,
  Form,
  Input,
  Select,
  message
} from 'antd'
import { 
  DatabaseOutlined,
  FolderOutlined,
  FileOutlined,
  PlusOutlined,
  EditOutlined,
  DeleteOutlined
} from '@ant-design/icons'

const { Title } = Typography
const { Search } = Input
const { Option } = Select

const DataBrowser: React.FC = () => {
  const [selectedCollection, setSelectedCollection] = useState<string>('users')
  const [showCreateModal, setShowCreateModal] = useState(false)

  const collections = [
    { key: 'users', title: 'users', icon: <FolderOutlined />, children: [] },
    { key: 'products', title: 'products', icon: <FolderOutlined />, children: [] },
    { key: 'orders', title: 'orders', icon: <FolderOutlined />, children: [] },
    { key: 'analytics', title: 'analytics', icon: <FolderOutlined />, children: [] },
  ]

  const mockData = [
    {
      key: '1',
      _id: 'user_001',
      name: 'Alice Johnson',
      email: 'alice@example.com',
      department: 'Engineering',
      created: '2024-01-15T10:30:00Z'
    },
    {
      key: '2',
      _id: 'user_002', 
      name: 'Bob Smith',
      email: 'bob@example.com',
      department: 'Product',
      created: '2024-01-16T14:22:00Z'
    }
  ]

  const columns = [
    {
      title: 'Document ID',
      dataIndex: '_id',
      key: '_id',
      render: (text: string) => (
        <Space>
          <FileOutlined />
          <span style={{ fontFamily: 'monospace' }}>{text}</span>
        </Space>
      ),
    },
    {
      title: 'Name',
      dataIndex: 'name',
      key: 'name',
    },
    {
      title: 'Email',
      dataIndex: 'email',
      key: 'email',
    },
    {
      title: 'Department',
      dataIndex: 'department',
      key: 'department',
    },
    {
      title: 'Created',
      dataIndex: 'created',
      key: 'created',
      render: (date: string) => new Date(date).toLocaleDateString(),
    },
    {
      title: 'Actions',
      key: 'actions',
      render: () => (
        <Space>
          <Button size="small" icon={<EditOutlined />}>Edit</Button>
          <Button size="small" danger icon={<DeleteOutlined />}>Delete</Button>
        </Space>
      ),
    },
  ]

  const handleCreateDocument = () => {
    message.success('Document created successfully')
    setShowCreateModal(false)
  }

  return (
    <div style={{ padding: '24px' }}>
      <div style={{ marginBottom: '24px', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <Title level={2} style={{ margin: 0 }}>
          <DatabaseOutlined style={{ marginRight: '12px' }} />
          Data Browser
        </Title>
        <Button 
          type="primary" 
          icon={<PlusOutlined />}
          onClick={() => setShowCreateModal(true)}
        >
          Create Document
        </Button>
      </div>

      <div style={{ display: 'flex', gap: '16px' }}>
        <Card 
          title="Collections" 
          style={{ width: '300px', height: 'fit-content' }}
          bodyStyle={{ padding: '8px' }}
        >
          <Tree
            treeData={collections}
            defaultSelectedKeys={['users']}
            onSelect={(keys) => setSelectedCollection(keys[0] as string)}
            showIcon
          />
        </Card>

        <Card 
          title={`Collection: ${selectedCollection}`}
          style={{ flex: 1 }}
          extra={
            <Space>
              <Search 
                placeholder="Search documents..." 
                style={{ width: 200 }}
                allowClear
              />
            </Space>
          }
        >
          <Table
            columns={columns}
            dataSource={mockData}
            pagination={{ pageSize: 20 }}
            scroll={{ x: true }}
          />
        </Card>
      </div>

      <Modal
        title="Create New Document"
        open={showCreateModal}
        onCancel={() => setShowCreateModal(false)}
        footer={null}
      >
        <Form layout="vertical" onFinish={handleCreateDocument}>
          <Form.Item label="Collection" name="collection" initialValue="users">
            <Select>
              <Option value="users">users</Option>
              <Option value="products">products</Option>
              <Option value="orders">orders</Option>
              <Option value="analytics">analytics</Option>
            </Select>
          </Form.Item>
          <Form.Item label="Document ID" name="id">
            <Input placeholder="Auto-generated if empty" />
          </Form.Item>
          <Form.Item label="Document Data" name="data">
            <Input.TextArea 
              rows={6} 
              placeholder="Enter JSON document data..."
              defaultValue={JSON.stringify({ name: "", email: "", department: "" }, null, 2)}
            />
          </Form.Item>
          <Form.Item>
            <Space>
              <Button type="primary" htmlType="submit">Create</Button>
              <Button onClick={() => setShowCreateModal(false)}>Cancel</Button>
            </Space>
          </Form.Item>
        </Form>
      </Modal>
    </div>
  )
}

export default DataBrowser
