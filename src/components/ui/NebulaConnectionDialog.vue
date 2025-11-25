<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue';
import type { ConnectPayload, DatabaseType, ConnectionStatus } from '../../types/database';

const databaseOptions: { label: string; value: DatabaseType }[] = [
	{ label: 'MySQL', value: 'mysql' },
	{ label: 'PostgreSQL', value: 'postgres' },
	{ label: 'SQLite', value: 'sqlite' },
	{ label: 'MongoDB', value: 'mongodb' },
];

const defaultPorts: Record<'mysql' | 'postgres', number> = {
	mysql: 3306,
	postgres: 5432,
};

const props = defineProps<{
	visible: boolean;
	status: ConnectionStatus;
	error: string | null;
}>();

const emit = defineEmits<{
	(e: 'update:visible', value: boolean): void;
	(e: 'submit', payload: ConnectPayload): void;
}>();

const form = reactive({
	name: '',
	type: 'mysql' as DatabaseType,
	host: '127.0.0.1',
	port: defaultPorts.mysql,
	user: '',
	password: '',
	database: '',
	file: '',
	uri: '',
});

const localError = ref<string | null>(null);
const isConnecting = computed(() => props.status === 'connecting');

watch(
	() => props.visible,
	(visible) => {
		if (visible) {
			localError.value = null;
			if (form.type === 'mysql' || form.type === 'postgres') {
				if (!form.host) {
					form.host = '127.0.0.1';
				}
			}
			if (form.type === 'mongodb' && !form.uri) {
				form.uri = 'mongodb://127.0.0.1:27017';
			}
		}
	}
);

watch(
	() => form.type,
	(type) => {
		localError.value = null;

		if (type === 'mysql') {
			form.port = defaultPorts.mysql;
			if (!form.host) {
				form.host = '127.0.0.1';
			}
		}

		if (type === 'postgres') {
			form.port = defaultPorts.postgres;
			if (!form.host) {
				form.host = '127.0.0.1';
			}
		}

		if (type === 'mongodb' && !form.uri) {
			form.uri = 'mongodb://127.0.0.1:27017';
		}
	}
);

function closeDialog() {
	emit('update:visible', false);
}

function handleSubmit() {
	localError.value = null;
	const name = form.name?.trim() || undefined;

	if (form.type === 'mysql') {
		if (!form.host.trim() || !form.user.trim()) {
			localError.value = 'Host and username are required.';
			return;
		}
		if (!form.database.trim()) {
			localError.value = 'Please provide a default database/schema.';
			return;
		}

		emit('submit', {
			type: 'mysql',
			name,
			host: form.host.trim(),
			port: typeof form.port === 'number' && !Number.isNaN(form.port) ? form.port : defaultPorts.mysql,
			user: form.user.trim(),
			password: form.password,
			database: form.database.trim(),
		});
		return;
	}

	if (form.type === 'postgres') {
		if (!form.host.trim() || !form.user.trim()) {
			localError.value = 'Host and username are required.';
			return;
		}
		if (!form.database.trim()) {
			localError.value = 'Database name is required for PostgreSQL connections.';
			return;
		}

		emit('submit', {
			type: 'postgres',
			name,
			host: form.host.trim(),
			port: typeof form.port === 'number' && !Number.isNaN(form.port) ? form.port : defaultPorts.postgres,
			user: form.user.trim(),
			password: form.password,
			database: form.database.trim(),
		});
		return;
	}

	if (form.type === 'sqlite') {
		if (!form.file.trim()) {
			localError.value = 'Provide a file path for your SQLite database.';
			return;
		}

		emit('submit', {
			type: 'sqlite',
			name,
			file: form.file.trim(),
		});
		return;
	}

	if (form.type === 'mongodb') {
		if (!form.uri.trim()) {
			localError.value = 'A MongoDB connection string (URI) is required.';
			return;
		}

		emit('submit', {
			type: 'mongodb',
			name,
			uri: form.uri.trim(),
			database: form.database?.trim() || undefined,
		});
		return;
	}

	localError.value = 'Unsupported database type.';
}
</script>

<template>
	<Dialog
		:visible="visible"
		modal
		:draggable="false"
		:closable="true"
		:dismissable-mask="false"
		class="connection-dialog"
		header="Create Connection"
		@update:visible="emit('update:visible', $event)"
	>
		<div class="dialog-body">
			<p class="dialog-lead">
				Connect to your database server to start exploring schemas and running queries.
			</p>

			<div class="form-grid">
				<div class="field">
					<label>Display Name</label>
					<InputText v-model="form.name" placeholder="Optional label" />
				</div>

				<div class="field">
					<label>Database Type</label>
					<Dropdown
						v-model="form.type"
						:options="databaseOptions"
						option-label="label"
						option-value="value"
					/>
				</div>

				<template v-if="form.type === 'mysql' || form.type === 'postgres'">
					<div class="field">
						<label>Host</label>
						<InputText v-model="form.host" placeholder="127.0.0.1" />
					</div>

					<div class="field">
						<label>Port</label>
						<InputText
							v-model.number="form.port"
							type="number"
							:placeholder="form.type === 'mysql' ? defaultPorts.mysql.toString() : defaultPorts.postgres.toString()"
						/>
					</div>

					<div class="field">
						<label>Username</label>
						<InputText v-model="form.user" :placeholder="form.type === 'mysql' ? 'root' : 'postgres'" />
					</div>

					<div class="field">
						<label>Password</label>
						<Password v-model="form.password" toggle-mask :feedback="false" input-class="password-input" />
					</div>

					<div class="field field-span">
						<label>{{ form.type === 'postgres' ? 'Database' : 'Default Database' }}</label>
						<InputText
							v-model="form.database"
							:placeholder="form.type === 'postgres' ? 'postgres' : 'nebula'"
						/>
					</div>
				</template>

				<template v-else-if="form.type === 'sqlite'">
					<div class="field field-span">
						<label>Database File</label>
						<InputText v-model="form.file" placeholder="/path/to/database.sqlite" />
					</div>
				</template>

				<template v-else-if="form.type === 'mongodb'">
					<div class="field field-span">
						<label>Connection String</label>
						<InputText v-model="form.uri" placeholder="mongodb://127.0.0.1:27017" />
					</div>
					<div class="field">
						<label>Database (optional)</label>
						<InputText v-model="form.database" placeholder="admin" />
					</div>
				</template>
			</div>

			<div v-if="localError || error" class="inline-error">
				{{ localError || error }}
			</div>
		</div>

		<template #footer>
			<div class="dialog-footer">
				<Button
					label="Cancel"
					class="p-button-text"
					@click="closeDialog"
				/>
				<Button
					label="Connect"
					:loading="isConnecting"
					class="p-button-rounded p-button-primary"
					@click="handleSubmit"
				/>
			</div>
		</template>
	</Dialog>
</template>

<style scoped>
.connection-dialog :deep(.p-dialog-header) {
	background: linear-gradient(90deg, rgba(32, 27, 61, 0.85), rgba(20, 30, 68, 0.88));
	border-bottom: 1px solid rgba(127, 90, 240, 0.35);
	color: var(--txt-primary);
}

.connection-dialog :deep(.p-dialog-content) {
	background: rgba(10, 14, 32, 0.96);
	color: var(--txt-primary);
	padding: 24px 28px;
}

.connection-dialog :deep(.p-dialog-footer) {
	background: rgba(10, 14, 32, 0.96);
	border-top: 1px solid rgba(127, 90, 240, 0.24);
}

.dialog-body {
	display: flex;
	flex-direction: column;
	gap: 20px;
}

.dialog-lead {
	margin: 0;
	color: var(--txt-secondary);
	font-size: 14px;
	line-height: 1.5;
}

.form-grid {
	display: grid;
	grid-template-columns: repeat(2, minmax(0, 1fr));
	gap: 16px 18px;
}

.field {
	display: flex;
	flex-direction: column;
	gap: 8px;
}

.field label {
	font-size: 12px;
	letter-spacing: 0.04em;
	text-transform: uppercase;
	color: var(--txt-muted);
}

.field-span {
	grid-column: span 2;
}

.password-input {
	width: 100%;
}

.inline-error {
	margin-top: 4px;
	padding: 10px 14px;
	border-radius: var(--radius-sm);
	background: rgba(220, 38, 38, 0.18);
	color: #fca5a5;
	font-size: 13px;
}

.dialog-footer {
	display: flex;
	justify-content: flex-end;
	gap: 12px;
}

@media (max-width: 720px) {
	.form-grid {
		grid-template-columns: 1fr;
	}

	.field-span {
		grid-column: span 1;
	}
}
</style>
