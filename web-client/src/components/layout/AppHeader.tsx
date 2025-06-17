import React from 'react'
import { Layout, Menu, Avatar, Dropdown, Badge, Button, Space, Typography } from 'antd'
import { 
  UserOutlined, 
  SettingOutlined, 
  LogoutOutlined, 
  BellOutlined,
  GlobalOutlined,
  DatabaseOutlined
} from '@ant-design/icons'
import type { MenuProps } from 'antd'

const { Header } = Layout
const { Text } = Typography

interface AppHeaderProps {
  connectionStatus?: 'connected' | 'connecting' | 'disconnected'
  notificationCount?: number
  currentUser?: {
    name: string
    role: string
  }
}

const AppHeader: React.FC<AppHeaderProps> = ({ 
  connectionStatus = 'connected',
  notificationCount = 0,
  currentUser = { name: 'Admin User', role: 'Administrator' }
}) => {
  const userMenuItems: MenuProps['items'] = [
    {
      key: 'profile',
      icon: <UserOutlined />,
      label: 'Profile Settings',
    },
    {
      key: 'settings',
      icon: <SettingOutlined />,
      label: 'Preferences',
    },
    {
      type: 'divider',
    },
    {
      key: 'logout',
      icon: <LogoutOutlined />,
      label: 'Sign Out',
      danger: true,
    },
  ]

  const getConnectionStatusColor = () => {
    switch (connectionStatus) {
      case 'connected': return '#52c41a'
      case 'connecting': return '#faad14'
      case 'disconnected': return '#f5222d'
      default: return '#d9d9d9'
    }
  }

  const getConnectionStatusText = () => {
    switch (connectionStatus) {
      case 'connected': return 'Connected'
      case 'connecting': return 'Connecting...'
      case 'disconnected': return 'Disconnected'
      default: return 'Unknown'
    }
  }

  return (
    <Header className="app-header">
      <div className="app-logo">
        <DatabaseOutlined style={{ fontSize: '24px', marginRight: '8px' }} />
        <span style={{ fontSize: '20px', fontWeight: 'bold' }}>AerolithDB</span>
      </div>
      
      <div className="app-header-center">
        <Space>
          <div className="connection-status">
            <div 
              className="status-dot"
              style={{ 
                backgroundColor: getConnectionStatusColor(),
                width: '8px',
                height: '8px',
                borderRadius: '50%',
                display: 'inline-block',
                marginRight: '8px'
              }}
            />
            <Text style={{ color: 'rgba(255, 255, 255, 0.85)' }}>
              {getConnectionStatusText()}
            </Text>
          </div>
        </Space>
      </div>

      <div className="app-header-actions">
        <Space size="middle">
          <Badge count={notificationCount} size="small">
            <Button 
              type="text" 
              icon={<BellOutlined />} 
              style={{ color: 'rgba(255, 255, 255, 0.85)' }}
            />
          </Badge>
          
          <Button 
            type="text" 
            icon={<GlobalOutlined />} 
            style={{ color: 'rgba(255, 255, 255, 0.85)' }}
            title="Network Status"
          />
          
          <Dropdown menu={{ items: userMenuItems }} placement="bottomRight">
            <div style={{ 
              display: 'flex', 
              alignItems: 'center', 
              cursor: 'pointer',
              padding: '4px 8px',
              borderRadius: '4px',
              transition: 'background-color 0.3s'
            }}>
              <Avatar icon={<UserOutlined />} size="small" style={{ marginRight: '8px' }} />
              <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start' }}>
                <Text style={{ color: 'white', fontSize: '14px' }}>{currentUser.name}</Text>
                <Text style={{ color: 'rgba(255, 255, 255, 0.65)', fontSize: '12px' }}>
                  {currentUser.role}
                </Text>
              </div>
            </div>
          </Dropdown>
        </Space>
      </div>
    </Header>
  )
}

export default AppHeader
