<script setup lang="ts">
const emit = defineEmits<{
	(e: 'create-connection'): void;
}>();
</script>

<template>
	<section class="welcome">
		<div class="welcome__hero">
			<div class="welcome__badge">Nebula</div>
			<h1 class="welcome__title">Welcome back to your space for data exploration.</h1>
			<p class="welcome__subtitle">
				Create a connection to start exploring schemas, crafting queries, and saving insights. Nebula keeps your
				focus where it belongs—on the data.
			</p>
			<div class="welcome__actions">
				<Button label="New Connection" class="primary" @click="emit('create-connection')" />
				<Button label="Import Settings" class="ghost" disabled />
			</div>
		</div>

		<div class="welcome__grid">
			<div class="welcome__card">
				<div class="card__icon pi pi-database"></div>
				<h3>Connect Instantly</h3>
				<p>
					Support for MySQL connections out of the box. Fill in your credentials and start browsing schemas within
					seconds.
				</p>
			</div>
			<div class="welcome__card">
				<div class="card__icon pi pi-sitemap"></div>
				<h3>Explore Schemas</h3>
				<p>
					Browse databases, tables, and columns from the sidebar. Quickly refresh to sync structural changes from
					your servers.
				</p>
			</div>
			<div class="welcome__card">
				<div class="card__icon pi pi-code"></div>
				<h3>Query with Monaco</h3>
				<p>
					Write SQL using a familiar editor experience. Syntax highlighting, multi-cursor editing, and command
					palettes are ready for liftoff.
				</p>
			</div>
		</div>
	</section>
</template>

<style scoped>
.welcome {
	display: flex;
	flex-direction: column;
	gap: 36px;
	padding: 32px;
	border-radius: var(--radius-lg);
	background: radial-gradient(circle at top left, rgba(127, 90, 240, 0.12), transparent 60%),
		radial-gradient(circle at top right, rgba(130, 207, 255, 0.12), transparent 55%),
		rgba(9, 12, 28, 0.7);
	box-shadow: var(--shadow-soft);
	min-height: 100%;
}

.welcome__hero {
	display: flex;
	flex-direction: column;
	gap: 18px;
	max-width: 680px;
}

.welcome__badge {
	width: fit-content;
	padding: 6px 12px;
	border-radius: 999px;
	background: rgba(124, 58, 237, 0.18);
	border: 1px solid rgba(127, 90, 240, 0.32);
	color: var(--accent-secondary);
	letter-spacing: 0.08em;
	font-size: 11px;
	text-transform: uppercase;
}

.welcome__title {
	margin: 0;
	font-size: 32px;
	font-weight: 700;
	line-height: 1.25;
}

.welcome__subtitle {
	margin: 0;
	color: var(--txt-secondary);
	font-size: 16px;
	line-height: 1.6;
}

.welcome__actions {
	display: flex;
	gap: 12px;
}

.welcome__actions .primary {
	background: linear-gradient(135deg, rgba(127, 90, 240, 0.9) 0%, rgba(130, 207, 255, 0.9) 100%);
	border: none;
	color: #0b0d1f;
	font-weight: 600;
	padding: 0.65rem 1.4rem;
	border-radius: var(--radius-sm);
}

.welcome__actions .ghost {
	background: transparent;
	border: 1px solid rgba(127, 90, 240, 0.35);
	color: var(--txt-secondary);
	padding: 0.65rem 1.4rem;
	border-radius: var(--radius-sm);
}

.welcome__grid {
	display: grid;
	grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
	gap: 18px;
}

.welcome__card {
	background: rgba(8, 11, 27, 0.85);
	border: 1px solid rgba(127, 90, 240, 0.24);
	border-radius: var(--radius-md);
	padding: 20px;
	display: flex;
	flex-direction: column;
	gap: 12px;
	box-shadow: inset 0 0 26px rgba(14, 17, 34, 0.38);
}

.welcome__card h3 {
	margin: 0;
	font-size: 16px;
}

.welcome__card p {
	margin: 0;
	color: var(--txt-secondary);
	font-size: 14px;
	line-height: 1.5;
}

.card__icon {
	font-size: 22px;
	color: var(--accent-secondary);
}

@media (max-width: 720px) {
	.welcome {
		padding: 24px;
	}

	.welcome__title {
		font-size: 26px;
	}
}
</style>
