<script setup lang="ts">
import { computed } from 'vue';
import type { ConnectionInfo, ConnectionStatus, SchemaNode } from '../../types/database';
import { formatConnectionLabel } from '../../utils/connection';

const props = defineProps<{
	status: ConnectionStatus;
	connection: ConnectionInfo | null;
	schema: SchemaNode[];
	isLoading: boolean;
	error: string | null;
}>();

const emit = defineEmits<{
	(e: 'request-connect'): void;
	(e: 'refresh'): void;
	(e: 'disconnect'): void;
}>();

const isConnected = computed(() => props.status === 'connected');
const statusLabel = computed(() => {
	switch (props.status) {
		case 'connected':
			return 'Connected';
		case 'connecting':
			return 'Connecting…';
		case 'error':
			return 'Connection Error';
		default:
			return 'Not Connected';
	}
});

const statusTone = computed(() => {
	if (props.status === 'connected') return 'status-chip--online';
	if (props.status === 'connecting') return 'status-chip--pending';
	if (props.status === 'error') return 'status-chip--error';
	return 'status-chip--offline';
});

const connectionLabel = computed(() => {
	if (!props.connection) {
		return '';
	}

	return formatConnectionLabel(props.connection);
});

function handleConnectClick() {
	emit('request-connect');
}

function handleRefreshClick() {
	emit('refresh');
}

function handleDisconnectClick() {
	emit('disconnect');
}
</script>

<template>
	<aside class="sidebar">
		<header class="sidebar__header">
			<div>
				<p class="sidebar__title">Connections</p>
				<div class="status-row">
					<span class="status-chip" :class="statusTone">{{ statusLabel }}</span>
					<span v-if="connectionLabel" class="status-host">{{ connectionLabel }}</span>
				</div>
			</div>

			<div class="sidebar__actions">
				<Button
					icon="pi pi-plus"
					class="p-button-rounded p-button-text"
					title="New connection"
					@click="handleConnectClick"
				/>
				<Button
					icon="pi pi-refresh"
					class="p-button-rounded p-button-text"
					:title="isLoading ? 'Refreshing…' : 'Refresh schema'"
					:loading="isLoading"
					@click="handleRefreshClick"
					:disabled="!isConnected"
				/>
				<Button
					icon="pi pi-power-off"
					class="p-button-rounded p-button-text"
					title="Disconnect"
					@click="handleDisconnectClick"
					:disabled="!isConnected"
				/>
			</div>
		</header>

		<div v-if="error" class="sidebar__alert">
			{{ error }}
		</div>

		<div class="sidebar__content">
			<div v-if="!isConnected" class="sidebar__empty">
				<h3>No active connection</h3>
				<p>Connect to a database server to populate the schema explorer.</p>
				<Button label="Create connection" class="primary" @click="handleConnectClick" />
			</div>
			<div v-else class="sidebar__tree">
				<Tree :value="schema" class="schema-tree" />
			</div>
		</div>
	</aside>
</template>

<style scoped>
.sidebar {
	display: flex;
	flex-direction: column;
	height: 100%;
	gap: 18px;
}

.sidebar__header {
	display: flex;
	justify-content: space-between;
	align-items: flex-start;
	gap: 12px;
}

.sidebar__title {
	margin: 0;
	font-size: 13px;
	letter-spacing: 0.14em;
	text-transform: uppercase;
	color: var(--txt-muted);
}

.status-row {
	display: flex;
	align-items: center;
	gap: 10px;
	margin-top: 6px;
}

.status-chip {
	padding: 4px 10px;
	border-radius: 999px;
	font-size: 11px;
	font-weight: 600;
	letter-spacing: 0.04em;
	text-transform: uppercase;
}

.status-chip--online {
	background: rgba(34, 197, 94, 0.18);
	border: 1px solid rgba(34, 197, 94, 0.45);
	color: #bbf7d0;
}

.status-chip--pending {
	background: rgba(253, 224, 71, 0.18);
	border: 1px solid rgba(250, 204, 21, 0.45);
	color: #fef08a;
}

.status-chip--offline {
	background: rgba(99, 102, 241, 0.12);
	border: 1px solid rgba(99, 102, 241, 0.32);
	color: rgba(203, 213, 225, 0.85);
}

.status-chip--error {
	background: rgba(239, 68, 68, 0.18);
	border: 1px solid rgba(239, 68, 68, 0.4);
	color: #fecaca;
}

.status-host {
	font-size: 12px;
	color: var(--txt-secondary);
}

.sidebar__actions {
	display: flex;
	gap: 6px;
}

.sidebar__alert {
	padding: 12px 14px;
	border-radius: var(--radius-sm);
	background: rgba(239, 68, 68, 0.18);
	color: #fecaca;
	font-size: 13px;
}

.sidebar__content {
	flex: 1;
	border-radius: var(--radius-md);
	background: rgba(6, 10, 27, 0.82);
	border: 1px solid rgba(127, 90, 240, 0.18);
	box-shadow: inset 0 0 32px rgba(5, 7, 20, 0.45);
	padding: 16px;
	overflow: hidden;
}

.sidebar__empty {
	display: flex;
	flex-direction: column;
	gap: 12px;
	text-align: center;
	color: var(--txt-secondary);
}

.sidebar__empty h3 {
	margin: 0;
	font-size: 16px;
	color: var(--txt-primary);
}

.sidebar__empty p {
	margin: 0;
	font-size: 13px;
}

.sidebar__empty .primary {
	align-self: center;
	margin-top: 8px;
	background: rgba(127, 90, 240, 0.9);
	border: none;
	color: #0b0d1f;
	font-weight: 600;
	padding: 0.55rem 1rem;
	border-radius: var(--radius-sm);
}

.sidebar__tree {
	height: 100%;
	overflow: hidden;
}

.schema-tree {
	height: 100%;
	background: transparent;
	border: none;
}

.schema-tree :deep(.p-tree-container) {
	padding: 4px 0;
	color: var(--txt-primary);
}

.schema-tree :deep(.p-tree-toggler) {
	color: var(--txt-secondary);
}

.schema-tree :deep(.p-tree-node-content) {
	border-radius: 8px;
}

.schema-tree :deep(.p-highlight) {
	background: rgba(127, 90, 240, 0.28);
	color: var(--txt-primary);
}
</style>