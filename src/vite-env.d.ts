/// <reference types="vite/client" />

import type {
	ConnectPayload,
	PersistConnectionPayload,
	SavedConnectionSummary,
	SchemaNode,
} from './types/database';

declare module 'monaco-editor/esm/vs/editor/editor.worker?worker' {
	const EditorWorkerFactory: new () => Worker;
	export default EditorWorkerFactory;
}

declare module 'monaco-editor/esm/vs/language/json/json.worker?worker' {
	const JsonWorkerFactory: new () => Worker;
	export default JsonWorkerFactory;
}

declare module 'monaco-editor/esm/vs/language/css/css.worker?worker' {
	const CssWorkerFactory: new () => Worker;
	export default CssWorkerFactory;
}

declare module 'monaco-editor/esm/vs/language/html/html.worker?worker' {
	const HtmlWorkerFactory: new () => Worker;
	export default HtmlWorkerFactory;
}

declare module 'monaco-editor/esm/vs/language/typescript/ts.worker?worker' {
	const TsWorkerFactory: new () => Worker;
	export default TsWorkerFactory;
}

interface NebulaQueryResult {
	rows: any[];
	fields: string[];
	error?: string;
}

interface NebulaConnectResponse {
	success?: boolean;
	error?: string;
}

interface NebulaConfigBridge {
	saveConnection(config: PersistConnectionPayload): Promise<{
		success?: boolean;
		error?: string;
		connection?: SavedConnectionSummary;
	}>;
	listConnections(): Promise<{ connections?: SavedConnectionSummary[]; error?: string }>;
	getLastConnection(): Promise<{ id?: string; error?: string }>;
	connectSaved(id: string): Promise<NebulaConnectResponse>;
}

interface NebulaDatabaseBridge {
	connect(config: ConnectPayload): Promise<NebulaConnectResponse>;
	query(sql: string): Promise<NebulaQueryResult | { error: string }>;
	structure(): Promise<{ nodes?: SchemaNode[]; error?: string }>;
	disconnect(): Promise<{ success?: boolean; error?: string }>;
}

declare global {
	interface Window {
		db?: NebulaDatabaseBridge;
		config?: NebulaConfigBridge;
	}
}
