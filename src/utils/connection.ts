import type { ConnectionInfo } from '../types/database';

export function formatConnectionTarget(info: ConnectionInfo): string {
  switch (info.type) {
    case 'mysql':
    case 'postgres':
      return info.name || `${info.host}:${info.port}`;
    case 'sqlite':
      return info.name || info.file;
    case 'mongodb':
      return info.name || info.database || info.uri;
    default:
      return info.name || 'Database';
  }
}

export function formatConnectionLabel(info: ConnectionInfo): string {
  const target = formatConnectionTarget(info);
  if ('user' in info && info.user) {
    return `${target} • ${info.user}`;
  }
  return target;
}

export function formatConnectionStatus(info: ConnectionInfo): string {
  const target = formatConnectionTarget(info);
  if ('user' in info && info.user) {
    return `Connected to ${target} as ${info.user}`;
  }
  return `Connected to ${target}`;
}
