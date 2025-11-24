<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import MonacoEditor from 'monaco-editor-vue3';
import '../../monaco-setup';
import 'monaco-editor/min/vs/editor/editor.main.css';

const props = defineProps<{
  connected: boolean;
}>();

const emit = defineEmits<{
  (e: 'request-connect'): void;
}>();

const code = ref(
  `-- Write your SQL here
SELECT *
FROM information_schema.tables
LIMIT 100;`
);

const editorOptions = computed(() => ({
  automaticLayout: true,
  minimap: { enabled: false },
  lineNumbers: 'on',
  glyphMargin: false,
  folding: true,
  renderLineHighlight: 'line',
  readOnly: !props.connected,
  scrollBeyondLastLine: false,
  language: 'sql',
}));

watch(
  () => props.connected,
  (connected) => {
    if (!connected) {
      code.value = code.value; // retain content but triggers reactivity when needed
    }
  }
);

function handleConnectRequest() {
  emit('request-connect');
}
</script>

<template>
  <div class="editor-pane">
    <MonacoEditor
      v-model="code"
      theme="vs-dark"
      language="sql"
      :options="editorOptions"
      class="editor-pane__monaco"
    />
    <div v-if="!connected" class="editor-pane__overlay">
      <h2>Connect to begin querying</h2>
      <p>
        Establish a database connection to run SQL, get IntelliSense, and execute commands right from the Monaco
        editor.
      </p>
      <Button label="Create connection" class="primary" @click="handleConnectRequest" />
    </div>
  </div>
</template>

<style scoped>
.editor-pane {
  position: relative;
  flex: 1;
  min-height: 240px;
  border-radius: var(--radius-md);
  border: 1px solid rgba(127, 90, 240, 0.18);
  background: rgba(5, 9, 24, 0.88);
  box-shadow: inset 0 0 32px rgba(6, 9, 22, 0.45);
  overflow: hidden;
}

.editor-pane__monaco {
  height: 100%;
}

.editor-pane__overlay {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 32px;
  text-align: center;
  background: rgba(5, 9, 24, 0.92);
  backdrop-filter: blur(8px);
  gap: 12px;
}

.editor-pane__overlay h2 {
  margin: 0;
  font-size: 20px;
}

.editor-pane__overlay p {
  margin: 0;
  color: var(--txt-secondary);
  font-size: 14px;
  max-width: 360px;
}

.editor-pane__overlay .primary {
  margin-top: 12px;
  background: rgba(127, 90, 240, 0.9);
  border: none;
  color: #0b0d1f;
  font-weight: 600;
  padding: 0.65rem 1.6rem;
  border-radius: var(--radius-sm);
}
</style>
