import React, { useEffect, useRef, useState } from 'react';
import './App.css';
import AceEditor from 'react-ace';

import type { Problem } from 'rufus-wasm';
import ProblemsPane from './ProblemsPane';

import "ace-builds/src-noconflict/mode-ocaml";
import "ace-builds/src-noconflict/mode-plain_text";
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
  problems: Problem[];
  result: string;
  examples: Example[];
}

export default function App() {
  const [state, setState] = useState<State>({
    wasm: null,
    program: '',
    output: '',
    problems: [],
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

  function checkProgram() {
    // NOTE(MH): This is the other half of the workaround mentioned above.
    const state = stateRef.current;
    const wasm = state.wasm;
    if (!wasm) {
      alert("WASM not loaded!");
      return;
    }
    const result = wasm.exec(state.program);
    const { output, problems } = result; // We need to read from result before calling free().
    setState(function (state) { return { ...state, output, problems }; });
    result.free();
  }

  function handleRunClick(event: React.SyntheticEvent) {
    event.preventDefault();
    checkProgram();
  }

  const annotations = [];
  const markers = [];
  for (const problem of state.problems) {
    const annotation = {
      row: problem.start.line - 1,
      col: problem.start.column - 1,
      text: `${problem.severity} [Col ${problem.start.column}]: ${problem.message} -- ${problem.source}`,
      type: problem.severity.toLowerCase(),
    };
    annotations.push(annotation);
    let className = "marker";
    switch (problem.severity) {
      case "ERROR":
        className += " error";
        break;
    }
    const marker = {
      startRow: problem.start.line - 1,
      startCol: problem.start.column - 1,
      endRow: problem.end.line - 1,
      endCol: problem.end.column - 1,
      className,
      type: "text" as const,
    };
    markers.push(marker);
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
          <div className="columns">
            <div className="column is-6">
              <div className="field">
                <label className="label">Program</label>
                <div className="control">
                  <AceEditor
                    name="editor"
                    value={state.program}
                    mode="ocaml"
                    theme="xcode"
                    fontSize="1rem"
                    focus={true}
                    showPrintMargin={false}
                    width="100%"
                    minLines={EDITOR_ROWS}
                    maxLines={EDITOR_ROWS}
                    onChange={handleProgramChange}
                    commands={[{
                      name: 'Check program',
                      bindKey: { win: 'Ctrl-S', mac: 'Command-S' },
                      exec: checkProgram,
                    }]}
                    setOptions={{
                      useSoftTabs: true,
                      newLineMode: "unix",
                    }}
                    annotations={annotations}
                    markers={markers}
                  />
                </div>
              </div>
            </div>
            <div className="column is-6">
              <div className="field">
                <label className="label">Output</label>
                <div className="control">
                  <AceEditor
                    name="output"
                    value={state.output}
                    readOnly
                    mode="plain_text"
                    fontSize="1rem"
                    showPrintMargin={false}
                    width="100%"
                    minLines={EDITOR_ROWS}
                    maxLines={EDITOR_ROWS}
                    setOptions={{
                      highlightActiveLine: false,
                      highlightGutterLine: false,
                    }}
                  />
                </div>
              </div>
            </div>
          </div>
          <div className="field">
            <label className="label">Problems</label>
            <div className="control">
              <ProblemsPane problems={state.problems} />
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
      <footer className="footer">
        <div className="content has-text-centered">
          © 2019–2024 <a href="https://github.com/hurryabit/rufus" target="blank">Martin Huschenbett</a>
        </div>
      </footer>
    </>
  );
}
