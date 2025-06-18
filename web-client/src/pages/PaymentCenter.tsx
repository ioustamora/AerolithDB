import React, { useState } from 'react';
import { 
  Card, 
  Row, 
  Col, 
  Button, 
  Typography, 
  Space,
  Tabs,
  Alert,
  Statistic,
  Tag
} from 'antd';
import {
  WalletOutlined,
  DollarCircleOutlined,
  HistoryOutlined,
  SettingOutlined
} from '@ant-design/icons';
import WalletConnector from '@components/wallet/WalletConnector';
import PaymentDashboard from '@components/payment/PaymentDashboard';

const { Title, Paragraph } = Typography;
const { TabPane } = Tabs;

interface WalletConnection {
  connected: boolean;
  address: string;
  network: string;
  balances: TokenBalance[];
}

interface TokenBalance {
  token: string;
  amount: string;
  formatted_amount: string;
  decimals: number;
}

const PaymentCenter: React.FC = () => {
  const [walletConnectorVisible, setWalletConnectorVisible] = useState(false);
  const [walletConnection, setWalletConnection] = useState<WalletConnection | null>(null);

  const handleConnectWallet = () => {
    setWalletConnectorVisible(true);
  };

  const handleWalletConnected = (connection: WalletConnection) => {
    setWalletConnection(connection);
    setWalletConnectorVisible(false);
  };

  const handleDisconnectWallet = () => {
    setWalletConnection(null);
  };

  return (
    <div className="payment-center">
      <div className="page-header">
        <Title level={2}>
          <WalletOutlined style={{ marginRight: 12 }} />
          Payment Center
        </Title>
        <Paragraph>
          Manage your cryptocurrency wallets and service payments using Tron (USDT) and Solana (USDC) networks.
        </Paragraph>
      </div>

      {!walletConnection ? (
        <Card>
          <div style={{ textAlign: 'center', padding: '40px 20px' }}>
            <WalletOutlined style={{ fontSize: 64, color: '#1890ff', marginBottom: 16 }} />
            <Title level={3}>Connect Your Wallet</Title>
            <Paragraph style={{ marginBottom: 24 }}>
              Connect your Tron or Solana wallet to start making payments for AerolithDB services.
              We support USDT (Tron) and USDC (Solana) for secure and fast transactions.
            </Paragraph>
            
            <Row gutter={[16, 16]} justify="center">
              <Col xs={24} sm={12} md={8}>
                <Card size="small" style={{ textAlign: 'center' }}>
                  <Title level={4}>Tron Network</Title>
                  <Tag color="red">USDT</Tag>
                  <Paragraph>Low fees, fast transactions</Paragraph>
                </Card>
              </Col>
              <Col xs={24} sm={12} md={8}>
                <Card size="small" style={{ textAlign: 'center' }}>
                  <Title level={4}>Solana Network</Title>
                  <Tag color="purple">USDC</Tag>
                  <Paragraph>Ultra-fast, low cost</Paragraph>
                </Card>
              </Col>
            </Row>

            <Button 
              type="primary" 
              size="large" 
              icon={<WalletOutlined />}
              onClick={handleConnectWallet}
              style={{ marginTop: 24 }}
            >
              Connect Wallet
            </Button>
          </div>
        </Card>
      ) : (
        <Row gutter={[24, 24]}>
          <Col span={24}>
            <Card>
              <Row justify="space-between" align="middle">
                <Col>
                  <Space direction="vertical" size="small">
                    <Title level={4} style={{ margin: 0 }}>
                      Wallet Connected
                    </Title>
                    <Space>
                      <Tag color="green">{walletConnection.network.toUpperCase()}</Tag>
                      <span style={{ fontFamily: 'monospace' }}>
                        {`${walletConnection.address.slice(0, 6)}...${walletConnection.address.slice(-4)}`}
                      </span>
                    </Space>
                  </Space>
                </Col>
                <Col>
                  <Button onClick={handleDisconnectWallet}>
                    Disconnect
                  </Button>
                </Col>
              </Row>
            </Card>
          </Col>

          <Col span={24}>
            <Card>
              <Title level={4}>Token Balances</Title>
              <Row gutter={[16, 16]}>
                {walletConnection.balances.map((balance) => (
                  <Col xs={24} sm={12} md={6} key={balance.token}>
                    <Card size="small">
                      <Statistic
                        title={balance.token.toUpperCase()}
                        value={balance.formatted_amount}
                        precision={2}
                        prefix={<DollarCircleOutlined />}
                      />
                    </Card>
                  </Col>
                ))}
              </Row>
            </Card>
          </Col>

          <Col span={24}>
            <Tabs defaultActiveKey="dashboard">
              <TabPane tab="Payment Dashboard" key="dashboard" icon={<DollarCircleOutlined />}>
                <PaymentDashboard walletConnection={walletConnection} />
              </TabPane>
              <TabPane tab="Transaction History" key="history" icon={<HistoryOutlined />}>
                <Alert 
                  message="Transaction History" 
                  description="View your payment and service subscription history."
                  type="info"
                  showIcon
                  style={{ marginBottom: 16 }}
                />
                {/* Transaction history component would go here */}
                <Card>
                  <Paragraph>Transaction history feature coming soon...</Paragraph>
                </Card>
              </TabPane>
              <TabPane tab="Settings" key="settings" icon={<SettingOutlined />}>
                <Alert 
                  message="Payment Settings" 
                  description="Configure payment preferences and notifications."
                  type="info"
                  showIcon
                  style={{ marginBottom: 16 }}
                />
                {/* Payment settings component would go here */}
                <Card>
                  <Paragraph>Payment settings feature coming soon...</Paragraph>
                </Card>
              </TabPane>
            </Tabs>
          </Col>
        </Row>
      )}

      <WalletConnector
        visible={walletConnectorVisible}
        onCancel={() => setWalletConnectorVisible(false)}
        onConnect={handleWalletConnected}
      />
    </div>
  );
};

export default PaymentCenter;
