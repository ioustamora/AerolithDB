export interface WebSocketEvent {
  type: 'network_status' | 'node_update' | 'document_changed' | 'query_executed' | 'connection_changed'
  timestamp: string
  nodeId?: string
  data: any
}

export interface NetworkEvent extends WebSocketEvent {
  type: 'network_status'
  data: {
    totalNodes: number
    healthyNodes: number
    networkHealth: 'healthy' | 'degraded' | 'critical'
  }
}

export interface NodeUpdateEvent extends WebSocketEvent {
  type: 'node_update'
  data: {
    nodeId: string
    status: 'healthy' | 'warning' | 'error' | 'connecting'
    metrics: {
      cpuUsage: number
      memoryUsage: number
      connections: number
      documents: number
    }
  }
}

export class WebSocketManager {
  private connections = new Map<string, WebSocket>()
  private eventHandlers = new Map<string, ((event: WebSocketEvent) => void)[]>()
  private reconnectAttempts = new Map<string, number>()
  private maxReconnectAttempts = 5
  private isConnected = false

  constructor() {
    this.setupHeartbeat()
  }

  // Connect to cluster WebSocket endpoints
  async connectToCluster(nodes: Array<{ id: string; address: string; port: number }>): Promise<void> {
    console.log('🔌 Connecting to cluster WebSocket endpoints...')
    
    for (const node of nodes) {
      await this.connectToNode(node)
    }
    
    this.isConnected = this.connections.size > 0
    console.log(`✅ Connected to ${this.connections.size}/${nodes.length} nodes`)
  }
  // Connect to individual node WebSocket
  private async connectToNode(node: { id: string; address: string; port: number }): Promise<void> {
    // WebSocket port mapping: try multiple possible endpoints
    const wsEndpoints = [
      `ws://${node.address}:${node.port + 3}/ws`, // Standard offset
      `ws://${node.address}:${node.port}/ws`,     // Same port
      `ws://${node.address}:8083/ws`,             // Default WebSocket port
    ]
    
    for (const wsUrl of wsEndpoints) {
      try {
        console.log(`🔌 Attempting to connect to ${node.id} at ${wsUrl}`)
        const ws = new WebSocket(wsUrl)
        
        ws.onopen = () => {
          console.log(`✅ Connected to ${node.id} WebSocket`)
          this.connections.set(node.id, ws)
          this.reconnectAttempts.set(node.id, 0)
          
          // Send subscription for AerolithDB WebSocket API events
          const subscriptionMessage = {
            type: 'subscribe',
            collection: null, // Subscribe to all collections
            query: null       // No specific query filter
          }
          ws.send(JSON.stringify(subscriptionMessage))
        }

        ws.onmessage = (event) => {
          try {
            const wsEvent = JSON.parse(event.data)
            
            // Map AerolithDB WebSocket events to our format
            const mappedEvent = this.mapAerolithEvent(wsEvent, node.id)
            if (mappedEvent) {
              this.handleWebSocketEvent(mappedEvent)
            }
          } catch (error) {
            console.error('Failed to parse WebSocket message:', error)
          }
        }

        ws.onclose = () => {
          console.warn(`🔌 Disconnected from ${node.id} WebSocket`)
          this.connections.delete(node.id)
          this.scheduleReconnect(node)
        }

        ws.onerror = (error) => {
          console.error(`❌ WebSocket error for ${node.id}:`, error)
        }

        // If connection attempt is successful, break the loop
        await new Promise((resolve, reject) => {
          const timeout = setTimeout(() => reject(new Error('Connection timeout')), 5000)
          ws.onopen = () => {
            clearTimeout(timeout)
            resolve(ws)
          }
          ws.onerror = () => {
            clearTimeout(timeout)
            reject(new Error('Connection failed'))
          }
        })
        
        break // Successfully connected, exit the endpoint loop
        
      } catch (error) {
        console.warn(`Failed to connect to ${wsUrl}:`, error)
        continue // Try next endpoint
      }
    }
  }

  // Map AerolithDB WebSocket events to our client format
  private mapAerolithEvent(aerolithEvent: any, nodeId: string): WebSocketEvent | null {
    if (!aerolithEvent.type) return null

    switch (aerolithEvent.type) {
      case 'DocumentChanged':
        return {
          type: 'document_changed',
          timestamp: aerolithEvent.timestamp || new Date().toISOString(),
          nodeId,
          data: {
            collection: aerolithEvent.collection,
            document_id: aerolithEvent.document_id,
            action: aerolithEvent.action,
            data: aerolithEvent.data
          }
        }

      case 'QueryUpdate':
        return {
          type: 'query_executed',
          timestamp: aerolithEvent.timestamp || new Date().toISOString(),
          nodeId,
          data: {
            query_id: aerolithEvent.query_id,
            results: aerolithEvent.results,
            count: aerolithEvent.count
          }
        }

      case 'ConnectionStatus':
        return {
          type: 'connection_changed',
          timestamp: new Date().toISOString(),
          nodeId,
          data: {
            status: aerolithEvent.status,
            message: aerolithEvent.message
          }
        }

      case 'Error':
        console.error(`WebSocket error from ${nodeId}:`, aerolithEvent)
        return {
          type: 'connection_changed',
          timestamp: aerolithEvent.timestamp || new Date().toISOString(),
          nodeId,
          data: {
            status: 'error',
            message: aerolithEvent.message,
            code: aerolithEvent.code
          }
        }

      default:
        // Unknown event type, create a generic event
        return {
          type: 'network_status',
          timestamp: new Date().toISOString(),
          nodeId,
          data: aerolithEvent
        }
    }
  }

  // Handle incoming WebSocket events
  private handleWebSocketEvent(event: WebSocketEvent): void {
    // Emit to all registered handlers for this event type
    const handlers = this.eventHandlers.get(event.type) || []
    handlers.forEach(handler => {
      try {
        handler(event)
      } catch (error) {
        console.error('Error in WebSocket event handler:', error)
      }
    })

    // Emit to global event handlers
    const globalHandlers = this.eventHandlers.get('*') || []
    globalHandlers.forEach(handler => {
      try {
        handler(event)
      } catch (error) {
        console.error('Error in global WebSocket event handler:', error)
      }
    })
  }

  // Subscribe to WebSocket events
  subscribe(eventType: string, handler: (event: WebSocketEvent) => void): () => void {
    if (!this.eventHandlers.has(eventType)) {
      this.eventHandlers.set(eventType, [])
    }
    
    this.eventHandlers.get(eventType)!.push(handler)
    
    // Return unsubscribe function
    return () => {
      const handlers = this.eventHandlers.get(eventType) || []
      const index = handlers.indexOf(handler)
      if (index > -1) {
        handlers.splice(index, 1)
      }
    }
  }

  // Subscribe to all events
  subscribeToAll(handler: (event: WebSocketEvent) => void): () => void {
    return this.subscribe('*', handler)
  }

  // Send message to specific node
  sendToNode(nodeId: string, message: any): boolean {
    const ws = this.connections.get(nodeId)
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify(message))
      return true
    }
    return false
  }

  // Broadcast message to all connected nodes
  broadcast(message: any): number {
    let sent = 0
    this.connections.forEach((ws, nodeId) => {
      if (ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify(message))
        sent++
      }
    })
    return sent
  }

  // Schedule reconnection to a node
  private scheduleReconnect(node: { id: string; address: string; port: number }): void {
    const attempts = this.reconnectAttempts.get(node.id) || 0
    
    if (attempts < this.maxReconnectAttempts) {
      const delay = Math.min(1000 * Math.pow(2, attempts), 30000) // Exponential backoff, max 30s
      
      setTimeout(() => {
        console.log(`🔄 Attempting to reconnect to ${node.id} (attempt ${attempts + 1})`)
        this.reconnectAttempts.set(node.id, attempts + 1)
        this.connectToNode(node)
      }, delay)
    } else {
      console.error(`❌ Max reconnection attempts reached for ${node.id}`)
    }
  }

  // Setup heartbeat to detect network issues
  private setupHeartbeat(): void {
    setInterval(() => {
      this.connections.forEach((ws, nodeId) => {
        if (ws.readyState === WebSocket.OPEN) {
          ws.send(JSON.stringify({ type: 'ping', timestamp: Date.now() }))
        }
      })
    }, 30000) // Heartbeat every 30 seconds
  }

  // Get connection status
  getConnectionStatus(): {
    connected: number
    total: number
    nodes: Array<{ nodeId: string; status: 'connected' | 'disconnected' | 'connecting' }>
  } {
    const nodes: Array<{ nodeId: string; status: 'connected' | 'disconnected' | 'connecting' }> = []
    
    this.connections.forEach((ws, nodeId) => {
      nodes.push({
        nodeId,
        status: ws.readyState === WebSocket.OPEN ? 'connected' : 
               ws.readyState === WebSocket.CONNECTING ? 'connecting' : 'disconnected'
      })
    })

    return {
      connected: nodes.filter(n => n.status === 'connected').length,
      total: nodes.length,
      nodes
    }
  }

  // Cleanup all connections
  disconnect(): void {
    console.log('🔌 Disconnecting from all WebSocket connections...')
    
    this.connections.forEach((ws, nodeId) => {
      if (ws.readyState === WebSocket.OPEN) {
        ws.close()
      }
    })
    
    this.connections.clear()
    this.eventHandlers.clear()
    this.reconnectAttempts.clear()
    this.isConnected = false
  }

  // Check if manager is connected to cluster
  isConnectedToCluster(): boolean {
    return this.isConnected && this.connections.size > 0
  }

  // Get active connections count
  getActiveConnectionsCount(): number {
    let active = 0
    this.connections.forEach(ws => {
      if (ws.readyState === WebSocket.OPEN) {
        active++
      }
    })
    return active
  }
}

// Export singleton instance
export const webSocketManager = new WebSocketManager()
export default webSocketManager
