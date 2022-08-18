import { Col, message, Row, Upload, UploadFile } from 'antd';
import { createUseStyles } from 'react-jss';
import { InboxOutlined } from '@ant-design/icons';
const { Dragger } = Upload;

const useStyles = createUseStyles({
  uploadArea: {
    width: '100%',
    paddingTop: 6,
    paddingBottom: 6,
  },
});

const KeyUpload: React.FC = () => {
  const classes = useStyles();

  const addVerificationKey = (file: UploadFile) => {
    if (file.name.endsWith('.vkey')) {
      console.log(file);
    } else {
      message.error(
        'The verification key needs to have a *.vkey filename pattern'
      );
    }
  };

  const addSigningKey = (file: UploadFile) => {
    if (file.name.endsWith('.skey')) {
      console.log(file);
    } else {
      message.error('The signing key needs to have a *.skey filename pattern');
    }
  };

  return (
    <Row className={classes.uploadArea} gutter={[0, 12]}>
      <Col md={24} lg={12}>
        <Dragger beforeUpload={addVerificationKey}>
          <p className="ant-upload-drag-icon">
            <InboxOutlined />
          </p>
          <p className="ant-upload-text">
            Click or drag your verification key here
          </p>
          <p className="ant-upload-hint">
            Your verification key (*.vkey) will remain local.
          </p>
        </Dragger>
      </Col>
      <Col md={24} lg={12}>
        <Dragger beforeUpload={addSigningKey}>
          <p className="ant-upload-drag-icon">
            <InboxOutlined />
          </p>
          <p className="ant-upload-text">Click or drag your signing key here</p>
          <p className="ant-upload-hint">
            Your signing key (*.skey) will remain local.
          </p>
        </Dragger>
      </Col>
    </Row>
  );
};

export default KeyUpload;
