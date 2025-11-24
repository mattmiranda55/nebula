import * as mysql from "./mysql";
import * as postgres from "./postgres";
import * as sqlite from "./sqlite";
import * as mongodb from "./mongodb";

export type DatabaseType = "mysql" | "postgres" | "sqlite" | "mongodb";

export type ConnectConfig =
    | ({ type: "mysql" } & mysql.MysqlConnectConfig)
    | ({ type: "postgres" } & postgres.PostgresConnectConfig)
    | ({ type: "sqlite" } & sqlite.SqliteConnectConfig)
    | ({ type: "mongodb" } & mongodb.MongoConnectConfig);

export interface SchemaNode {
    key: string;
    label: string;
    icon?: string;
    children?: SchemaNode[];
}

interface DriverModule {
    connect(config: any): Promise<{ success?: boolean; error?: string }>;
    query(sql: string): Promise<{ rows: unknown[]; fields: string[] } | { error: string }>;
    structure(): Promise<SchemaNode[]>;
    disconnect(): Promise<void>;
}

let activeDriver: DriverModule | null = null;
let activeType: DatabaseType | null = null;

function resolveDriver(type: DatabaseType): DriverModule {
    switch (type) {
        case "mysql":
            return mysql;
        case "postgres":
            return postgres;
        case "sqlite":
            return sqlite;
        case "mongodb":
            return mongodb;
        default:
            throw new Error("Unknown database driver");
    }
}

export async function connect(config: ConnectConfig) {
    if (activeDriver) {
        await activeDriver.disconnect();
    }

    const driver = resolveDriver(config.type);
    const result = await driver.connect(config);
    activeDriver = driver;
    activeType = config.type;
    return result;
}

export async function query(sql: string) {
    if (!activeDriver) {
        throw new Error("No active database connection.");
    }

    return await activeDriver.query(sql);
}

export async function structure(): Promise<SchemaNode[]> {
    if (!activeDriver) {
        throw new Error("No active database connection.");
    }

    return await activeDriver.structure();
}

export async function disconnect() {
    if (!activeDriver) {
        return;
    }

    await activeDriver.disconnect();
    activeDriver = null;
    activeType = null;
}