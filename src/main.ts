import { createApp } from "vue";
import "./assets/main.css";
import App from "./App.vue";
import { openUrl } from '@tauri-apps/plugin-opener';

document.addEventListener('click', (e) => {
  const target = e.target as HTMLElement;
  const a = target.closest('a');
  if (a && a.href && (a.href.startsWith('http://') || a.href.startsWith('https://'))) {
    e.preventDefault();
    openUrl(a.href).catch(console.error);
  }
});

createApp(App).mount("#app");
