<script setup lang="ts">
import { computed, reactive, ref, watch, onMounted } from 'vue';
import NebulaLayout from './components/layout/NebulaLayout.vue';
import NebulaSidebar from './components/ui/NebulaSidebar.vue';
import NebulaHeader from './components/ui/NebulaHeader.vue';
import NebulaEditor from './components/ui/NebulaEditor.vue';
import NebulaResults from './components/ui/NebulaResults.vue';
import NebulaWelcome from './components/ui/NebulaWelcome.vue';
import NebulaConnectionDialog from './components/ui/NebulaConnectionDialog.vue';
import type {
  ConnectPayload,
  ConnectionInfo,
  ConnectionStatus,
  PersistConnectionPayload,
  SchemaNode,
} from './types/database';
import { formatConnectionStatus } from './utils/connection';

const connection = reactive({
  status: 'disconnected' as ConnectionStatus,
  info: null as ConnectionInfo | null,
  error: null as string | null,
});

const schemaTree = ref<SchemaNode[]>([
  {
    key: 'disconnected',
    label: 'Not Connected',
    icon: 'pi pi-lock',
    children: [],
  },
]);

const schemaLoading = ref(false);
const showConnectionDialog = ref(false);
const welcomeDismissed = ref(false);

const isConnected = computed(() => connection.status === 'connected');
const connectionError = computed(() => (connection.status === 'error' ? connection.error : null));
const shouldShowWelcome = computed(() => !isConnected.value && !welcomeDismissed.value);

const connectionFooterText = computed(() => {
  if (!connection.info || connection.status !== 'connected') {
    return 'Disconnected';
  }

  return formatConnectionStatus(connection.info);
});

watch(
  () => connection.status,
  (status) => {
    if (status !== 'connected') {
      schemaTree.value = [
        {
          key: 'disconnected',
          label: 'Not Connected',
          icon: 'pi pi-lock',
          children: [],
        },
      ];
    }
  }
);

function openConnectionDialog() {
  connection.error = null;
  showConnectionDialog.value = true;
}

async function attemptAutoConnect() {
  if (connection.status !== 'disconnected') return;
  if (!window.config?.listConnections || !window.config?.connectSaved || !window.config?.getLastConnection) return;

  try {
    const { id, error: lastError } = await window.config.getLastConnection();
    if (lastError || !id) return;

    const { connections, error } = await window.config.listConnections();
    if (error || !connections?.length) return;

    const saved = connections.find((c) => c.id === id);
    if (!saved) return;

    connection.status = 'connecting';
    connection.error = null;

    const response = await window.config.connectSaved(saved.id);
    if (response?.error) {
      connection.status = 'error';
      connection.error = `Failed to restore connection: ${response.error}`;
      return;
    }

    let info: ConnectionInfo | null = null;
    switch (saved.type) {
      case 'mysql': {
        info = {
          type: 'mysql',
          name: saved.name,
          host: saved.host || '',
          port: saved.port ?? 3306,
          user: saved.user || '',
          database: saved.database,
        };
        break;
      }
      case 'postgres': {
        info = {
          type: 'postgres',
          name: saved.name,
          host: saved.host || '',
          port: saved.port ?? 5432,
          user: saved.user || '',
          database: saved.database || '',
        };
        break;
      }
      case 'sqlite': {
        info = {
          type: 'sqlite',
          name: saved.name,
          file: saved.file || '',
        };
        break;
      }
      case 'mongodb': {
        info = {
          type: 'mongodb',
          name: saved.name,
          uri: saved.uri || '',
          database: saved.database,
        };
        break;
      }
    }

    if (!info) return;

    connection.status = 'connected';
    connection.info = info;
    welcomeDismissed.value = true;
    await loadSchema();
  } catch (err: any) {
    connection.status = 'error';
    connection.error = err?.message ?? 'Failed to reconnect to saved database';
  }
}

onMounted(() => {
  attemptAutoConnect();
});

async function handleConnect(payload: ConnectPayload) {
  connection.status = 'connecting';
  connection.error = null;

  const displayName = payload.name?.trim() || undefined;

  try {
    if (!window?.db?.connect) {
      throw new Error('Database bridge is not available in this environment.');
    }

    let config: PersistConnectionPayload;
    let info: ConnectionInfo;

    switch (payload.type) {
      case 'mysql': {
        const host = payload.host.trim();
        const port = typeof payload.port === 'string' ? Number(payload.port) || 3306 : payload.port || 3306;
        const user = payload.user.trim();
        const database = payload.database.trim();
        config = {
          type: 'mysql',
          host,
          port,
          user,
          password: payload.password,
          database: database || undefined,
        };
        info = {
          type: 'mysql',
          name: displayName,
          host,
          port,
          user,
          database: database || undefined,
        };
        break;
      }
      case 'postgres': {
        const host = payload.host.trim();
        const port = typeof payload.port === 'string' ? Number(payload.port) || 5432 : payload.port || 5432;
        const user = payload.user.trim();
        const database = payload.database.trim();
        config = {
          type: 'postgres',
          host,
          port,
          user,
          password: payload.password,
          database,
        };
        info = {
          type: 'postgres',
          name: displayName,
          host,
          port,
          user,
          database,
        };
        break;
      }
      case 'sqlite': {
        const file = payload.file.trim();
        config = {
          type: 'sqlite',
          file,
        };
        info = {
          type: 'sqlite',
          name: displayName,
          file,
        };
        break;
      }
      case 'mongodb': {
        const uri = payload.uri.trim();
        const database = payload.database?.trim() || undefined;
        config = {
          type: 'mongodb',
          uri,
          database,
        };
        info = {
          type: 'mongodb',
          name: displayName,
          uri,
          database,
        };
        break;
      }
      default: {
        throw new Error('Unsupported database type.');
      }
    }

    const response = await window.db.connect(config);

    if (response?.error) {
      connection.status = 'error';
      connection.error = response.error;
      return;
    }

    if (window.config?.saveConnection) {
      try {
        await window.config.saveConnection({ ...config, name: displayName });
      } catch (persistErr: any) {
        console.warn('Failed to persist connection:', persistErr?.message ?? persistErr);
      }
    }

    connection.status = 'connected';
    connection.info = info;
    welcomeDismissed.value = true;
    showConnectionDialog.value = false;
    await loadSchema();
  } catch (err: any) {
    connection.status = 'error';
    connection.error = err?.message ?? 'Failed to connect to database';
  }
}

async function handleDisconnect() {
  try {
    await window?.db?.disconnect?.();
  } catch (err) {
    console.warn('Disconnect failed', err);
  }
  connection.status = 'disconnected';
  connection.info = null;
  connection.error = null;
  welcomeDismissed.value = false;
  showConnectionDialog.value = false;
}

async function loadSchema() {
  if (!isConnected.value || !window?.db?.structure) {
    return;
  }

  schemaLoading.value = true;
  try {
    const result = await window.db.structure();

    if (result?.error) {
      throw new Error(result.error);
    }

    const nodes = Array.isArray(result?.nodes) ? result.nodes : [];

    if (!nodes.length) {
      schemaTree.value = [
        {
          key: 'schema-empty',
          label: 'No schema information available',
          icon: 'pi pi-info-circle',
          children: [],
        },
      ];
    } else {
      schemaTree.value = nodes;
    }
  } catch (err: any) {
    console.error('Failed to load database structure', err);
    const message = err?.message ?? 'Failed to load database structure';
    connection.error = message;
    if (/No active database connection/i.test(message)) {
      connection.status = 'disconnected';
    }
  } finally {
    schemaLoading.value = false;
  }
}

function handleSchemaRefresh() {
  loadSchema();
}

function handleWelcomeConnect() {
  openConnectionDialog();
}

function handleEditorFocus() {
  if (!isConnected.value) {
    openConnectionDialog();
  }
}
</script>

<template>
  <NebulaLayout>
    <template #header>
      <NebulaHeader
        :status="connection.status"
        :connection="connection.info"
        :error="connectionError"
        @request-connect="openConnectionDialog"
        @disconnect="handleDisconnect"
      />
    </template>

    <template #sidebar>
      <NebulaSidebar
        :status="connection.status"
        :connection="connection.info"
        :schema="schemaTree"
        :is-loading="schemaLoading"
        :error="connectionError"
        @request-connect="openConnectionDialog"
        @refresh="handleSchemaRefresh"
        @disconnect="handleDisconnect"
      />
    </template>

    <template #main>
      <NebulaWelcome
        v-if="shouldShowWelcome"
        @create-connection="handleWelcomeConnect"
      />
      <div v-else class="nebula-main-pane">
        <NebulaEditor :connected="isConnected" @request-connect="handleEditorFocus" />
        <NebulaResults v-if="isConnected" />
      </div>
    </template>

    <template #footer>
      <div>
        <span>{{ connectionFooterText }}</span>
      </div>
      <div>
        <span v-if="schemaLoading">Loading schema…</span>
      </div>
    </template>
  </NebulaLayout>

  <NebulaConnectionDialog
    v-model:visible="showConnectionDialog"
    :status="connection.status"
    :error="connectionError"
    @submit="handleConnect"
  />
</template>

<style scoped>
.nebula-main-pane {
  display: flex;
  flex-direction: column;
  gap: 16px;
  height: 100%;
}
</style>
