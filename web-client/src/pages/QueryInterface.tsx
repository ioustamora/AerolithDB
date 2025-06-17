import React, { useState } from 'react'
import { 
  Card, 
  Button, 
  Space, 
  Typography, 
  Input,
  Select,
  Table,
  Tabs,
  Alert
} from 'antd'
import { 
  SearchOutlined,
  PlayCircleOutlined,
  SaveOutlined,
  HistoryOutlined
} from '@ant-design/icons'

const { Title } = Typography
const { TextArea } = Input
const { Option } = Select
const { TabPane } = Tabs

const QueryInterface: React.FC = () => {
  const [query, setQuery] = useState('')
  const [collection, setCollection] = useState('users')
  const [results, setResults] = useState<any[]>([])
  const [loading, setLoading] = useState(false)

  const mockResults = [
    { _id: 'user_001', name: 'Alice Johnson', email: 'alice@example.com' },
    { _id: 'user_002', name: 'Bob Smith', email: 'bob@example.com' }
  ]

  const executeQuery = () => {
    setLoading(true)
    setTimeout(() => {
      setResults(mockResults)
      setLoading(false)
    }, 1000)
  }

  const columns = [
    { title: 'ID', dataIndex: '_id', key: '_id' },
    { title: 'Name', dataIndex: 'name', key: 'name' },
    { title: 'Email', dataIndex: 'email', key: 'email' }
  ]

  return (
    <div style={{ padding: '24px' }}>
      <Title level={2}>
        <SearchOutlined style={{ marginRight: '12px' }} />
        Query Interface
      </Title>

      <Card style={{ marginBottom: '16px' }}>
        <Space direction="vertical" style={{ width: '100%' }}>
          <div style={{ display: 'flex', gap: '16px', alignItems: 'center' }}>
            <span>Collection:</span>
            <Select value={collection} onChange={setCollection} style={{ width: 200 }}>
              <Option value="users">users</Option>
              <Option value="products">products</Option>
              <Option value="orders">orders</Option>
            </Select>
            <Button type="primary" icon={<PlayCircleOutlined />} onClick={executeQuery} loading={loading}>
              Execute Query
            </Button>
            <Button icon={<SaveOutlined />}>Save Query</Button>
            <Button icon={<HistoryOutlined />}>Query History</Button>
          </div>
          
          <TextArea
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Enter your query (JSON format)..."
            rows={6}
            style={{ fontFamily: 'monospace' }}
          />
        </Space>
      </Card>

      <Card title={`Results (${results.length} documents)`}>
        <Table
          columns={columns}
          dataSource={results}
          rowKey="_id"
          pagination={{ pageSize: 50 }}
          loading={loading}
        />
      </Card>
    </div>
  )
}

export default QueryInterface
