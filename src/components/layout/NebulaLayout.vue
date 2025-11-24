<template>
	<div class="nebula-shell">
		<header class="nebula-shell__header">
			<slot name="header" />
		</header>
		<div class="nebula-shell__body">
			<Splitter class="nebula-shell__splitter">
				<SplitterPanel :size="23" :minSize="14" class="nebula-shell__sidebar">
					<slot name="sidebar" />
				</SplitterPanel>
				<SplitterPanel :size="77" :minSize="40" class="nebula-shell__main">
					<div class="nebula-shell__content">
						<slot name="main" />
					</div>
				</SplitterPanel>
			</Splitter>
		</div>
		<footer class="nebula-shell__footer">
			<slot name="footer" />
		</footer>
		<div class="nebula-shell__overlay">
			<slot name="command-palette" />
		</div>
	</div>
</template>

<style scoped>
.nebula-shell {
	min-height: 100vh;
	display: grid;
	grid-template-rows: auto 1fr auto;
	color: var(--txt-primary);
}

.nebula-shell__header {
	position: sticky;
	top: 0;
	z-index: 5;
	background: var(--bg-header);
	box-shadow: 0 18px 24px rgba(3, 4, 21, 0.55);
	border-bottom: 1px solid var(--border-soft);
}

.nebula-shell__body {
	min-height: 0;
}

.nebula-shell__splitter {
	height: 100%;
	border: none;
	background: transparent;
}

.nebula-shell__sidebar {
	background: var(--bg-sidebar);
	border-right: 1px solid var(--border-soft);
	padding: 18px 14px;
	backdrop-filter: blur(22px);
	box-shadow: inset -24px 0 48px rgba(5, 7, 22, 0.45);
	overflow: hidden;
}

.nebula-shell__sidebar ::-webkit-scrollbar {
	width: 6px;
}

.nebula-shell__sidebar ::-webkit-scrollbar-thumb {
	background: rgba(132, 94, 247, 0.38);
	border-radius: 999px;
}

.nebula-shell__main {
	background: transparent;
	backdrop-filter: blur(24px);
	overflow: hidden;
}

.nebula-shell__content {
	height: 100%;
	padding: 20px 24px 24px;
	background: var(--bg-panel);
	border-left: 1px solid rgba(127, 90, 240, 0.14);
	border-right: 1px solid rgba(46, 226, 234, 0.09);
	box-shadow: inset 0 0 56px rgba(8, 12, 38, 0.45);
	border-radius: 0;
	display: flex;
	flex-direction: column;
}

.nebula-shell__footer {
	height: 38px;
	background: linear-gradient(90deg, rgba(12, 18, 43, 0.9) 0%, rgba(7, 11, 28, 0.95) 100%);
	border-top: 1px solid var(--border-soft);
	display: flex;
	align-items: center;
	justify-content: space-between;
	padding: 0 18px;
	color: var(--txt-secondary);
	font-size: 12px;
}

.nebula-shell__footer:empty {
	display: none;
}

.nebula-shell__overlay {
	position: fixed;
	inset: 0;
	pointer-events: none;
}

.nebula-shell__overlay > * {
	pointer-events: auto;
}

@media (max-width: 1024px) {
	.nebula-shell__content {
		padding: 16px;
	}

	.nebula-shell__sidebar {
		padding: 16px 12px;
	}
}
</style>

