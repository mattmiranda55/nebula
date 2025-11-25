import pg from "pg";
const { Client } = pg;
let client = null;
let cachedConfig = null;
export async function connect(config) {
    client = new Client({
        host: config.host,
        port: config.port,
        user: config.user,
        password: config.password,
        database: config.database,
    });
    await client.connect();
    cachedConfig = config;
    return { success: true };
}
export async function query(sql) {
    if (!client) {
        throw new Error("PostgreSQL not connected");
    }
    const result = await client.query(sql);
    return {
        rows: result.rows,
        fields: result.fields?.map((field) => field.name) ?? [],
    };
}
export async function structure() {
    if (!client) {
        throw new Error("PostgreSQL not connected");
    }
    const databaseName = cachedConfig?.database ?? "postgres";
    const result = await client.query(`SELECT table_schema, table_name
		 FROM information_schema.tables
		 WHERE table_type = 'BASE TABLE'
		   AND table_schema NOT IN ('pg_catalog', 'information_schema')
		 ORDER BY table_schema, table_name;`);
    const schemaMap = new Map();
    for (const row of result.rows) {
        const schemaName = row.table_schema;
        const tableName = row.table_name;
        if (!schemaMap.has(schemaName)) {
            schemaMap.set(schemaName, []);
        }
        schemaMap.get(schemaName).push({
            key: `pg-${schemaName}-${tableName}`,
            label: tableName,
            icon: "pi pi-table",
        });
    }
    const schemaNodes = Array.from(schemaMap.entries()).map(([schemaName, tables]) => ({
        key: `pg-schema-${schemaName}`,
        label: schemaName,
        icon: "pi pi-folder",
        children: tables,
    }));
    return [
        {
            key: `pg-db-${databaseName}`,
            label: databaseName,
            icon: "pi pi-database",
            children: schemaNodes,
        },
    ];
}
export async function disconnect() {
    if (client) {
        await client.end();
        client = null;
        cachedConfig = null;
    }
}
//# sourceMappingURL=postgres.js.map