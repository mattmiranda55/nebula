import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import type { ConnectConfig, DatabaseType } from "./db/index.js";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const CONFIG_PATH = path.join(__dirname, "..", "config.toml");
const ENC_PREFIX = "enc:";
const METADATA_SECTION = "nebula";
const LAST_CONNECTION_KEY = "last_connection";

type Section = {
    name: string;
    entries: Record<string, string | number>;
};

type StoredConnection = ConnectConfig & {
    id?: string;
    name?: string;
    password?: string;
};

export interface StoredConnectionSummary {
    id: string;
    name?: string;
    type: DatabaseType;
    host?: string;
    port?: number;
    user?: string;
    database?: string;
    file?: string;
    uri?: string;
}

function ensureKey(): Buffer {
    const key = process.env["NEBULA_CONFIG_KEY"];
    if (!key) {
        throw new Error("Set NEBULA_CONFIG_KEY in your environment to encrypt/decrypt passwords.");
    }
    return crypto.createHash("sha256").update(key).digest();
}

function encrypt(secret: string): string {
    const key = ensureKey();
    const iv = crypto.randomBytes(12);
    const cipher = crypto.createCipheriv("aes-256-gcm", key, iv);
    const encrypted = Buffer.concat([cipher.update(secret, "utf8"), cipher.final()]);
    const tag = cipher.getAuthTag();
    return `${ENC_PREFIX}${iv.toString("hex")}:${tag.toString("hex")}:${encrypted.toString("hex")}`;
}

function decrypt(value?: string): string | undefined {
    if (!value) return undefined;
    if (!value.startsWith(ENC_PREFIX)) return value;
    const payload = value.slice(ENC_PREFIX.length);
    const [ivHex, tagHex, dataHex] = payload.split(":");
    if (!ivHex || !tagHex || !dataHex) {
        throw new Error("Malformed encrypted secret in config.toml");
    }
    const key = ensureKey();
    const iv = Buffer.from(ivHex, "hex");
    const tag = Buffer.from(tagHex, "hex");
    const data = Buffer.from(dataHex, "hex");
    const decipher = crypto.createDecipheriv("aes-256-gcm", key, iv);
    decipher.setAuthTag(tag);
    const decrypted = Buffer.concat([decipher.update(data), decipher.final()]);
    return decrypted.toString("utf8");
}

function parseToml(content: string): Section[] {
    const lines = content.split(/\r?\n/);
    const sections: Section[] = [];
    let current: Section | null = null;
    let parent: string | null = null;

    for (const raw of lines) {
        const trimmed = raw.trim();
        if (!trimmed || trimmed.startsWith("#")) continue;

        const sectionMatch = trimmed.match(/^\[([^\]]+)\]$/);
        if (sectionMatch) {
            if (current) sections.push(current);
            let name = sectionMatch[1];

            if (name === METADATA_SECTION) {
                parent = null;
            } else if (name === "postgres" || name === "mysql" || name === "sqlite" || name === "mongodb") {
                parent = name;
            } else if (parent && !name.includes(".")) {
                name = `${parent}.${name}`;
                parent = null;
            } else {
                parent = null;
            }

            current = { name, entries: {} };
            continue;
        }

        if (!current) continue;
        const kvMatch = trimmed.match(/^([A-Za-z0-9_.-]+)\s*=\s*(.+)$/);
        if (!kvMatch) continue;

        const [, key, rawValue] = kvMatch;
        const clean = rawValue.trim();
        let value: string | number = clean;
        if (
            (clean.startsWith('"') && clean.endsWith('"')) ||
            (clean.startsWith("'") && clean.endsWith("'"))
        ) {
            value = clean.slice(1, -1).replace(/\\"/g, '"');
        } else if (!Number.isNaN(Number(clean))) {
            value = Number(clean);
        }

        current.entries[key] = value;
    }

    if (current) sections.push(current);
    return sections;
}

function dedupeSections(sections: Section[]): Section[] {
    const seen = new Set<string>();
    const result: Section[] = [];
    for (let i = sections.length - 1; i >= 0; i--) {
        const section = sections[i];
        if (seen.has(section.name)) continue;
        seen.add(section.name);
        result.unshift(section);
    }
    return result;
}

function serializeToml(sections: Section[]): string {
    const lines: string[] = [];

    for (const section of sections) {
        if (section.name === METADATA_SECTION) {
            lines.push(`[${section.name}]`);
            for (const [key, value] of Object.entries(section.entries)) {
                lines.push(`${key} = ${typeof value === "number" ? value : `"${value}"`}`);
            }
            lines.push("");
            continue;
        }

        const parts = section.name.split(".");
        if (parts.length > 1) {
            const [parent, child] = [parts[0], parts.slice(1).join(".")];
            lines.push(`[${parent}]`);
            lines.push(`    [${child}]`);
            for (const [key, value] of Object.entries(section.entries)) {
                lines.push(`    ${key} = ${typeof value === "number" ? value : `"${value}"`}`);
            }
        } else {
            lines.push(`[${section.name}]`);
            for (const [key, value] of Object.entries(section.entries)) {
                lines.push(`${key} = ${typeof value === "number" ? value : `"${value}"`}`);
            }
        }
        lines.push("");
    }

    return lines.join("\n").replace(/\n+$/, "\n");
}

function defaultSection(type: DatabaseType): string {
    if (type === "postgres") return "postgres.connection1";
    return type;
}

function mapSectionToConnection(section: Section): StoredConnection | null {
    if (section.name === "mysql") {
        const password = decrypt(section.entries["password"] as string | undefined);
        return {
            id: section.name,
            type: "mysql",
            name: (section.entries["name"] as string | undefined) || undefined,
            host: (section.entries["host"] as string) || "",
            port: Number(section.entries["port"] ?? 3306),
            user: (section.entries["user"] as string) || (section.entries["username"] as string) || "",
            password,
            database: (section.entries["database"] as string | undefined) || undefined,
        };
    }

    if (section.name.startsWith("postgres")) {
        const password = decrypt(section.entries["password"] as string | undefined);
        return {
            id: section.name,
            type: "postgres",
            name: (section.entries["name"] as string | undefined) || undefined,
            host: (section.entries["host"] as string) || "",
            port: Number(section.entries["port"] ?? 5432),
            user: (section.entries["user"] as string) || (section.entries["username"] as string) || "",
            password,
            database: (section.entries["database"] as string) || "",
        };
    }

    if (section.name === "sqlite") {
        return {
            id: section.name,
            type: "sqlite",
            name: (section.entries["name"] as string | undefined) || undefined,
            file: (section.entries["file"] as string) || "",
        };
    }

    if (section.name === "mongodb") {
        return {
            id: section.name,
            type: "mongodb",
            name: (section.entries["name"] as string | undefined) || undefined,
            uri:
                (section.entries["uri"] as string) ||
                (section.entries["host"] as string) ||
                "",
            database: (section.entries["database"] as string | undefined) || undefined,
        };
    }

    return null;
}

function readSections(): Section[] {
    if (!fs.existsSync(CONFIG_PATH)) return [];
    const content = fs.readFileSync(CONFIG_PATH, "utf8");
    const parsed = parseToml(content);
    const normalized = dedupeSections(parsed);
    // Rewrite if normalized differs in section count to clean up duplicates.
    if (normalized.length !== parsed.length) {
        writeSections(normalized);
    }
    return normalized;
}

function writeSections(sections: Section[]) {
    const serialized = serializeToml(sections);
    fs.writeFileSync(CONFIG_PATH, serialized, "utf8");
}

function getLastConnectionIdFromSections(sections: Section[]): string | undefined {
    const meta = sections.find((section) => section.name === METADATA_SECTION);
    const value = meta?.entries?.[LAST_CONNECTION_KEY];
    return typeof value === "string" && value.trim() ? value.trim() : undefined;
}

function setLastConnectionId(sections: Section[], id: string) {
    let meta = sections.find((section) => section.name === METADATA_SECTION);
    if (!meta) {
        meta = { name: METADATA_SECTION, entries: {} };
        sections.push(meta);
    }
    meta.entries[LAST_CONNECTION_KEY] = id;
}

export function listConnections(): StoredConnectionSummary[] {
    return readSections()
        .map(mapSectionToConnection)
        .filter(Boolean)
        .map((conn) => ({
            id: conn!.id || defaultSection(conn!.type),
            name: conn?.name,
            type: conn!.type,
            host: (conn as any).host,
            port: (conn as any).port,
            user: (conn as any).user,
            database: (conn as any).database,
            file: (conn as any).file,
            uri: (conn as any).uri,
        })) as StoredConnectionSummary[];
}

export function loadConnection(id: string): StoredConnection | null {
    const match = readSections()
        .map(mapSectionToConnection)
        .find((conn) => conn && (conn.id === id || defaultSection(conn.type) === id));
    return (match as StoredConnection) || null;
}

export function saveConnection(profile: StoredConnection): StoredConnectionSummary {
    const sections = readSections();
    const sectionName = profile.id || defaultSection(profile.type);
    const password = profile.password ? encrypt(profile.password) : undefined;

    const entries: Record<string, string | number> = {
        name: profile.name || "",
    };

    if (profile.type === "mysql") {
        const mysqlProfile = profile as Extract<ConnectConfig, { type: "mysql" }>;
        entries["host"] = mysqlProfile.host;
        entries["port"] = mysqlProfile.port;
        entries["username"] = mysqlProfile.user;
        if (password) entries["password"] = password;
        if (mysqlProfile.database) entries["database"] = mysqlProfile.database;
    } else if (profile.type === "postgres") {
        const pgProfile = profile as Extract<ConnectConfig, { type: "postgres" }>;
        entries["host"] = pgProfile.host;
        entries["port"] = pgProfile.port;
        entries["username"] = pgProfile.user;
        if (password) entries["password"] = password;
        entries["database"] = pgProfile.database;
    } else if (profile.type === "sqlite") {
        const sqliteProfile = profile as Extract<ConnectConfig, { type: "sqlite" }>;
        entries["file"] = sqliteProfile.file;
    } else if (profile.type === "mongodb") {
        const mongoProfile = profile as Extract<ConnectConfig, { type: "mongodb" }>;
        entries["uri"] = (mongoProfile as any).uri;
        entries["host"] = (mongoProfile as any).uri;
        if ((mongoProfile as any).database) entries["database"] = (mongoProfile as any).database;
    }

    const idx = sections.findIndex((section) => section.name === sectionName);
    const nextSection: Section = { name: sectionName, entries };
    if (idx >= 0) {
        sections[idx] = nextSection;
    } else {
        sections.push(nextSection);
    }

    setLastConnectionId(sections, sectionName);
    writeSections(sections);

    return {
        id: sectionName,
        name: profile.name,
        type: profile.type,
        host: (profile as any).host,
        port: (profile as any).port,
        user: (profile as any).user,
        database: (profile as any).database,
        file: (profile as any).file,
        uri: (profile as any).uri,
    };
}

export function getLastConnectionId(): string | undefined {
    const sections = readSections();
    return getLastConnectionIdFromSections(sections);
}
