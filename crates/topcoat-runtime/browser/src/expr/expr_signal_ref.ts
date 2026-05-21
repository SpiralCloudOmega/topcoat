import type { SignalId } from "../signal";
import type { Interpreter } from "./interpreter";

export type ExprSignalRef = {
	type: "SignalRef";
	id: SignalId;
};

export function interpret_expr_signal_ref(
	expr: ExprSignalRef,
	interpreter: Interpreter,
): unknown {
	return interpreter.readSignal(expr.id);
}
