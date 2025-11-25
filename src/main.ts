import { createApp, nextTick } from 'vue'
import App from './App.vue'
import PrimeVue from 'primevue/config';
import {
  Checkbox,
  Password,
  Button,
  DataTable,
  Paginator,
  Tree,
  Divider,
  Panel,
  Splitter,
  Drawer,
  Tooltip,
  FileUpload,
  Menu,
  Toast,
  Message,
  ProgressBar,
  ProgressSpinner,
  Column,
  Card,
  InputText,
  SplitterPanel,
  Dialog,
  Dropdown
} from 'primevue';

// icons
// import 'primevue/resources/themes/aura-dark-purple/theme.css';
// import 'primevue/resources/primevue.min.css';
import 'primeicons/primeicons.css';

import './style.css'

const components = {
  Checkbox,
  Password,
  Button,
  DataTable,
  Paginator,
  Tree,
  Divider,
  Panel,
  Splitter,
  Drawer,
  Tooltip,
  FileUpload,
  Menu,
  Toast,
  Message,
  ProgressBar,
  ProgressSpinner,
  Column,
  Card,
  InputText,
  SplitterPanel,
  Dialog,
  Dropdown
};

const app = createApp(App);

app.use(PrimeVue);

Object.entries(components).forEach(([name, component]) => {
  app.component(name, component as any);
});

app.mount('#app')

// Hook into IPC after mount. Use optional chaining to avoid runtime errors
// when preload script didn't expose the API (e.g., running in plain browser).
nextTick(() => {
  try {
    (window as any).ipcRenderer?.on?.('main-process-message', (_event: any, message: any) => {
      console.log(message);
    });
  } catch (e) {
    // swallow — nothing to do in non-Electron environments
  }
});

