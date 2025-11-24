export type ConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'error';

export type DatabaseType = 'mysql' | 'postgres' | 'sqlite' | 'mongodb';

export interface BaseConnectionInfo {
  name?: string;
  type: DatabaseType;
}

export interface MysqlConnectionInfo extends BaseConnectionInfo {
  type: 'mysql';
  host: string;
  port: number;
  user: string;
  database?: string;
}

export interface PostgresConnectionInfo extends BaseConnectionInfo {
  type: 'postgres';
  host: string;
  port: number;
  user: string;
  database: string;
}

export interface SqliteConnectionInfo extends BaseConnectionInfo {
  type: 'sqlite';
  file: string;
}

export interface MongoConnectionInfo extends BaseConnectionInfo {
  type: 'mongodb';
  uri: string;
  database?: string;
}

export type ConnectionInfo =
  | MysqlConnectionInfo
  | PostgresConnectionInfo
  | SqliteConnectionInfo
  | MongoConnectionInfo;

export interface MysqlConnectPayload extends BaseConnectionInfo {
  type: 'mysql';
  host: string;
  port: number | string;
  user: string;
  password: string;
  database: string;
}

export interface PostgresConnectPayload extends BaseConnectionInfo {
  type: 'postgres';
  host: string;
  port: number | string;
  user: string;
  password: string;
  database: string;
}

export interface SqliteConnectPayload extends BaseConnectionInfo {
  type: 'sqlite';
  file: string;
}

export interface MongoConnectPayload extends BaseConnectionInfo {
  type: 'mongodb';
  uri: string;
  database?: string;
}

export type ConnectPayload =
  | MysqlConnectPayload
  | PostgresConnectPayload
  | SqliteConnectPayload
  | MongoConnectPayload;

export interface SchemaNode {
  key: string;
  label: string;
  icon?: string;
  children?: SchemaNode[];
}
