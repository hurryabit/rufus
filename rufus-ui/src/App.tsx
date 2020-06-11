import React from 'react';
import 'bulma';
import './App.css';

const EDITOR_ROWS: number = 15;

const DEFAULT_PROGRAM: string =
`let twice = fun f x -> f (f x) in
let inc = fun x -> x + 1 in
twice inc 0`;

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
      program: DEFAULT_PROGRAM,
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
      this.setState({ wasm });
    } catch (err) {
      console.error(`Unexpected error in loadWasm. [Message: ${err.message}]`);
    }
  }

  handleProgramChange = (event: React.ChangeEvent<HTMLTextAreaElement>) => {
    this.setState({ program: event.currentTarget.value })
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
        this.setState({ result: value });
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
        <section className="hero is-info">
          <div className="hero-body">
            <div className="container">
              <h1 className="title">
                rufus
              </h1>
              <h2 className="subtitle">
                An experiment about a CEK machine implemented in Rust,
                compiled to Web Assembly and made alive via Typescript + React.
              </h2>
            </div>
          </div>
        </section>
        <section className="section">
          <div className="container">
            <div className="columns">
              <div className="column is-two-thirds">
                <div className="field">
                  <label className="label">Program</label>
                  <div className="control">
                    <textarea
                      className="textarea is-family-code"
                      rows={EDITOR_ROWS}
                      value={state.program}
                      onChange={this.handleProgramChange}
                      onKeyDown={this.handleProgramKeyDown}
                    />
                  </div>
                </div>
                <div className="field">
                  <label className="label">&nbsp;</label>
                  <div className="control">
                    <button
                      className="button is-fullwidth is-info"
                      onClick={this.handleRunClick}
                    >
                      Run
                    </button>
                  </div>
                </div>
              </div>
              <div className="column is-one-third">
                <div className="field">
                  <label className="label">Output</label>
                  <div className="control">
                    <textarea
                      className="textarea is-family-code"
                      readOnly
                      rows={EDITOR_ROWS}
                      value={state.output}
                    />
                  </div>
                </div>
                <div className="field">
                  <label className="label">Result</label>
                  <div className="control">
                    <input
                      className="input is-family-code"
                      type="text"
                      readOnly
                      value={state.result}
                    />
                  </div>
                </div>
              </div>
            </div>
          </div>
        </section>
        <footer className="footer">
          <div className="content has-text-centered">
              © 2019 <a href="https://github.com/hurryabit/rufus" target="blank">Martin Huschenbett</a>
          </div>
        </footer>
      </React.Fragment>
    );
  }
}

export default App;
