<template>
  <div class="app-shell">
    <aside class="sidebar">
      <div class="sidebar-main">
        <h1>稿酬簿</h1>
        <RouterLink class="nav-link" to="/">任务管理</RouterLink>
        <RouterLink class="nav-link" to="/contacts">对接人</RouterLink>
        <RouterLink class="nav-link" to="/stats">统计报表</RouterLink>
      </div>
      <section v-if="quote" class="quote-panel">
        <p class="quote-label">今日箴言</p>
        <div class="quote-body">
          <span class="quote-mark">“</span>
          <p class="quote-text">{{ quote }}</p>
        </div>
      </section>
    </aside>
    <main class="content">
      <RouterView />
    </main>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";

const quote = ref("");
let timer: number | null = null;

async function refreshQuote() {
  try {
    const response = await fetch("https://v1.hitokoto.cn/", {
      headers: { Accept: "application/json" },
    });

    if (!response.ok) {
      quote.value = "";
      return;
    }

    const data = (await response.json()) as { hitokoto?: string };
    quote.value = data.hitokoto?.trim() ?? "";
  } catch {
    quote.value = "";
  }
}

onMounted(() => {
  refreshQuote();
  timer = window.setInterval(refreshQuote, 10 * 60 * 1000);
});

onUnmounted(() => {
  if (timer !== null) {
    window.clearInterval(timer);
  }
});
</script>
