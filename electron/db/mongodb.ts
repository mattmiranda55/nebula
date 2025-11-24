import { MongoClient } from "mongodb";

export interface MongoConnectConfig {
	uri: string;
	database?: string;
}

export interface SchemaNode {
	key: string;
	label: string;
	icon?: string;
	children?: SchemaNode[];
}

let client: MongoClient | null = null;
let cachedConfig: MongoConnectConfig | null = null;

export async function connect(config: MongoConnectConfig) {
	if (!config.uri) {
		throw new Error("MongoDB connection requires a connection string (URI).");
	}

	client = new MongoClient(config.uri);
	await client.connect();
	cachedConfig = config;

	return { success: true };
}

export async function query(_query: string) {
	throw new Error("MongoDB query execution is not yet implemented.");
}

export async function structure(): Promise<SchemaNode[]> {
	if (!client) {
		throw new Error("MongoDB not connected");
	}

	const databasesToInspect: { name: string }[] = [];

	if (cachedConfig?.database) {
		databasesToInspect.push({ name: cachedConfig.database });
	} else {
		const admin = client.db().admin();
		const list = await admin.listDatabases();
		databasesToInspect.push(...list.databases);
	}

	const databaseNodes: SchemaNode[] = [];

	for (const dbInfo of databasesToInspect) {
		const db = client.db(dbInfo.name);
		const collections = await db.listCollections({}, { nameOnly: true }).toArray();
		const collectionNodes: SchemaNode[] = collections.map((collection, index) => ({
			key: `mongo-${dbInfo.name}-collection-${index}`,
			label: collection.name,
			icon: "pi pi-folder-open",
		}));

		databaseNodes.push({
			key: `mongo-db-${dbInfo.name}`,
			label: dbInfo.name,
			icon: "pi pi-database",
			children: collectionNodes,
		});
	}

	return databaseNodes.length > 0
		? databaseNodes
		: [
			{
				key: "mongo-empty",
				label: "No databases found",
				icon: "pi pi-info-circle",
				children: [],
			},
		];
}

export async function disconnect() {
	if (client) {
		await client.close();
		client = null;
		cachedConfig = null;
	}
}
