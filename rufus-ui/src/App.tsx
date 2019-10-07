import React from 'react';
import { Button, Col, Container, Form, Row } from 'react-bootstrap';
import 'bootstrap/dist/css/bootstrap.css';
import './App.css';

type Props = {};

type State = {
  wasm: typeof import('rufus') | null;
  program: string;
  output: string;
  result: string;
}

class App extends React.Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = {
      wasm: null,
      program: '',
      output: '',
      result: '',
    };
  }
  componentDidMount() {
    this.loadWasm();
  }

  loadWasm = async () => {
    try {
      const wasm = await import('rufus');
      this.setState({wasm});
    } catch(err) {
      console.error(`Unexpected error in loadWasm. [Message: ${err.message}]`);
    }
  }

  handleProgramChange = (event: React.ChangeEvent<HTMLTextAreaElement>) => {
    this.setState({program: event.currentTarget.value})
  }

  handleProgramKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === 'Enter' && event.metaKey) {
      this.handleRunClick(event);
    }
  }

  handleRunClick = (event: React.SyntheticEvent) => {
    event.preventDefault();
    const wasm = this.state.wasm;
    if (!wasm) {
      alert("WASM not loaded!");
      return;
    }
    const result = wasm.exec(this.state.program);
    const status = result.status;
    const value = result.get_value();
    switch (status) {
      case wasm.ExecResultStatus.Ok:
        this.setState({result: value});
        break;
      case wasm.ExecResultStatus.Err:
        alert(value);
        break;
    }
  }

  render() {
    const state = this.state;
    return (
      <Container>
        <Row>
          <Col xs={8}>
            <Form>
              <Form.Group>
                <Form.Label>Program:</Form.Label>
                <textarea
                  className='form-control'
                  rows={20}
                  value={state.program}
                  onChange={this.handleProgramChange}
                  onKeyDown={this.handleProgramKeyDown}
                />
              </Form.Group>
              <Form.Group>
                <Form.Label>&nbsp;</Form.Label>
                <Button
                  variant="primary"
                  className='btn-block'
                  type="submit"
                  onClick={this.handleRunClick}
                >
                  Run
                </Button>
              </Form.Group>
            </Form>
          </Col>
          <Col xs={4}>
            <Form>
              <Form.Group>
                <Form.Label>Output:</Form.Label>
                <Form.Control
                  as="textarea"
                  readOnly
                  rows={20}
                  value={state.output}
                />
              </Form.Group>
              <Form.Group>
                <Form.Label>Result:</Form.Label>
                <Form.Control
                  type='text'
                  readOnly
                  value={state.result}
                />
              </Form.Group>
            </Form>
          </Col>
        </Row>
      </Container>
    );
  }
}

export default App;
