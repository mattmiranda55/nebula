import * as mysql from "./mysql.js";
import * as postgres from "./postgres.js";
import * as sqlite from "./sqlite.js";
import * as mongodb from "./mongodb.js";
let activeDriver = null;
function resolveDriver(type) {
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
export async function connect(config) {
    if (activeDriver) {
        await activeDriver.disconnect();
    }
    const driver = resolveDriver(config.type);
    const result = await driver.connect(config);
    activeDriver = driver;
    return result;
}
export async function query(sql) {
    if (!activeDriver) {
        throw new Error("No active database connection.");
    }
    return await activeDriver.query(sql);
}
export async function structure() {
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
}
//# sourceMappingURL=index.js.map