import React, { useState, useEffect } from 'react';
import {
  Modal,
  Button,
  Select,
  Card,
  Typography,
  Space,
  Alert,
  Spin,
  Statistic,
  Row,
  Col,
  Tag,
  notification
} from 'antd';
import {
  WalletOutlined,
  CheckCircleOutlined,
  DisconnectOutlined,
  ReloadOutlined
} from '@ant-design/icons';
import { 
  paymentService, 
  WalletConnection as PaymentWalletConnection, 
  TokenBalance as PaymentTokenBalance 
} from '../../services/PaymentService';

const { Title, Text } = Typography;
const { Option } = Select;

interface WalletConnectorProps {
  visible: boolean;
  onCancel: () => void;
  onConnect: (connection: PaymentWalletConnection) => void;
}

const WalletConnector: React.FC<WalletConnectorProps> = ({
  visible,
  onCancel,
  onConnect
}) => {
  const [selectedNetwork, setSelectedNetwork] = useState<'tron' | 'solana'>('tron');
  const [connecting, setConnecting] = useState(false);
  const [connection, setConnection] = useState<WalletConnection | null>(null);
  const [loadingBalances, setLoadingBalances] = useState(false);

  const networks = [
    { value: 'tron', label: 'Tron Network', wallets: ['TronLink'] },
    { value: 'solana', label: 'Solana Network', wallets: ['Phantom', 'Solflare'] }
  ];

  const mockBalances: Record<string, TokenBalance[]> = {
    tron: [
      { token: 'TRX', amount: '1000000000', formatted_amount: '1,000.00', decimals: 6 },
      { token: 'USDT', amount: '50000000', formatted_amount: '50.00', decimals: 6 }
    ],
    solana: [
      { token: 'SOL', amount: '2000000000', formatted_amount: '2.00', decimals: 9 },
      { token: 'USDC', amount: '75000000', formatted_amount: '75.00', decimals: 6 }
    ]
  };

  const connectWallet = async () => {
    setConnecting(true);
    
    try {
      // Simulate wallet connection
      await new Promise(resolve => setTimeout(resolve, 2000));
      
      const mockAddress = selectedNetwork === 'tron' 
        ? 'TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t'
        : 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v';
      
      const newConnection: WalletConnection = {
        connected: true,
        address: mockAddress,
        network: selectedNetwork,
        balances: mockBalances[selectedNetwork]
      };
      
      setConnection(newConnection);
      onConnect(newConnection);
      
      notification.success({
        message: 'Wallet Connected',
        description: `Successfully connected to ${selectedNetwork} wallet`,
      });
    } catch (error) {
      notification.error({
        message: 'Connection Failed',
        description: 'Failed to connect to wallet. Please try again.',
      });
    } finally {
      setConnecting(false);
    }
  };

  const disconnectWallet = () => {
    setConnection(null);
    notification.info({
      message: 'Wallet Disconnected',
      description: 'Wallet has been disconnected successfully',
    });
  };

  const refreshBalances = async () => {
    if (!connection) return;
    
    setLoadingBalances(true);
    
    // Simulate balance refresh
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    setConnection({
      ...connection,
      balances: mockBalances[connection.network]
    });
    
    setLoadingBalances(false);
    
    notification.success({
      message: 'Balances Updated',
      description: 'Wallet balances have been refreshed',
    });
  };

  const formatAddress = (address: string) => {
    return `${address.slice(0, 6)}...${address.slice(-6)}`;
  };

  return (
    <Modal
      title={
        <Space>
          <WalletOutlined />
          Cryptocurrency Wallet
        </Space>
      }
      open={visible}
      onCancel={onCancel}
      footer={null}
      width={600}
    >
      {!connection ? (
        <div>
          <Alert
            message="Connect Wallet"
            description="Connect your cryptocurrency wallet to make payments for AerolithDB services"
            type="info"
            showIcon
            style={{ marginBottom: 24 }}
          />
          
          <Card>
            <Space direction="vertical" style={{ width: '100%' }} size="large">
              <div>
                <Text strong>Select Network:</Text>
                <Select
                  value={selectedNetwork}
                  onChange={setSelectedNetwork}
                  style={{ width: '100%', marginTop: 8 }}
                  size="large"
                >
                  {networks.map(network => (
                    <Option key={network.value} value={network.value}>
                      <Space>
                        {network.label}
                        <Tag>{network.wallets.join(', ')}</Tag>
                      </Space>
                    </Option>
                  ))}
                </Select>
              </div>
              
              <Button
                type="primary"
                size="large"
                loading={connecting}
                onClick={connectWallet}
                style={{ width: '100%' }}
                icon={<WalletOutlined />}
              >
                {connecting ? 'Connecting...' : `Connect ${selectedNetwork.toUpperCase()} Wallet`}
              </Button>
            </Space>
          </Card>
        </div>
      ) : (
        <div>
          <Alert
            message="Wallet Connected"
            description={
              <Space direction="vertical">
                <Text>Network: <strong>{connection.network.toUpperCase()}</strong></Text>
                <Text>Address: <strong>{formatAddress(connection.address)}</strong></Text>
              </Space>
            }
            type="success"
            showIcon
            action={
              <Button
                size="small"
                icon={<DisconnectOutlined />}
                onClick={disconnectWallet}
              >
                Disconnect
              </Button>
            }
            style={{ marginBottom: 24 }}
          />
          
          <Card
            title={
              <Space>
                <Text strong>Token Balances</Text>
                <Button
                  size="small"
                  icon={<ReloadOutlined />}
                  loading={loadingBalances}
                  onClick={refreshBalances}
                >
                  Refresh
                </Button>
              </Space>
            }
          >
            <Row gutter={[16, 16]}>
              {connection.balances.map(balance => (
                <Col xs={24} sm={12} key={balance.token}>
                  <Card size="small">
                    <Statistic
                      title={balance.token}
                      value={balance.formatted_amount}
                      precision={2}
                      suffix={balance.token}
                      valueStyle={{ color: '#1890ff' }}
                    />
                  </Card>
                </Col>
              ))}
            </Row>
          </Card>
          
          <div style={{ marginTop: 16, textAlign: 'center' }}>
            <Text type="secondary">
              Ready to make payments with your connected wallet
            </Text>
          </div>
        </div>
      )}
    </Modal>
  );
};

export default WalletConnector;
