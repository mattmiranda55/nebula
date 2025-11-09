<script setup lang="ts">
// NebulaLayout: lightweight slot-driven IDE layout
// Slots: activity, sidebar, header, editor, panel, right, status, command-palette
</script>

<template>
	<div class="nebula-root">

		<div class="sidebar">
			<slot name="sidebar" />
		</div>

		<header class="header">
			<slot name="header" />
		</header>

		<main class="editor">
			<slot name="editor" />
		</main>

		<section class="terminal">
			<slot name="terminal" />
		</section>

		<!-- overlay slot for things like command palette -->
		<div class="overlay">
			<slot name="command-palette" />
		</div>
	</div>
</template>

<style scoped>
:root {
	--nebula-activity-width: 56px;
	--nebula-sidebar-width: 280px;
	--nebula-right-width: 320px;
	--nebula-panel-height: 220px;
	--nebula-status-height: 24px;
}
.nebula-root {
	display: grid;
	height: 100vh;
	grid-template-columns: var(--nebula-activity-width) var(--nebula-sidebar-width) 1fr var(--nebula-right-width);
	grid-template-rows: auto 1fr var(--nebula-panel-height) var(--nebula-status-height);
	gap: 0;
	background: var(--surface, #0f1720);
	color: var(--on-surface, #e6eef6);
}
.activity { grid-column: 1 / 2; grid-row: 1 / 4; border-right: 1px solid rgba(255,255,255,0.03); display:flex; align-items:flex-start; }
.sidebar { grid-column: 2 / 3; grid-row: 1 / 3; border-right: 1px solid rgba(255,255,255,0.03); overflow:auto; }
.header { grid-column: 3 / 4; grid-row: 1 / 2; border-bottom: 1px solid rgba(255,255,255,0.03); display:flex; align-items:center; padding:0 12px; }
.editor { grid-column: 3 / 4; grid-row: 2 / 3; overflow:auto; padding:12px; }
.right { grid-column: 4 / 5; grid-row: 1 / 3; border-left: 1px solid rgba(255,255,255,0.03); overflow:auto; }
.panel { grid-column: 2 / 5; grid-row: 3 / 4; border-top: 1px solid rgba(255,255,255,0.03); overflow:auto; }
.status { grid-column: 1 / 5; grid-row: 4 / 5; border-top: 1px solid rgba(255,255,255,0.03); display:flex; align-items:center; padding:0 12px; }
.overlay { position: fixed; inset: 0; pointer-events: none; }
.overlay > * { pointer-events: auto; }

/* small responsive tweak */
@media (max-width: 900px) {
	.nebula-root { grid-template-columns: var(--nebula-activity-width) 1fr; grid-template-rows: auto 1fr var(--nebula-panel-height) var(--nebula-status-height); }
	.sidebar { display:none; }
	.right { display:none; }
	.panel { grid-column: 1 / 3; }
	.status { grid-column: 1 / 3; }
}
</style>
