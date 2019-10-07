import React from 'react';
import { Button, Col, Container, Form, Jumbotron, Row } from 'react-bootstrap';
import 'bootstrap/dist/css/bootstrap.css';
import './App.css';

const EDITOR_ROWS: number = 12;

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
      <React.Fragment>
        <Container>
          <Row>
            <Col>
              <Jumbotron>
                <h1>rufus</h1>
                <p>
                  An experiment about a CEK machine implemented in Rust,
                  compiled to Web Assembly and made alive via Typescript + React.
                </p>
              </Jumbotron>
            </Col>
          </Row>
          <Row>
            <Col xs={8}>
              <Form>
                <Form.Group>
                  <Form.Label>Program:</Form.Label>
                  <textarea
                    className='form-control'
                    rows={EDITOR_ROWS}
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
                    rows={EDITOR_ROWS}
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
        <footer className="fixed-bottom">
          <div className="footer-copyright text-center py-1">
            <span className="text-muted">
              © 2019 <a href="https://github.com/hurryabit/rufus" target="blank">Martin Huschenbett</a>
            </span>
          </div>
        </footer>
      </React.Fragment>
    );
  }
}

export default App;
