import { createApp, nextTick } from 'vue'
import './style.css'
import App from './App.vue'
import PrimeVue from 'primevue/config';
import Material from '@primeuix/themes/material';
import Checkbox from 'primevue/checkbox';
import Password from 'primevue/password';
import Button from 'primevue/button';
import DataTable from 'primevue/datatable';
import Paginator from 'primevue/paginator';
import Tree from 'primevue/tree';
import Divider from 'primevue/divider';
import Panel from 'primevue/panel';
import Splitter from 'primevue/splitter';
import Drawer from 'primevue/drawer';
import Tooltip from 'primevue/tooltip';
import FileUpload from 'primevue/fileupload';
import Menu from 'primevue/menu';
import Toast from 'primevue/toast';
import Message from 'primevue/message';
import ProgressBar from 'primevue/progressbar';
import ProgressSpinner from 'primevue/progressspinner';

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
  ProgressSpinner
};
// Create app instance (do not mount before configuring plugins/components)
const app = createApp(App);

app.use(PrimeVue, {
  theme: {
        preset: Material,
        options: {
            prefix: 'p',
            darkModeSelector: 'system',
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
