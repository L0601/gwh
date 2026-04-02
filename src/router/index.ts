import { createRouter, createWebHashHistory } from "vue-router";
import TaskView from "../views/TaskView.vue";
import ContactView from "../views/ContactView.vue";
import StatsView from "../views/StatsView.vue";

export default createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", component: TaskView },
    { path: "/contacts", component: ContactView },
    { path: "/stats", component: StatsView }
  ]
});
