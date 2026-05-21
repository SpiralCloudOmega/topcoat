import { type ExprLit, interpret_expr_lit } from "./expr_lit";
import {
	type ExprSignalRef,
	interpret_expr_signal_ref,
} from "./expr_signal_ref";
import type { Interpreter } from "./interpreter";

export { Interpreter } from "./interpreter";

export type Expr = ExprLit<unknown> | ExprSignalRef;

export function interpret(expr: Expr, interpreter: Interpreter): unknown {
	switch (expr.type) {
		case "Lit":
			return interpret_expr_lit(expr, interpreter);
		case "SignalRef":
			return interpret_expr_signal_ref(expr, interpreter);
	}
}
