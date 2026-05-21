import type { Interpreter } from "./interpreter";

export type ExprLit<T> = {
	type: "Lit";
	value: T;
};

export function interpret_expr_lit<T>(
	expr: ExprLit<T>,
	_interpreter: Interpreter,
): T {
	return expr.value;
}
