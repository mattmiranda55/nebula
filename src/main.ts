import { createApp } from 'vue'
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

var app = createApp(App).mount('#app').$nextTick(() => {
  // Use contextBridge
  window.ipcRenderer.on('main-process-message', (_event, message) => {
    console.log(message)
  })
})

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

Object.entries(components).forEach(([name, component]) => {
  app.component(name, component);
});
