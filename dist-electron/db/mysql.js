import mysql from "mysql2/promise";
let connection = null;
let cachedConfig = null;
export async function connect(config) {
    connection = await mysql.createConnection({
        host: config.host,
        port: config.port,
        user: config.user,
        password: config.password,
        database: config.database,
    });
    cachedConfig = config;
    return { success: true };
}
export async function query(sql) {
    if (!connection) {
        throw new Error("MySQL not connected");
    }
    const [rows, fields] = await connection.query(sql);
    return {
        rows,
        fields: Array.isArray(fields) ? fields.map((f) => f.name) : [],
    };
}
export async function structure() {
    if (!connection) {
        throw new Error("MySQL not connected");
    }
    const database = cachedConfig?.database;
    if (!database) {
        return [
            {
                key: "mysql-empty",
                label: "No default database selected",
                icon: "pi pi-info-circle",
                children: [],
            },
        ];
    }
    const [tablesResult] = await connection.query(`SHOW TABLES`);
    const tableRows = Array.isArray(tablesResult) ? tablesResult : [];
    const tableNodes = tableRows.map((row, index) => {
        const tableName = row[`Tables_in_${database}`];
        return {
            key: `mysql-table-${index}`,
            label: tableName,
            icon: "pi pi-table",
        };
    });
    return [
        {
            key: `mysql-db-${database}`,
            label: database,
            icon: "pi pi-database",
            children: tableNodes,
        },
    ];
}
export async function disconnect() {
    if (connection) {
        await connection.end();
        connection = null;
        cachedConfig = null;
    }
}
//# sourceMappingURL=mysql.js.map