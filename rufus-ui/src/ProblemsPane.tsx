import type { Problem } from 'rufus-wasm';

type Props = {
    problems: Problem[];
}

export default function ProblemsPane({ problems }: Props) {
    const text = problems.map(function ({ start, severity, message, source }) {
        return `${severity} [Ln ${start.line + 1}, Col ${start.column + 1}]: ${message} -- ${source}`;
    }).join("\n");

    return <textarea
        className="textarea has-fixed-size is-family-code"
        readOnly
        rows={4}
        value={text}
    />;
}
