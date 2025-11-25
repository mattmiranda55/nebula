import Database from "better-sqlite3";
import path from "node:path";
let db = null;
let cachedConfig = null;
export async function connect(config) {
    db = new Database(config.file, { fileMustExist: false });
    cachedConfig = config;
    return { success: true };
}
export async function query(sql) {
    if (!db) {
        throw new Error("SQLite not connected");
    }
    const statement = db.prepare(sql);
    if (statement.reader) {
        const rows = statement.all();
        const columnDefinitions = typeof statement.columns === "function" ? statement.columns() : [];
        const columns = columnDefinitions.map((column) => column.name);
        return {
            rows,
            fields: columns,
        };
    }
    const info = statement.run();
    return {
        rows: [
            {
                changes: info.changes,
                lastInsertRowid: info.lastInsertRowid,
            },
        ],
        fields: ["changes", "lastInsertRowid"],
    };
}
export async function structure() {
    if (!db) {
        throw new Error("SQLite not connected");
    }
    const rows = db
        .prepare(`SELECT name, type
			 FROM sqlite_master
			 WHERE type IN ('table', 'view')
			   AND name NOT LIKE 'sqlite_%'
			 ORDER BY name;`)
        .all();
    const tableNodes = rows.map((row, index) => ({
        key: `sqlite-${row.type}-${index}`,
        label: row.name,
        icon: row.type === "view" ? "pi pi-eye" : "pi pi-table",
    }));
    const displayName = cachedConfig?.file ? path.basename(cachedConfig.file) : "SQLite";
    return [
        {
            key: `sqlite-db-${displayName}`,
            label: displayName,
            icon: "pi pi-database",
            children: tableNodes,
        },
    ];
}
export async function disconnect() {
    if (db) {
        db.close();
        db = null;
        cachedConfig = null;
    }
}
//# sourceMappingURL=sqlite.js.map