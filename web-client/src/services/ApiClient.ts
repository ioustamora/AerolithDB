import axios, { AxiosInstance } from 'axios'
import { NetworkNode, DatabaseStats, QueryResponse, Document } from '../types/index'

export interface ApiClientConfig {
  baseURL?: string
  timeout?: number
}

export class ApiClient {
  private client: AxiosInstance
  private knownNodes: string[] = []

  constructor(config: ApiClientConfig = {}) {
    this.client = axios.create({
      baseURL: config.baseURL || 'http://localhost:8080',
      timeout: config.timeout || 30000,
      headers: {
        'Content-Type': 'application/json'
      }
    })

    // Initialize with default bootstrap node
    this.knownNodes = ['http://localhost:8080']
  }

  // Network Discovery and Monitoring
  async discoverNetwork(): Promise<NetworkNode[]> {
    const discoveredNodes: NetworkNode[] = []
    
    // Start with bootstrap node
    for (const nodeUrl of this.knownNodes) {
      try {
        const nodeInfo = await this.getNodeInfo(nodeUrl)
        discoveredNodes.push(nodeInfo)
        
        // Get peer list from this node
        const peers = await this.getPeerList(nodeUrl)
        for (const peer of peers) {
          if (!this.knownNodes.includes(peer)) {
            this.knownNodes.push(peer)
          }
        }
      } catch (error) {
        console.warn(`Failed to connect to node: ${nodeUrl}`, error)
      }
    }

    // Query all discovered nodes
    for (const nodeUrl of this.knownNodes) {
      if (!discoveredNodes.find(n => `http://${n.address}:${n.port}` === nodeUrl)) {
        try {
          const nodeInfo = await this.getNodeInfo(nodeUrl)
          discoveredNodes.push(nodeInfo)
        } catch (error) {
          // Node might be down, add as disconnected
          discoveredNodes.push({
            id: this.extractNodeId(nodeUrl),
            address: this.extractAddress(nodeUrl),
            port: this.extractPort(nodeUrl),
            status: 'error',
            role: 'regular',
            version: 'unknown',
            uptime: '0s',
            connections: 0,
            documents: 0,
            memoryUsage: 0,
            cpuUsage: 0,
            lastSeen: 'unreachable'
          })
        }
      }
    }

    return discoveredNodes
  }
  async getNodeInfo(nodeUrl: string): Promise<NetworkNode> {
    try {
      // Try to get health and stats from the node
      const [healthResponse, statsResponse] = await Promise.all([
        this.client.get(`${nodeUrl}/health`),
        this.client.get(`${nodeUrl}/stats`).catch(() => ({ data: {} })) // Stats might not be available
      ])
      
      const health = healthResponse.data
      const stats = statsResponse.data || {}

      return {
        id: this.extractNodeId(nodeUrl),
        address: this.extractAddress(nodeUrl),
        port: this.extractPort(nodeUrl),
        status: health.status === 'healthy' ? 'healthy' : 'warning',
        role: this.extractPort(nodeUrl) === 8080 ? 'bootstrap' : 'regular',
        version: health.version || '1.0.0',
        uptime: this.formatUptime(health.uptime_seconds || 0),
        connections: stats.active_connections || stats.connections || 0,
        documents: stats.total_documents || stats.documents || 0,
        memoryUsage: Math.round((stats.memory_usage_mb || stats.memory_usage || 0) / 10.24) / 10, // Convert to percentage
        cpuUsage: Math.round((stats.cpu_usage || 0) * 100),
        lastSeen: 'just now'
      }
    } catch (error) {
      // Node is unreachable, return error state
      return {
        id: this.extractNodeId(nodeUrl),
        address: this.extractAddress(nodeUrl),
        port: this.extractPort(nodeUrl),
        status: 'error',
        role: this.extractPort(nodeUrl) === 8080 ? 'bootstrap' : 'regular',
        version: 'unknown',
        uptime: '0s',
        connections: 0,
        documents: 0,
        memoryUsage: 0,
        cpuUsage: 0,
        lastSeen: 'unreachable'
      }
    }
  }

  async getPeerList(nodeUrl: string): Promise<string[]> {
    try {
      // Try multiple possible peer endpoints
      const endpoints = [
        `${nodeUrl}/api/v1/peers`,
        `${nodeUrl}/peers`,
        `${nodeUrl}/network/peers`
      ]
      
      for (const endpoint of endpoints) {
        try {
          const response = await this.client.get(endpoint)
          if (response.data.peers) {
            return response.data.peers
          }
        } catch {
          continue
        }
      }
      
      // Fallback: derive peers from port range for local testing
      const basePort = this.extractPort(nodeUrl)
      const baseAddress = this.extractAddress(nodeUrl)
      const peers: string[] = []
      
      // Check common port range for local development
      for (let i = 0; i < 8; i++) {
        const port = 8080 + i
        if (port !== basePort) {
          peers.push(`http://${baseAddress}:${port}`)
        }
      }
      
      return peers
    } catch (error) {
      console.warn(`Failed to get peer list from ${nodeUrl}:`, error)
      return []
    }
  }

  // Database Operations
  async getStats(nodeUrl?: string): Promise<DatabaseStats> {
    const url = nodeUrl || this.knownNodes[0]
    const response = await this.client.get(`${url}/api/v1/stats`)
    return response.data
  }

  async getCollections(nodeUrl?: string): Promise<string[]> {
    const url = nodeUrl || this.knownNodes[0]
    const response = await this.client.get(`${url}/api/v1/collections`)
    return response.data.collections || []
  }

  async getDocuments(collection: string, nodeUrl?: string): Promise<Document[]> {
    const url = nodeUrl || this.knownNodes[0]
    const response = await this.client.get(`${url}/api/v1/collections/${collection}/documents`)
    return response.data.documents || []
  }

  async queryDocuments(collection: string, query: any, nodeUrl?: string): Promise<QueryResponse> {
    const url = nodeUrl || this.knownNodes[0]
    const response = await this.client.post(`${url}/api/v1/collections/${collection}/query`, query)
    return response.data
  }

  async createDocument(collection: string, document: any, nodeUrl?: string): Promise<Document> {
    const url = nodeUrl || this.knownNodes[0]
    const response = await this.client.post(`${url}/api/v1/collections/${collection}/documents`, document)
    return response.data
  }

  async getDocument(collection: string, id: string, nodeUrl?: string): Promise<Document> {
    const url = nodeUrl || this.knownNodes[0]
    const response = await this.client.get(`${url}/api/v1/collections/${collection}/documents/${id}`)
    return response.data
  }

  async updateDocument(collection: string, id: string, document: any, nodeUrl?: string): Promise<Document> {
    const url = nodeUrl || this.knownNodes[0]
    const response = await this.client.put(`${url}/api/v1/collections/${collection}/documents/${id}`, document)
    return response.data
  }

  async deleteDocument(collection: string, id: string, nodeUrl?: string): Promise<void> {
    const url = nodeUrl || this.knownNodes[0]
    await this.client.delete(`${url}/api/v1/collections/${collection}/documents/${id}`)
  }

  // Health Monitoring
  async checkNodeHealth(nodeUrl: string): Promise<boolean> {
    try {
      const response = await this.client.get(`${nodeUrl}/health`, { timeout: 5000 })
      return response.data.status === 'healthy'
    } catch (error) {
      return false
    }
  }

  async getClusterHealth(): Promise<{ healthy: number; total: number; nodes: NetworkNode[] }> {
    const nodes = await this.discoverNetwork()
    const healthy = nodes.filter(n => n.status === 'healthy').length
    
    return {
      healthy,
      total: nodes.length,
      nodes
    }
  }

  // Utility methods
  private extractNodeId(nodeUrl: string): string {
    const port = this.extractPort(nodeUrl)
    return port === 8080 ? 'node-bootstrap' : `node-${port - 8080}`
  }

  private extractAddress(nodeUrl: string): string {
    try {
      const url = new URL(nodeUrl)
      return url.hostname
    } catch {
      return 'localhost'
    }
  }

  private extractPort(nodeUrl: string): number {
    try {
      const url = new URL(nodeUrl)
      return parseInt(url.port) || 8080
    } catch {
      return 8080
    }
  }

  private formatUptime(seconds: number): string {
    const days = Math.floor(seconds / 86400)
    const hours = Math.floor((seconds % 86400) / 3600)
    const minutes = Math.floor((seconds % 3600) / 60)
    
    if (days > 0) {
      return `${days}d ${hours}h ${minutes}m`
    } else if (hours > 0) {
      return `${hours}h ${minutes}m`
    } else {
      return `${minutes}m`
    }
  }
}

// Export singleton instance
export const apiClient = new ApiClient()
export default apiClient
