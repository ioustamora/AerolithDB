// API Response Types
export interface APIResponse<T = any> {
  data: T
  status: string
  message?: string
  timestamp: string
}

export interface PaginatedResponse<T> {
  data: T[]
  pagination: {
    page: number
    limit: number
    total: number
    hasMore: boolean
  }
}

// Document Types
export interface Document {
  id: string
  collection: string
  data: Record<string, any>
  version: number
  created_at: string
  updated_at: string
}

export interface DocumentRequest {
  data: Record<string, any>
}

export interface DocumentResponse extends Document {}

// Collection Types
export interface Collection {
  name: string
  document_count: number
  size_bytes: number
  created_at: string
  updated_at: string
}

// Query Types
export interface QueryRequest {
  filter?: Record<string, any>
  sort?: Record<string, 1 | -1>
  limit?: number
  offset?: number
  projection?: string[]
}

export interface QueryResponse {
  documents: Document[]
  total: number
  execution_time_ms: number
}

// Statistics Types
export interface DatabaseStats {
  total_documents?: number
  total_collections?: number
  storage_size_bytes?: number
  index_size_bytes?: number
  uptime_seconds?: number
  version?: string
  collections?: Collection[]
  
  // Extended stats for dashboard
  total_size_bytes?: number
  throughput?: number
  average_latency?: number
  cache_hit_rate?: number
  storage_usage_percent?: number
  memory_usage?: number
  cpu_usage?: number
  active_connections?: number
  queries_per_second?: number
}

// WebSocket Event Types
export enum DocumentAction {
  Created = 'created',
  Updated = 'updated',
  Deleted = 'deleted'
}

export interface WebSocketEvent {
  type: 'document_changed' | 'query_update' | 'connection_status' | 'error' | 'network_status' | 'node_update' | 'query_executed' | 'connection_changed'
  timestamp: string
  nodeId?: string
  data: any
}

export interface DocumentChangedEvent extends WebSocketEvent {
  type: 'document_changed'
  data: {
    collection: string
    document_id: string
    action: DocumentAction
    document?: Document
  }
}

export interface QueryUpdateEvent extends WebSocketEvent {
  type: 'query_update'
  data: {
    query_id: string
    results: Document[]
    count: number
  }
}

// Network Node Types
export interface NetworkNode {
  id: string
  address: string
  port: number
  status: 'healthy' | 'warning' | 'error' | 'connecting'
  role: 'bootstrap' | 'regular'
  version: string
  uptime: string
  connections: number
  documents: number
  memoryUsage: number
  cpuUsage: number
  lastSeen: string
}

// Database Statistics Types
export interface DatabaseStats {
  total_documents?: number
  total_collections?: number
  total_size_bytes?: number
  throughput?: number
  average_latency?: number
  cache_hit_rate?: number
  storage_usage_percent?: number
  memory_usage?: number
  cpu_usage?: number
  active_connections?: number
  queries_per_second?: number
}

// Real-time Events (removed duplicate)

// Connection Types
export enum ConnectionStatus {
  Connected = 'connected',
  Connecting = 'connecting',
  Disconnected = 'disconnected',
  Error = 'error'
}

export interface ConnectionInfo {
  status: ConnectionStatus
  lastConnected?: string
  error?: string
  latency?: number
}

// User Interface Types
export interface User {
  id: string
  username: string
  email: string
  role: string
  permissions: string[]
  created_at: string
  last_login?: string
}

export interface AuthResponse {
  token: string
  user: User
  expires_at: string
}

// Navigation Types
export interface MenuItem {
  key: string
  label: string
  icon?: any
  path: string
  children?: MenuItem[]
}

// Error Types
export interface APIError {
  message: string
  code: string
  details?: Record<string, any>
  timestamp: string
}

// Settings Types
export interface AppSettings {
  theme: 'light' | 'dark' | 'auto'
  language: string
  autoRefresh: boolean
  refreshInterval: number
  notifications: boolean
  realTimeUpdates: boolean
}

// Filter and Search Types
export interface SearchFilters {
  collections?: string[]
  dateRange?: {
    start: string
    end: string
  }
  textSearch?: string
  customFilters?: Record<string, any>
}

export interface SortOption {
  field: string
  direction: 'asc' | 'desc'
}

// Chart and Analytics Types
export interface ChartDataPoint {
  name: string
  value: number
  timestamp?: string
}

export interface MetricData {
  label: string
  value: number
  unit?: string
  trend?: 'up' | 'down' | 'stable'
  change?: number
}

// Form Types
export interface FormField {
  name: string
  label: string
  type: 'text' | 'number' | 'boolean' | 'select' | 'date' | 'json'
  required?: boolean
  options?: Array<{ label: string; value: any }>
  validation?: {
    min?: number
    max?: number
    pattern?: string
    message?: string
  }
}

// Loading States
export interface LoadingState {
  isLoading: boolean
  error?: string | null
  lastUpdated?: string
}
