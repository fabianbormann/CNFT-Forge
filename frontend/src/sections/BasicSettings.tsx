import { Typography, Input, InputNumber } from 'antd';
import { createUseStyles } from 'react-jss';
const { Title } = Typography;

const useStyles = createUseStyles({
  form: {
    width: '100%',
  },
  field: {
    paddingTop: 6,
    paddingBottom: 6,
    textAlign: 'initial',
    '& > label': {
      marginLeft: 2,
    },
    '& > .ant-input-number': {
      width: '100%',
    },
  },
});

const BasicSettings: React.FC = () => {
  const classes = useStyles();

  return (
    <div className={classes.form}>
      <Title level={4}>Mint Settings</Title>
      <div className={classes.field}>
        <label>NFT Name</label>
        <Input placeholder="Tokenname" />
      </div>
      <div className={classes.field}>
        <label>NFT Description</label>
        <Input maxLength={60} showCount placeholder="This NFT enables ..." />
      </div>
      <div className={classes.field}>
        <label>Amount</label>
        <InputNumber min={1} defaultValue={1} />
      </div>
      <div className={classes.field}>
        <label>Payment Address</label>
        <Input placeholder="addr_test1qzm3yge..." />
      </div>
      <div className={classes.field}>
        <label>IPFS CID</label>
        <Input addonBefore="ipfs://" placeholder="QmVgeH1FsEwsmZJLuerNLcm..." />
      </div>
    </div>
  );
};

export default BasicSettings;
