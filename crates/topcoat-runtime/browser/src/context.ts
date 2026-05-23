import type { WriteSignal } from "@maverick-js/signals";

import type { SignalId, SignalRegistry } from "./signal";

/**
 * The `__context` object passed into every compiled expression. It is the only
 * way generated code can reach back into the runtime — keeping the surface
 * narrow makes the generated JS easy to audit and keeps non-context globals
 * inaccessible from inside `new Function`.
 */
export class Context {
	constructor(private readonly registry: SignalRegistry) {}

	signal(id: SignalId): WriteSignal<unknown> {
		return this.registry.handle(id);
	}
}
