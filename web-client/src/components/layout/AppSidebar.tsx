import React from 'react'
import { Layout, Menu } from 'antd'
import { 
  DashboardOutlined,
  DatabaseOutlined,
  SearchOutlined,
  EyeOutlined,
  SettingOutlined,
  BarChartOutlined,
  GlobalOutlined
} from '@ant-design/icons'
import type { MenuProps } from 'antd'
import { useLocation, useNavigate } from 'react-router-dom'

const { Sider } = Layout

interface AppSidebarProps {
  collapsed?: boolean
  onCollapse?: (collapsed: boolean) => void
}

const AppSidebar: React.FC<AppSidebarProps> = ({ collapsed = false, onCollapse }) => {
  const location = useLocation()
  const navigate = useNavigate()

  const menuItems: MenuProps['items'] = [
    {
      key: '/',
      icon: <DashboardOutlined />,
      label: 'Dashboard',
    },
    {
      key: '/data',
      icon: <DatabaseOutlined />,
      label: 'Data Browser',
    },
    {
      key: '/query',
      icon: <SearchOutlined />,
      label: 'Query Interface',
    },
    {
      key: '/network',
      icon: <GlobalOutlined />,
      label: 'Network Explorer',
    },
    {
      key: '/realtime',
      icon: <EyeOutlined />,
      label: 'Real-time Monitor',
    },
    {
      key: '/analytics',
      icon: <BarChartOutlined />,
      label: 'Analytics',
    },
    {
      type: 'divider',
    },
    {
      key: '/admin',
      icon: <SettingOutlined />,
      label: 'Administration',
    },
  ]

  const handleMenuClick: MenuProps['onClick'] = ({ key }) => {
    navigate(key)
  }

  return (
    <Sider 
      className="app-sider"
      collapsible 
      collapsed={collapsed} 
      onCollapse={onCollapse}
      width={240}
      collapsedWidth={80}
    >
      <Menu
        theme="dark"
        mode="inline"
        selectedKeys={[location.pathname]}
        items={menuItems}
        onClick={handleMenuClick}
        style={{ borderRight: 0 }}
      />
    </Sider>
  )
}

export default AppSidebar
