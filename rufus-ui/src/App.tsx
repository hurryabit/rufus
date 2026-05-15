import React, { useEffect, useRef, useState } from 'react';
import './App.css';
import AceEditor from 'react-ace';

import "ace-builds/src-noconflict/mode-ocaml";
import "ace-builds/src-noconflict/theme-xcode";

const EXAMPLES_DIR: string = '/rufus/examples';

const EDITOR_ROWS: number = 20;

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

export default function App() {
  const [helpOpen, setHelpOpen] = useState(false);
  const [state, setState] = useState<State>({
    wasm: null,
    program: '',
    output: '',
    result: '',
    examples: [],
  });
  // NOTE(MH): This is a workaround for https://github.com/securingsincity/react-ace/issues/684.
  const stateRef = useRef(state);
  stateRef.current = state;

  async function loadExample(example_file: string) {
    try {
      const response = await fetch(`${EXAMPLES_DIR}/${example_file}`);
      const program = await response.text();
      setState(function (state) { return { ...state, program: program }; });
    } catch (err) {
      console.error(`Unexpected error in loadExample. [Message: ${err}]`);
      alert('Cannot load example. See console for details.');
    }
  }

  useEffect(function () {
    async function loadExamples() {
      try {
        const response = await fetch(`${EXAMPLES_DIR}/index.json`);
        const examples: Example[] = await response.json();
        setState(function (state) { return { ...state, examples }; });
        if (examples.length > 0) {
          loadExample(examples[0].file);
        }
      } catch (err) {
        console.error(`Unexptected error in loadExamples. [Message: ${err}]`);
      }
    }

    async function loadWasm() {
      try {
        const wasm = await import('rufus-wasm');
        setState(function (state) { return { ...state, wasm }; })
      } catch (err) {
        console.error(`Unexpected error in loadWasm. [Message: ${err}]`);
      }
    }

    loadExamples();
    loadWasm();
  }, []);


  function handleExampleSelect(event: React.ChangeEvent<HTMLSelectElement>) {
    loadExample(event.target.value);
  }

  function handleProgramChange(program: string) {
    setState(function (state) { return { ...state, program }; })
  }

  function runCommand() {
    // NOTE(MH): This is the other half of the workaround mentioned above.
    const state = stateRef.current;
    const wasm = state.wasm;
    if (!wasm) {
      alert("WASM not loaded!");
      return;
    }
    const result = wasm.exec(state.program);
    const status = result.status;
    const value = result.get_value();
    switch (status) {
      case wasm.ExecResultStatus.Ok:
        setState(function (state) { return { ...state, result: value }; });
        break;
      case wasm.ExecResultStatus.Err:
        alert(value);
        break;
    }
  }

  function handleRunClick(event: React.SyntheticEvent) {
    event.preventDefault();
    runCommand();
  }


  return (
    <>
      <section className="hero is-link">
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
            <div className="control" style={{ position: 'relative' }}>
              <button
                className="button is-small is-rounded"
                style={{ position: 'absolute', top: '0.4rem', right: '0.4rem', zIndex: 10, width: '1.75rem', height: '1.75rem', padding: 0, lineHeight: 1 }}
                title="Rufus language help"
                onClick={() => setHelpOpen(true)}
              >
                ?
              </button>
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
                onChange={handleProgramChange}
                commands={[{
                  name: 'Run program',
                  bindKey: { win: 'Ctrl-Enter', mac: 'Command-Enter' },
                  exec: runCommand,
                }]}
                setOptions={{
                  useSoftTabs: true,
                  newLineMode: "unix",
                }}
              />
            </div>
          </div>
          <div className="columns">
            <div className="column is-2">
              <div className="field">
                <label className="label">Example</label>
                <div className="control select is-fullwidth">
                  <select onChange={handleExampleSelect}>
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
                    className="button is-fullwidth is-link"
                    onClick={handleRunClick}
                  >
                    Run
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>
      <div className={`modal${helpOpen ? ' is-active' : ''}`}>
        <div className="modal-background" onClick={() => setHelpOpen(false)} />
        <div className="modal-card">
          <header className="modal-card-head">
            <p className="modal-card-title">Rufus Language Reference</p>
            <button className="delete" onClick={() => setHelpOpen(false)} />
          </header>
          <section className="modal-card-body content">
            <p>Rufus is a small functional expression language. A program is a single expression that evaluates to a value.</p>
            <h4>Literals</h4>
            <ul>
              <li><code>42</code>, <code>-7</code> — integers</li>
              <li><code>true</code>, <code>false</code> — booleans</li>
            </ul>
            <h4>Operators</h4>
            <ul>
              <li>Arithmetic: <code>+</code>, <code>-</code>, <code>*</code>, <code>/</code></li>
              <li>Comparison: <code>==</code>, <code>!=</code>, <code>&lt;</code>, <code>&lt;=</code>, <code>&gt;</code>, <code>&gt;=</code></li>
            </ul>
            <h4>Functions</h4>
            <p><code>fun x y -&gt; body</code> — anonymous function with one or more parameters</p>
            <p><code>f x y</code> — function application (juxtaposition)</p>
            <h4>Let bindings</h4>
            <p><code>let x = expr in body</code> — bind a name</p>
            <p><code>let rec f = fun args -&gt; body in expr</code> — recursive binding</p>
            <h4>Conditionals</h4>
            <p><code>if cond then a else b</code></p>
            <h4>Records</h4>
            <p><code>{'{ field1 = expr1; field2 = expr2 }'}</code> — create a record</p>
            <p><code>record.field</code> — field access</p>
            <h4>Comments</h4>
            <p><code>{'(* this is a comment *)'}</code></p>
          </section>
          <footer className="modal-card-foot" style={{ justifyContent: 'flex-end' }}>
            <button className="button" onClick={() => setHelpOpen(false)}>Close</button>
          </footer>
        </div>
      </div>
      <footer className="footer">
        <div className="content has-text-centered">
          © 2019–2024 <a href="https://github.com/hurryabit/rufus" target="blank">Martin Huschenbett</a>
        </div>
      </footer>
    </>
  );
}
