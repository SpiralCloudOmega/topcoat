import { Context } from "./context";
import { type SignalId, SignalRegistry } from "./signal";

export type ReactiveScopeId = string;

export type CommentMarker =
	| { kind: "signal"; id: SignalId; value: unknown }
	| { kind: "expr-start"; js: string }
	| { kind: "expr-end" }
	| {
			kind: "scope-start";
			id: ReactiveScopeId;
			track: SignalId[];
			path: string;
	  }
	| { kind: "scope-end"; id: ReactiveScopeId };

const SIGNAL_RE = /^\s*::topcoat::signal\("([^"]*)", "([^"]*)"\)\s*$/;
const EXPR_START_RE = /^\s*::topcoat::expr::start\("([^"]*)"\)\s*$/;
const EXPR_END_RE = /^\s*::topcoat::expr::end\s*$/;
const SCOPE_START_RE =
	/^\s*::topcoat::scope::start\(("[^"]+"), (\[[^\]]*\]), ("[^"]*")\)\s*$/;
const SCOPE_END_RE = /^\s*::topcoat::scope::end\(("[^"]+")\)\s*$/;

export function parseComment(node: Comment): CommentMarker | null {
	const text = node.data;

	const sig = SIGNAL_RE.exec(text);
	if (sig) {
		const id = sig[1];
		const valueExpr = new DOMParser().parseFromString(sig[2], "text/html")
			.documentElement.textContent;
		const value = new Function("cx", `return ${valueExpr};`)(
			new Context(new SignalRegistry()),
		);
		return {
			kind: "signal",
			id,
			value,
		};
	}

	const exprStart = EXPR_START_RE.exec(text);
	if (exprStart) {
		const js = new DOMParser().parseFromString(exprStart[1], "text/html")
			.documentElement.textContent;
		if (js === null) throw new Error("Failed to decode expression marker");
		return {
			kind: "expr-start",
			js,
		};
	}

	if (EXPR_END_RE.test(text)) {
		return { kind: "expr-end" };
	}

	const start = SCOPE_START_RE.exec(text);
	if (start) {
		return {
			kind: "scope-start",
			id: JSON.parse(start[1]) as ReactiveScopeId,
			track: JSON.parse(start[2]) as SignalId[],
			path: JSON.parse(start[3]) as string,
		};
	}

	const end = SCOPE_END_RE.exec(text);
	if (end) {
		return { kind: "scope-end", id: JSON.parse(end[1]) as ReactiveScopeId };
	}

	return null;
}
