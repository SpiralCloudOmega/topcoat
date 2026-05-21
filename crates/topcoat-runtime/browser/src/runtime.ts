import { Interpreter } from "./expr";
import { scan } from "./scan";
import { Scope } from "./scope";
import { SignalRegistry } from "./signal";

export class Runtime {
	readonly registry = new SignalRegistry();
	readonly interpreter = new Interpreter(this.registry);
	readonly rootScope: Scope = new Scope(null, this);

	start(root: ParentNode): void {
		scan(root, null, null, this.rootScope);
	}
}
