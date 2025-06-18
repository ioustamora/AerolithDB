import React, { useState } from 'react';
import {
  Card,
  Button,
  Table,
  Typography,
  Space,
  Tag,
  Select,
  InputNumber,
  Form,
  Modal,
  Alert,
  Steps,
  notification,
  Row,
  Col,
  Statistic
} from 'antd';
import {
  CreditCardOutlined,
  WalletOutlined,
  CheckCircleOutlined,
  ClockCircleOutlined,
  DollarOutlined
} from '@ant-design/icons';

const { Title, Text } = Typography;
const { Option } = Select;
const { Step } = Steps;

interface ServiceTier {
  id: string;
  name: string;
  pricePerMonth: Record<string, number>;
  features: {
    apiCallsPerDay: number;
    storageGB: number;
    supportLevel: string;
  };
}

interface PaymentTransaction {
  id: string;
  amount: string;
  token: string;
  service: string;
  status: 'pending' | 'confirmed' | 'failed';
  createdAt: string;
  networkHash?: string;
}

const PaymentDashboard: React.FC = () => {
  const [selectedTier, setSelectedTier] = useState<string>('');
  const [selectedToken, setSelectedToken] = useState<string>('usdc');
  const [duration, setDuration] = useState<number>(1);
  const [paymentModalVisible, setPaymentModalVisible] = useState(false);
  const [currentStep, setCurrentStep] = useState(0);
  const [processing, setProcessing] = useState(false);

  const serviceTiers: ServiceTier[] = [
    {
      id: 'starter',
      name: 'Starter',
      pricePerMonth: { usdt: 10, usdc: 10 },
      features: {
        apiCallsPerDay: 10000,
        storageGB: 1,
        supportLevel: 'Basic',
      },
    },
    {
      id: 'professional',
      name: 'Professional',
      pricePerMonth: { usdt: 50, usdc: 50 },
      features: {
        apiCallsPerDay: 100000,
        storageGB: 10,
        supportLevel: 'Premium',
      },
    },
    {
      id: 'enterprise',
      name: 'Enterprise',
      pricePerMonth: { usdt: 200, usdc: 200 },
      features: {
        apiCallsPerDay: 1000000,
        storageGB: 100,
        supportLevel: 'Enterprise',
      },
    },
  ];

  const mockTransactions: PaymentTransaction[] = [
    {
      id: 'tx_001',
      amount: '50.00',
      token: 'USDC',
      service: 'Professional Plan',
      status: 'confirmed',
      createdAt: '2024-01-15T10:30:00Z',
      networkHash: '0xabc123...def456',
    },
    {
      id: 'tx_002',
      amount: '10.00',
      token: 'USDT',
      service: 'Starter Plan',
      status: 'pending',
      createdAt: '2024-01-14T15:45:00Z',
    },
  ];

  const columns = [
    {
      title: 'Transaction ID',
      dataIndex: 'id',
      key: 'id',
      render: (id: string) => <Text code>{id}</Text>,
    },
    {
      title: 'Amount',
      dataIndex: 'amount',
      key: 'amount',
      render: (amount: string, record: PaymentTransaction) => (
        <Space>
          <Text strong>${amount}</Text>
          <Tag>{record.token}</Tag>
        </Space>
      ),
    },
    {
      title: 'Service',
      dataIndex: 'service',
      key: 'service',
    },
    {
      title: 'Status',
      dataIndex: 'status',
      key: 'status',
      render: (status: string) => {
        const statusConfig = {
          pending: { color: 'orange', icon: <ClockCircleOutlined /> },
          confirmed: { color: 'green', icon: <CheckCircleOutlined /> },
          failed: { color: 'red', icon: null },
        };
        
        const config = statusConfig[status as keyof typeof statusConfig];
        return (
          <Tag color={config.color} icon={config.icon}>
            {status.toUpperCase()}
          </Tag>
        );
      },
    },
    {
      title: 'Date',
      dataIndex: 'createdAt',
      key: 'createdAt',
      render: (date: string) => new Date(date).toLocaleDateString(),
    },
  ];

  const calculateTotal = () => {
    const tier = serviceTiers.find(t => t.id === selectedTier);
    if (!tier) return 0;
    return tier.pricePerMonth[selectedToken] * duration;
  };

  const handlePurchase = () => {
    setPaymentModalVisible(true);
    setCurrentStep(0);
  };

  const processPayment = async () => {
    setProcessing(true);
    setCurrentStep(1);

    // Simulate payment processing
    await new Promise(resolve => setTimeout(resolve, 2000));
    setCurrentStep(2);

    await new Promise(resolve => setTimeout(resolve, 2000));
    setCurrentStep(3);

    setProcessing(false);
    
    notification.success({
      message: 'Payment Successful',
      description: 'Your service has been activated successfully!',
    });

    setTimeout(() => {
      setPaymentModalVisible(false);
      setCurrentStep(0);
    }, 2000);
  };

  const paymentSteps = [
    {
      title: 'Review',
      description: 'Review payment details',
    },
    {
      title: 'Sign',
      description: 'Sign transaction in wallet',
    },
    {
      title: 'Broadcast',
      description: 'Broadcasting to network',
    },
    {
      title: 'Confirm',
      description: 'Awaiting confirmation',
    },
  ];

  return (
    <div style={{ padding: '24px' }}>
      <Title level={2}>
        <CreditCardOutlined style={{ marginRight: '12px' }} />
        Payment Dashboard
      </Title>

      <Row gutter={[24, 24]}>
        <Col xs={24} lg={16}>
          <Card title="Service Plans" style={{ marginBottom: 24 }}>
            <Row gutter={[16, 16]}>
              {serviceTiers.map(tier => (
                <Col xs={24} md={8} key={tier.id}>
                  <Card
                    hoverable
                    className={selectedTier === tier.id ? 'selected-tier' : ''}
                    onClick={() => setSelectedTier(tier.id)}
                    style={{
                      border: selectedTier === tier.id ? '2px solid #1890ff' : '1px solid #d9d9d9',
                    }}
                  >
                    <div style={{ textAlign: 'center' }}>
                      <Title level={4}>{tier.name}</Title>
                      <div style={{ marginBottom: 16 }}>
                        <Statistic
                          title="Monthly Price"
                          value={tier.pricePerMonth.usdc}
                          prefix="$"
                          valueStyle={{ color: '#1890ff' }}
                        />
                      </div>
                      <Space direction="vertical" size="small">
                        <Text>📊 {tier.features.apiCallsPerDay.toLocaleString()} API calls/day</Text>
                        <Text>💾 {tier.features.storageGB} GB storage</Text>
                        <Text>🎧 {tier.features.supportLevel} support</Text>
                      </Space>
                    </div>
                  </Card>
                </Col>
              ))}
            </Row>
          </Card>

          <Card title="Payment History">
            <Table
              columns={columns}
              dataSource={mockTransactions}
              rowKey="id"
              pagination={{ pageSize: 10 }}
            />
          </Card>
        </Col>

        <Col xs={24} lg={8}>
          <Card title="Purchase Service" style={{ position: 'sticky', top: 24 }}>
            <Form layout="vertical">
              <Form.Item label="Selected Plan">
                <Select
                  value={selectedTier}
                  onChange={setSelectedTier}
                  placeholder="Select a service tier"
                  size="large"
                >
                  {serviceTiers.map(tier => (
                    <Option key={tier.id} value={tier.id}>
                      {tier.name} - ${tier.pricePerMonth.usdc}/month
                    </Option>
                  ))}
                </Select>
              </Form.Item>

              <Form.Item label="Payment Token">
                <Select
                  value={selectedToken}
                  onChange={setSelectedToken}
                  size="large"
                >
                  <Option value="usdc">USDC</Option>
                  <Option value="usdt">USDT</Option>
                </Select>
              </Form.Item>

              <Form.Item label="Duration (months)">
                <InputNumber
                  value={duration}
                  onChange={(value) => setDuration(value || 1)}
                  min={1}
                  max={12}
                  size="large"
                  style={{ width: '100%' }}
                />
              </Form.Item>

              {selectedTier && (
                <Alert
                  message="Payment Summary"
                  description={
                    <div>
                      <div>Service: {serviceTiers.find(t => t.id === selectedTier)?.name}</div>
                      <div>Duration: {duration} month{duration > 1 ? 's' : ''}</div>
                      <div style={{ marginTop: 8 }}>
                        <Text strong style={{ fontSize: '16px' }}>
                          Total: ${calculateTotal()} {selectedToken.toUpperCase()}
                        </Text>
                      </div>
                    </div>
                  }
                  type="info"
                  showIcon
                  style={{ marginBottom: 16 }}
                />
              )}

              <Button
                type="primary"
                size="large"
                icon={<WalletOutlined />}
                onClick={handlePurchase}
                disabled={!selectedTier}
                style={{ width: '100%' }}
              >
                Purchase with Crypto
              </Button>
            </Form>
          </Card>
        </Col>
      </Row>

      <Modal
        title="Complete Payment"
        open={paymentModalVisible}
        onCancel={() => setPaymentModalVisible(false)}
        footer={null}
        width={600}
      >
        <Steps current={currentStep} style={{ marginBottom: 24 }}>
          {paymentSteps.map(step => (
            <Step key={step.title} title={step.title} description={step.description} />
          ))}
        </Steps>

        {currentStep === 0 && (
          <div>
            <Alert
              message="Payment Details"
              description={
                <div>
                  <div>Service: {serviceTiers.find(t => t.id === selectedTier)?.name}</div>
                  <div>Amount: ${calculateTotal()} {selectedToken.toUpperCase()}</div>
                  <div>Duration: {duration} month{duration > 1 ? 's' : ''}</div>
                </div>
              }
              type="info"
              style={{ marginBottom: 16 }}
            />

            <div style={{ textAlign: 'center' }}>
              <Button
                type="primary"
                size="large"
                icon={<DollarOutlined />}
                onClick={processPayment}
                loading={processing}
              >
                Proceed with Payment
              </Button>
            </div>
          </div>
        )}

        {currentStep > 0 && currentStep < 3 && (
          <div style={{ textAlign: 'center', padding: '40px 0' }}>
            <Text>Processing payment...</Text>
          </div>
        )}

        {currentStep === 3 && (
          <Alert
            message="Payment Completed!"
            description="Your service has been activated and is ready to use."
            type="success"
            showIcon
          />
        )}
      </Modal>

      <style jsx>{`
        .selected-tier {
          box-shadow: 0 4px 12px rgba(24, 144, 255, 0.15);
        }
      `}</style>
    </div>
  );
};

export default PaymentDashboard;
