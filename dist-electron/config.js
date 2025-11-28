import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const CONFIG_PATH = path.join(__dirname, "..", "config.toml");
const ENC_PREFIX = "enc:";
const METADATA_SECTION = "nebula";
const LAST_CONNECTION_KEY = "last_connection";
function ensureKey() {
    const key = process.env["NEBULA_CONFIG_KEY"];
    if (!key) {
        throw new Error("Set NEBULA_CONFIG_KEY in your environment to encrypt/decrypt passwords.");
    }
    return crypto.createHash("sha256").update(key).digest();
}
function encrypt(secret) {
    const key = ensureKey();
    const iv = crypto.randomBytes(12);
    const cipher = crypto.createCipheriv("aes-256-gcm", key, iv);
    const encrypted = Buffer.concat([cipher.update(secret, "utf8"), cipher.final()]);
    const tag = cipher.getAuthTag();
    return `${ENC_PREFIX}${iv.toString("hex")}:${tag.toString("hex")}:${encrypted.toString("hex")}`;
}
function decrypt(value) {
    if (!value)
        return undefined;
    if (!value.startsWith(ENC_PREFIX))
        return value;
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
function parseToml(content) {
    const lines = content.split(/\r?\n/);
    const sections = [];
    let current = null;
    let parent = null;
    for (const raw of lines) {
        const trimmed = raw.trim();
        if (!trimmed || trimmed.startsWith("#"))
            continue;
        const sectionMatch = trimmed.match(/^\[([^\]]+)\]$/);
        if (sectionMatch) {
            if (current)
                sections.push(current);
            let name = sectionMatch[1];
            if (name === METADATA_SECTION) {
                parent = null;
            }
            else if (name === "postgres" || name === "mysql" || name === "sqlite" || name === "mongodb") {
                parent = name;
            }
            else if (parent && !name.includes(".")) {
                name = `${parent}.${name}`;
                parent = null;
            }
            else {
                parent = null;
            }
            current = { name, entries: {} };
            continue;
        }
        if (!current)
            continue;
        const kvMatch = trimmed.match(/^([A-Za-z0-9_.-]+)\s*=\s*(.+)$/);
        if (!kvMatch)
            continue;
        const [, key, rawValue] = kvMatch;
        const clean = rawValue.trim();
        let value = clean;
        if ((clean.startsWith('"') && clean.endsWith('"')) ||
            (clean.startsWith("'") && clean.endsWith("'"))) {
            value = clean.slice(1, -1).replace(/\\"/g, '"');
        }
        else if (!Number.isNaN(Number(clean))) {
            value = Number(clean);
        }
        current.entries[key] = value;
    }
    if (current)
        sections.push(current);
    return sections;
}
function dedupeSections(sections) {
    const seen = new Set();
    const result = [];
    for (let i = sections.length - 1; i >= 0; i--) {
        const section = sections[i];
        if (seen.has(section.name))
            continue;
        seen.add(section.name);
        result.unshift(section);
    }
    return result;
}
function serializeToml(sections) {
    const lines = [];
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
        }
        else {
            lines.push(`[${section.name}]`);
            for (const [key, value] of Object.entries(section.entries)) {
                lines.push(`${key} = ${typeof value === "number" ? value : `"${value}"`}`);
            }
        }
        lines.push("");
    }
    return lines.join("\n").replace(/\n+$/, "\n");
}
function defaultSection(type) {
    if (type === "postgres")
        return "postgres.connection1";
    return type;
}
function mapSectionToConnection(section) {
    if (section.name === "mysql") {
        const password = decrypt(section.entries["password"]);
        return {
            id: section.name,
            type: "mysql",
            name: section.entries["name"] || undefined,
            host: section.entries["host"] || "",
            port: Number(section.entries["port"] ?? 3306),
            user: section.entries["user"] || section.entries["username"] || "",
            password,
            database: section.entries["database"] || undefined,
        };
    }
    if (section.name.startsWith("postgres")) {
        const password = decrypt(section.entries["password"]);
        return {
            id: section.name,
            type: "postgres",
            name: section.entries["name"] || undefined,
            host: section.entries["host"] || "",
            port: Number(section.entries["port"] ?? 5432),
            user: section.entries["user"] || section.entries["username"] || "",
            password,
            database: section.entries["database"] || "",
        };
    }
    if (section.name === "sqlite") {
        return {
            id: section.name,
            type: "sqlite",
            name: section.entries["name"] || undefined,
            file: section.entries["file"] || "",
        };
    }
    if (section.name === "mongodb") {
        return {
            id: section.name,
            type: "mongodb",
            name: section.entries["name"] || undefined,
            uri: section.entries["uri"] ||
                section.entries["host"] ||
                "",
            database: section.entries["database"] || undefined,
        };
    }
    return null;
}
function readSections() {
    if (!fs.existsSync(CONFIG_PATH))
        return [];
    const content = fs.readFileSync(CONFIG_PATH, "utf8");
    const parsed = parseToml(content);
    const normalized = dedupeSections(parsed);
    // Rewrite if normalized differs in section count to clean up duplicates.
    if (normalized.length !== parsed.length) {
        writeSections(normalized);
    }
    return normalized;
}
function writeSections(sections) {
    const serialized = serializeToml(sections);
    fs.writeFileSync(CONFIG_PATH, serialized, "utf8");
}
function getLastConnectionIdFromSections(sections) {
    const meta = sections.find((section) => section.name === METADATA_SECTION);
    const value = meta?.entries?.[LAST_CONNECTION_KEY];
    return typeof value === "string" && value.trim() ? value.trim() : undefined;
}
function setLastConnectionId(sections, id) {
    let meta = sections.find((section) => section.name === METADATA_SECTION);
    if (!meta) {
        meta = { name: METADATA_SECTION, entries: {} };
        sections.push(meta);
    }
    meta.entries[LAST_CONNECTION_KEY] = id;
}
export function listConnections() {
    return readSections()
        .map(mapSectionToConnection)
        .filter(Boolean)
        .map((conn) => ({
        id: conn.id || defaultSection(conn.type),
        name: conn?.name,
        type: conn.type,
        host: conn.host,
        port: conn.port,
        user: conn.user,
        database: conn.database,
        file: conn.file,
        uri: conn.uri,
    }));
}
export function loadConnection(id) {
    const match = readSections()
        .map(mapSectionToConnection)
        .find((conn) => conn && (conn.id === id || defaultSection(conn.type) === id));
    return match || null;
}
export function saveConnection(profile) {
    const sections = readSections();
    const sectionName = profile.id || defaultSection(profile.type);
    const password = profile.password ? encrypt(profile.password) : undefined;
    const entries = {
        name: profile.name || "",
    };
    if (profile.type === "mysql") {
        const mysqlProfile = profile;
        entries["host"] = mysqlProfile.host;
        entries["port"] = mysqlProfile.port;
        entries["username"] = mysqlProfile.user;
        if (password)
            entries["password"] = password;
        if (mysqlProfile.database)
            entries["database"] = mysqlProfile.database;
    }
    else if (profile.type === "postgres") {
        const pgProfile = profile;
        entries["host"] = pgProfile.host;
        entries["port"] = pgProfile.port;
        entries["username"] = pgProfile.user;
        if (password)
            entries["password"] = password;
        entries["database"] = pgProfile.database;
    }
    else if (profile.type === "sqlite") {
        const sqliteProfile = profile;
        entries["file"] = sqliteProfile.file;
    }
    else if (profile.type === "mongodb") {
        const mongoProfile = profile;
        entries["uri"] = mongoProfile.uri;
        entries["host"] = mongoProfile.uri;
        if (mongoProfile.database)
            entries["database"] = mongoProfile.database;
    }
    const idx = sections.findIndex((section) => section.name === sectionName);
    const nextSection = { name: sectionName, entries };
    if (idx >= 0) {
        sections[idx] = nextSection;
    }
    else {
        sections.push(nextSection);
    }
    setLastConnectionId(sections, sectionName);
    writeSections(sections);
    return {
        id: sectionName,
        name: profile.name,
        type: profile.type,
        host: profile.host,
        port: profile.port,
        user: profile.user,
        database: profile.database,
        file: profile.file,
        uri: profile.uri,
    };
}
export function getLastConnectionId() {
    const sections = readSections();
    return getLastConnectionIdFromSections(sections);
}
//# sourceMappingURL=config.js.map