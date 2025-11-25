import { MongoClient } from "mongodb";
let client = null;
let cachedConfig = null;
export async function connect(config) {
    if (!config.uri) {
        throw new Error("MongoDB connection requires a connection string (URI).");
    }
    client = new MongoClient(config.uri);
    await client.connect();
    cachedConfig = config;
    return { success: true };
}
export async function query(_query) {
    throw new Error("MongoDB query execution is not yet implemented.");
}
export async function structure() {
    if (!client) {
        throw new Error("MongoDB not connected");
    }
    const databasesToInspect = [];
    if (cachedConfig?.database) {
        databasesToInspect.push({ name: cachedConfig.database });
    }
    else {
        const admin = client.db().admin();
        const list = await admin.listDatabases();
        databasesToInspect.push(...list.databases);
    }
    const databaseNodes = [];
    for (const dbInfo of databasesToInspect) {
        const db = client.db(dbInfo.name);
        const collections = await db.listCollections({}, { nameOnly: true }).toArray();
        const collectionNodes = collections.map((collection, index) => ({
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
//# sourceMappingURL=mongodb.js.map