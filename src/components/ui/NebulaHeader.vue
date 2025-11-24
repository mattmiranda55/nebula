<script setup lang="ts">
import { computed } from 'vue';
import type { ConnectionInfo, ConnectionStatus } from '../../types/database';
import { formatConnectionStatus } from '../../utils/connection';

const props = defineProps<{
	status: ConnectionStatus;
	connection: ConnectionInfo | null;
	error: string | null;
}>();

const emit = defineEmits<{
	(e: 'request-connect'): void;
	(e: 'disconnect'): void;
}>();

const statusDescription = computed(() => {
	if (props.status === 'connected' && props.connection) {
		return formatConnectionStatus(props.connection);
	}
	if (props.status === 'connecting') {
		return 'Attempting to establish a connection…';
	}
	if (props.status === 'error' && props.error) {
		return props.error;
	}
	return 'No active database connection';
});

const statusTone = computed(() => {
	if (props.status === 'connected') return 'tone-online';
	if (props.status === 'connecting') return 'tone-pending';
	if (props.status === 'error') return 'tone-error';
	return 'tone-offline';
});

function handleConnectClick() {
	emit('request-connect');
}

function handleDisconnect() {
	emit('disconnect');
}
</script>

<template>
	<div class="header">
		<div class="brand">
			<div class="brand__glyph"></div>
			<div class="brand__copy">
				<p class="brand__title">Nebula Studio</p>
				<p class="brand__subtitle">Database exploration reimagined</p>
			</div>
		</div>

		<div class="status" :class="statusTone">
			<p>{{ statusDescription }}</p>
		</div>

		<div class="actions">
			<Button
				label="New Connection"
				icon="pi pi-plug"
				class="p-button-rounded p-button-text"
				@click="handleConnectClick"
			/>
			<Button
				label="Disconnect"
				icon="pi pi-power-off"
				class="p-button-rounded p-button-text"
				@click="handleDisconnect"
				:disabled="status !== 'connected'"
			/>
		</div>
	</div>
</template>

<style scoped>
.header {
	display: flex;
	align-items: center;
	justify-content: space-between;
	padding: 14px 24px;
	gap: 32px;
}

.brand {
	display: flex;
	align-items: center;
	gap: 14px;
}

.brand__glyph {
	width: 44px;
	height: 44px;
	border-radius: 14px;
	background: conic-gradient(from 180deg at 50% 50%, rgba(130, 207, 255, 0.95) 0deg, rgba(127, 90, 240, 0.9) 140deg, rgba(10, 13, 28, 0.95) 360deg);
	box-shadow: var(--shadow-glow);
}

.brand__copy {
	display: flex;
	flex-direction: column;
	gap: 4px;
}

.brand__title {
	margin: 0;
	font-size: 18px;
	font-weight: 600;
	letter-spacing: 0.08em;
	text-transform: uppercase;
}

.brand__subtitle {
	margin: 0;
	font-size: 12px;
	color: var(--txt-secondary);
}

.status {
	flex: 1;
	max-width: 520px;
	padding: 12px 18px;
	border-radius: var(--radius-md);
	border: 1px solid rgba(127, 90, 240, 0.28);
	background: rgba(9, 12, 29, 0.62);
	display: flex;
	align-items: center;
	color: var(--txt-secondary);
}

.status p {
	margin: 0;
	font-size: 13px;
}

.status.tone-online {
	border-color: rgba(34, 197, 94, 0.36);
	color: #bbf7d0;
}

.status.tone-pending {
	border-color: rgba(250, 204, 21, 0.32);
	color: #fef08a;
}

.status.tone-error {
	border-color: rgba(248, 113, 113, 0.45);
	color: #fecaca;
}

.actions {
	display: flex;
	gap: 10px;
}

.actions :deep(.pi) {
	font-size: 1rem;
}
</style>