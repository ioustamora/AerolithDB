# Cryptocurrency Wallet Integration for AerolithDB

## Overview

AerolithDB now includes comprehensive cryptocurrency wallet integration supporting Tron (USDT) and Solana (USDC) networks for service payments. This integration provides both web-based and CLI access to wallet functionality.

## Features

### Supported Networks and Tokens
- **Tron Network**: USDT (TRC-20) payments
- **Solana Network**: USDC (SPL Token) payments

### Web Client Integration
- React-based wallet connector component
- Payment dashboard with balance display
- Transaction history and status tracking
- Service subscription management

### CLI Integration
- Command-line wallet management
- Payment processing and confirmation
- Balance checking and transaction history
- Service purchasing capabilities

## Implementation Status

### Backend (Rust)
✅ **Payment Plugin Framework** 
- Abstract payment plugin trait system
- Tron and Solana blockchain provider scaffolds
- Payment transaction lifecycle management
- Fee estimation and confirmation tracking

✅ **REST API Endpoints**
- `/api/v1/payment/wallets/connect` - Wallet connection
- `/api/v1/payment/wallets/balance` - Balance checking
- `/api/v1/payment/transactions/create` - Payment creation
- `/api/v1/payment/transactions/{id}/confirm` - Payment confirmation
- `/api/v1/payment/history` - Transaction history
- `/api/v1/payment/pricing` - Service pricing tiers

✅ **Plugin Architecture Integration**
- Payment plugins integrated with core plugin system
- Blockchain abstraction layer for multi-network support
- Configuration management for testnet/mainnet environments

### Web Client (React/TypeScript)
✅ **UI Components**
- `WalletConnector` component for wallet connection
- `PaymentDashboard` for payment management
- `PaymentCenter` page for centralized payment operations
- Integration with existing web client routing

✅ **API Service Layer**
- PaymentService class for API communication
- Type definitions for payment operations
- Integration with existing API client infrastructure

### CLI (Rust)
✅ **Command Structure**
- `crypto-wallet connect` - Connect to wallet
- `crypto-wallet balance` - Check token balances
- `crypto-wallet pay` - Make payments
- `crypto-wallet history` - View transaction history
- `crypto-wallet status` - Check connection status

## Usage Examples

### CLI Usage
```bash
# Connect to Tron wallet
aerolithdb-cli crypto-wallet connect --network tron --address TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t

# Check USDT balance
aerolithdb-cli crypto-wallet balance --network tron --address TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t --token usdt

# Make a payment
aerolithdb-cli crypto-wallet pay --amount 10000000 --token usdt --from TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t --network tron --service database-storage

# View payment history
aerolithdb-cli crypto-wallet history --address TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t --network tron
```

### Web Client Usage
1. Navigate to `/payments` in the web interface
2. Click "Connect Wallet" 
3. Select network (Tron or Solana)
4. Connect via supported wallet extensions (TronLink, Phantom, Solflare)
5. View balances and make payments through the dashboard

## Security Features

- **Signature Verification**: All transactions require wallet signature verification
- **Network Validation**: Strict network and contract address validation
- **Rate Limiting**: API endpoint rate limiting to prevent abuse
- **Audit Logging**: Complete transaction audit trails
- **Testnet Support**: Safe testing environment for development

## Service Integration

### Pricing Tiers
- **Basic**: 10 USDT/USDC - Basic database access
- **Standard**: 50 USDT/USDC - Enhanced features and storage
- **Premium**: 200 USDT/USDC - Full feature set with priority support

### Billing Integration
- Automatic service activation upon payment confirmation
- Subscription management and renewal tracking
- Usage monitoring and limit enforcement
- Payment failure handling and retry logic

## Development Status

### Completed ✅
- Core payment plugin architecture
- Basic blockchain provider implementation
- REST API endpoint structure
- Web client UI components
- CLI command framework
- Documentation and examples

### In Progress 🔧
- Real blockchain network integration (currently using mock data)
- Production-ready error handling
- Comprehensive testing suite
- Security audit and validation

### Planned 📋
- Additional network support (Ethereum, Polygon)
- Multi-signature wallet support
- Recurring payment subscriptions
- Advanced analytics and reporting

## Configuration

### Environment Setup
```toml
[payment]
network = "testnet"  # or "mainnet"
tron_rpc = "https://api.shasta.trongrid.io"
solana_rpc = "https://api.devnet.solana.com"
supported_tokens = ["USDT", "USDC"]
```

### Smart Contract Addresses
- **Tron USDT (Testnet)**: TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t
- **Solana USDC (Devnet)**: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v

## Next Steps

1. **Complete Backend Integration**: Finish payment manager initialization and plugin registration
2. **Real Blockchain Integration**: Replace mock implementations with actual network calls
3. **Web Client Finalization**: Complete UI integration and API wiring
4. **Testing and Validation**: Comprehensive testing on testnet environments
5. **Security Review**: Complete security audit before mainnet deployment
6. **Documentation**: User guides and API documentation

## File Structure

```
aerolithdb-plugins/src/
├── payment.rs              # Payment plugin framework
├── blockchain/
│   ├── mod.rs             # Blockchain abstraction
│   ├── tron.rs            # Tron network provider
│   └── solana.rs          # Solana network provider

aerolithdb-api/src/
└── payment.rs              # REST API endpoints

aerolithdb-cli/src/
└── crypto_wallet.rs        # CLI wallet commands

web-client/src/
├── services/
│   └── PaymentService.ts   # API client service
├── components/
│   ├── wallet/
│   │   └── WalletConnector.tsx
│   └── payment/
│       └── PaymentDashboard.tsx
└── pages/
    └── PaymentCenter.tsx   # Main payment page
```

This implementation provides a solid foundation for cryptocurrency payment integration in AerolithDB, with support for both Tron and Solana networks using USDT and USDC tokens respectively.
