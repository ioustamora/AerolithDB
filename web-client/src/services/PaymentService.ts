import { ApiClient } from './ApiClient';

export interface WalletConnectionRequest {
  network: string;
  address: string;
  signature?: string;
}

export interface WalletConnection {
  connected: boolean;
  address: string;
  network: string;
  balances: TokenBalance[];
}

export interface TokenBalance {
  token: string;
  amount: string;
  formatted_amount: string;
  decimals: number;
}

export interface PaymentRequest {
  amount: string;
  token: string;
  from_address: string;
  network: string;
  service: string;
  description?: string;
}

export interface PaymentTransaction {
  id: string;
  status: string;
  amount: string;
  token: string;
  network: string;
  service: string;
  created_at: string;
  transaction_hash?: string;
  confirmation_count?: number;
}

export interface PaymentHistory {
  transactions: PaymentTransaction[];
  total: number;
  page: number;
  limit: number;
}

export interface PricingTier {
  service: string;
  tier: string;
  price_usdt: string;
  price_usdc: string;
  features: string[];
  limits: Record<string, any>;
}

class PaymentService {
  private apiClient: ApiClient;

  constructor() {
    this.apiClient = new ApiClient();
  }

  async connectWallet(request: WalletConnectionRequest): Promise<WalletConnection> {
    const response = await this.apiClient['client'].post('/payment/wallets/connect', request);
    return response.data;
  }

  async getWalletBalance(address: string, network: string, token?: string): Promise<TokenBalance[]> {
    const params = new URLSearchParams({ address, network });
    if (token) {
      params.append('token', token);
    }
    
    const response = await this.apiClient['client'].get(`/payment/wallets/balance?${params}`);
    return response.data.balances;
  }

  async disconnectWallet(): Promise<void> {
    await this.apiClient['client'].post('/payment/wallets/disconnect');
  }

  async createPayment(request: PaymentRequest): Promise<PaymentTransaction> {
    const response = await this.apiClient['client'].post('/payment/transactions/create', request);
    return response.data;
  }

  async getTransactionStatus(transactionId: string): Promise<PaymentTransaction> {
    const response = await this.apiClient['client'].get(`/payment/transactions/${transactionId}`);
    return response.data;
  }

  async confirmPayment(transactionId: string, transactionHash: string): Promise<PaymentTransaction> {
    const response = await this.apiClient['client'].post(`/payment/transactions/${transactionId}/confirm`, {
      transaction_hash: transactionHash
    });
    return response.data;
  }

  async getPaymentHistory(address: string, network?: string, limit: number = 10): Promise<PaymentHistory> {
    const params = new URLSearchParams({ address, limit: limit.toString() });
    if (network) {
      params.append('network', network);
    }

    const response = await this.apiClient['client'].get(`/payment/history?${params}`);
    return response.data;
  }

  async getPricingTiers(): Promise<PricingTier[]> {
    const response = await this.apiClient['client'].get('/payment/pricing');
    return response.data.tiers;
  }

  async purchaseService(service: string, tier: string, paymentMethod: PaymentRequest): Promise<PaymentTransaction> {
    const response = await this.apiClient['client'].post('/payment/service/purchase', {
      service,
      tier,
      payment_method: paymentMethod
    });
    return response.data;
  }
}

export const paymentService = new PaymentService();
export default PaymentService;
