import { Layout, Menu, Button } from 'antd';
import { createUseStyles } from 'react-jss';
import BasicSettings from './sections/BasicSettings';
import KeyUpload from './sections/KeyUpload';

const { Header, Footer, Content } = Layout;

const useStyles = createUseStyles({
  page: {
    minHeight: '100vh',
    display: 'flex',
    flexDirection: 'column',
  },
  content: {
    display: 'flex',
    flexDirection: 'column',
    flexGrow: 1,
    paddingTop: '2vh',
    textAlign: 'center',
    alignItems: 'center',
    alignSelf: 'center',
    width: '60%',
  },
  footer: {
    textAlign: 'center',
  },
  seperator: {
    width: '20%',
  },
  button: {
    marginTop: 8,
  },
});

const App: React.FC = () => {
  const classes = useStyles();

  return (
    <div className={classes.page}>
      <Layout>
        <Header>
          <div className={classes.seperator} />
          <Menu
            theme="dark"
            mode="horizontal"
            defaultSelectedKeys={['0']}
            items={[
              { key: 0, label: 'Mint' },
              { key: 1, label: 'About' },
            ]}
          />
          <div className={classes.seperator} />
        </Header>
        <Content className={classes.content}>
          <BasicSettings />
          <KeyUpload />
          <Button className={classes.button} type="primary" size="large">
            Start Minting
          </Button>
        </Content>
        <Footer className={classes.footer}>
          CNFT Forge ©2022 made with ❤️ by Fabian Bormann
        </Footer>
      </Layout>
    </div>
  );
};

export default App;
