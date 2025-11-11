<script setup lang="ts">
import NebulaLayout from './components/layout/NebulaLayout.vue';
import NebulaEditor from './components/ui/NebulaEditor.vue';
import NebulaHeader from './components/ui/NebulaHeader.vue';
import NebulaSidebar from './components/ui/NebulaSidebar.vue';
import NebulaTerminal from './components/ui/NebulaTerminal.vue';
</script>

<template>
  <NebulaLayout>

    <template #sidebar>
      <NebulaSidebar />
    </template>

    <template #header>
      <NebulaHeader />
    </template>

    <template #editor>
      <NebulaEditor />
    </template>

    <template #terminal>
      <NebulaTerminal />
    </template>

  </NebulaLayout>
</template> 

<style scoped>

</style>
