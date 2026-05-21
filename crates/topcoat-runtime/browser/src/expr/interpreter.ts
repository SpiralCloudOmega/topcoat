import type { SignalId, SignalRegistry } from "../signal";

export class Interpreter {
	public constructor(private readonly registry: SignalRegistry) {}

	public readSignal(id: SignalId): unknown {
		return this.registry.read(id);
	}
}
