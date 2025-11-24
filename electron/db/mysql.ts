import mysql from "mysql2/promise";

export interface MysqlConnectConfig {
    host: string;
    port: number;
    user: string;
    password?: string;
    database?: string;
}

export interface SchemaNode {
    key: string;
    label: string;
    icon?: string;
    children?: SchemaNode[];
}

let connection: mysql.Connection | null = null;
let cachedConfig: MysqlConnectConfig | null = null;

export async function connect(config: MysqlConnectConfig) {
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

export async function query(sql: string) {
    if (!connection) {
        throw new Error("MySQL not connected");
    }

    const [rows, fields] = await connection.query(sql);
    return {
        rows,
        fields: Array.isArray(fields) ? fields.map((f: any) => f.name) : [],
    };
}

export async function structure(): Promise<SchemaNode[]> {
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
    const tableNodes: SchemaNode[] = tableRows.map((row: any, index) => {
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
