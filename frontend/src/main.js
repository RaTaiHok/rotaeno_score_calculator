import { createApp } from "vue";
import { ElButton, ElCheckbox, ElInputNumber, ElOption, ElSelect } from "element-plus";
import "element-plus/es/components/button/style/css";
import "element-plus/es/components/checkbox/style/css";
import "element-plus/es/components/input-number/style/css";
import "element-plus/es/components/option/style/css";
import "element-plus/es/components/select/style/css";
import App from "./App.vue";
import "./style.css";

createApp(App)
  .use(ElButton)
  .use(ElCheckbox)
  .use(ElInputNumber)
  .use(ElOption)
  .use(ElSelect)
  .mount("#app");
