import React from 'react'
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom'
import { Layout } from 'antd'
import AppHeader from '@components/layout/AppHeader'
import AppSidebar from '@components/layout/AppSidebar'
import Dashboard from '@pages/Dashboard'
import DataBrowser from '@pages/DataBrowser'
import QueryInterface from '@pages/QueryInterface'
import NetworkExplorer from '@pages/NetworkExplorer'
import RealtimeMonitor from '@pages/RealtimeMonitor'
import Administration from '@pages/Administration'
import Analytics from '@pages/Analytics'
import NotFound from '@pages/NotFound'

const { Content } = Layout

const App: React.FC = () => {
  return (
    <Router>
      <Layout className="app-layout">
        <AppHeader />
        <Layout>
          <AppSidebar />
          <Layout>
            <Content className="app-content">              <Routes>
                <Route path="/" element={<Dashboard />} />
                <Route path="/data" element={<DataBrowser />} />
                <Route path="/data/:collection" element={<DataBrowser />} />
                <Route path="/data/:collection/:documentId" element={<DataBrowser />} />
                <Route path="/query" element={<QueryInterface />} />
                <Route path="/network" element={<NetworkExplorer />} />
                <Route path="/realtime" element={<RealtimeMonitor />} />
                <Route path="/admin" element={<Administration />} />
                <Route path="/analytics" element={<Analytics />} />
                <Route path="*" element={<NotFound />} />
              </Routes>
            </Content>
          </Layout>
        </Layout>
      </Layout>
    </Router>
  )
}

export default App
