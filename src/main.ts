import { createApp } from "vue";
import { createPinia } from "pinia";
import { createRouter, createWebHistory } from "vue-router";
import App from "./App.vue";
import "./styles/global.css";
import { isMobile } from "./utils/platform";

const mobileRoutes = [
  {
    path: "/",
    component: () => import("./components/mobile/MobileLayout.vue"),
    children: [
      { path: "", redirect: "/devices" },
      {
        path: "devices",
        name: "mobile-devices",
        component: () => import("./components/mobile/MobileDeviceList.vue"),
      },
      {
        path: "snippets",
        name: "mobile-snippets",
        component: () => import("./components/mobile/MobileSnippetList.vue"),
      },
      {
        path: "settings",
        name: "mobile-settings",
        component: () => import("./components/mobile/MobileSettings.vue"),
      },
    ],
  },
  {
    path: "/chat/:deviceId",
    name: "mobile-chat",
    component: () => import("./components/mobile/MobileChatView.vue"),
  },
  {
    path: "/snippet/:id",
    name: "mobile-snippet-edit",
    component: () => import("./components/mobile/MobileSnippetEdit.vue"),
  },
];

const desktopRoutes = [
  {
    path: "/",
    name: "main",
    component: () => import("./components/layout/AppLayout.vue"),
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes: isMobile() ? mobileRoutes : desktopRoutes,
});

const app = createApp(App);
app.use(createPinia());
app.use(router);
app.mount("#app");
