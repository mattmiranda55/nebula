import * as mysql from "./mysql";

type DB = "mysql" | "postgres" | "sqlite";

let driver : any = null;

export async function connect(config: any) {
    switch (config.type as DB) {
        case "mysql":
            driver = mysql;
            break;
        case "postgres":
            // driver = postgres;
            throw new Error("Postgres support not implemented yet.");
        case "sqlite":
            // driver = sqlite;
            throw new Error("SQLite support not implemented yet.");
        default:
            throw new Error("Unknown database type.");
    }

    return await driver.connect(config);
}

export async function query(sql: string){
    if(!driver) {
        throw new Error("No active database connection.");
    }
    return await driver.query(sql);
}

export async function disconnect(){
    if(!driver) return;
    await driver.disconnect();
    driver = null;
}