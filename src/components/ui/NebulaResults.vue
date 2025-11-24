<script setup lang="ts">
const props = defineProps<{
	message?: string;
}>();
</script>

<template>
	<section class="results">
		<div class="results__empty">
			<h3>Query results</h3>
			<p>{{ props.message || 'Run a query to view results here.' }}</p>
		</div>
	</section>
</template>

<style scoped>
.results {
	border-radius: var(--radius-md);
	border: 1px solid rgba(127, 90, 240, 0.16);
	background: rgba(4, 8, 23, 0.75);
	box-shadow: inset 0 0 22px rgba(6, 9, 22, 0.36);
	height: 220px;
	padding: 20px;
}

.results__empty {
	height: 100%;
	display: flex;
	flex-direction: column;
	align-items: center;
	justify-content: center;
	gap: 8px;
	color: var(--txt-secondary);
	text-align: center;
}

.results__empty h3 {
	margin: 0;
	font-size: 16px;
	color: var(--txt-primary);
}

.results__empty p {
	margin: 0;
	font-size: 13px;
	max-width: 320px;
}
</style>