import React from 'react';
import './App.css';
import AceEditor from 'react-ace';

import "ace-builds/src-noconflict/mode-ocaml";
import "ace-builds/src-noconflict/theme-xcode";

const EXAMPLES_DIR: string = '/rufus/examples';

const EDITOR_ROWS: number = 20;

type Props = {};

type Example = {
  name: string;
  file: string;
}

type State = {
  wasm: typeof import('rufus-wasm') | null;
  program: string;
  output: string;
  result: string;
  examples: Example[];
}

class App extends React.Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = {
      wasm: null,
      program: '',
      output: '',
      result: '',
      examples: [],
    };
  }
  componentDidMount() {
    this.loadExamples();
    this.loadWasm();
  }

  loadExamples = async () => {
    try {
      const response = await fetch(`${EXAMPLES_DIR}/index.json`);
      const examples: Example[] = await response.json();
      this.setState({ examples });
      if (examples.length > 0) {
        this.loadExample(examples[0].file);
      }
    } catch (err) {
      console.error(`Unexptected error in loadExamples. [Message: ${err.message}]`);
    }
  }

  loadExample = async (example_file: string) => {
    try {
      const response = await fetch(`${EXAMPLES_DIR}/${example_file}`);
      const program = await response.text();
      this.setState({ program });
    } catch (err) {
      console.error(`Unexpected error in handleExampleSelect. [Message: ${err.message}]`);
      alert('Cannot load example. See console for details.');
    }
  }

  loadWasm = async () => {
    try {
      const wasm = await import('rufus-wasm');
      this.setState({ wasm });
    } catch (err) {
      console.error(`Unexpected error in loadWasm. [Message: ${err.message}]`);
    }
  }

  handleExampleSelect = async (event: React.ChangeEvent<HTMLSelectElement>) => {
    await this.loadExample(event.target.value);
  }

  handleProgramChange = (program: string) => {
    this.setState({ program })
  }

  runCommand = () => {
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

  handleRunClick = (event: React.SyntheticEvent) => {
    event.preventDefault();
    this.runCommand();
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
            <div className="field">
              <label className="label">Program</label>
              <div className="control">
                <AceEditor
                  name="editor"
                  mode="ocaml"
                  theme="xcode"
                  fontSize="1rem"
                  focus={true}
                  showPrintMargin={false}
                  width="100%"
                  minLines={EDITOR_ROWS}
                  maxLines={EDITOR_ROWS}
                  value={state.program}
                  onChange={this.handleProgramChange}
                  commands={[{
                    name: 'Run program',
                    bindKey: { win: 'Ctrl-Enter', mac: 'Command-Enter' },
                    exec: this.runCommand,
                  }]}
                  setOptions={{
                    useSoftTabs: true,
                    newLineMode: "unix" as unknown as boolean,
                  }}
                />
              </div>
            </div>
            <div className="columns">
              <div className="column is-2">
                <div className="field">
                  <label className="label">Example</label>
                  <div className="control select is-fullwidth">
                    <select onChange={this.handleExampleSelect}>
                      {
                        state.examples.map(({ name, file }) => (<option key={file} value={file}>{name}</option>))
                      }
                    </select>
                  </div>
                </div>
              </div>
              <div className="column is-8">
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
              <div className="column is-2">
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
            </div>
          </div>
        </section>
        <footer className="footer">
          <div className="content has-text-centered">
            © 2019–2021 <a href="https://github.com/hurryabit/rufus" target="blank">Martin Huschenbett</a>
          </div>
        </footer>
      </React.Fragment>
    );
  }
}

export default App;
