import { createApp, nextTick } from 'vue'
import './style.css'
import App from './App.vue'
import PrimeVue from 'primevue/config';
import Material from '@primeuix/themes/material';
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
  SplitterPanel
} from 'primevue';

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
  SplitterPanel
};
// Create app instance (do not mount before configuring plugins/components)
const app = createApp(App);

app.use(PrimeVue, {
  theme: {
        preset: Material,
        options: {
            prefix: 'p',
            cssLayer: false
        }
    }
});

Object.entries(components).forEach(([name, component]) => {
  app.component(name, component as any);
});

// Mount the app
app.mount('#app');

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
