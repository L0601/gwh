<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useTaskStore } from "../stores/task";
import type { TaskItem } from "../types";

const store = useTaskStore();
const contactId = ref("");

const statusText: Record<TaskItem["status"], string> = {
  IN_PROGRESS: "进行中",
  COMPLETED_UNSETTLED: "完成未结账",
  SETTLED: "已结账"
};

onMounted(async () => {
  await store.refreshContacts();
  await store.fetchStats("");
});

const refresh = async () => {
  await store.fetchStats(contactId.value);
};

const statsCards = computed(() => {
  const tasks = store.stats?.tasks || [];
  const unsettledCount = tasks.filter((task) => task.status === "COMPLETED_UNSETTLED").length;
  return [
    {
      title: "接单数目",
      value: store.stats?.summary.task_count ?? 0,
      detail: "当前筛选条件下共记录了多少条任务",
      tone: "warm"
    },
    {
      title: "累计已收金额",
      value: store.stats?.summary.settled_total ?? 0,
      detail: "已经实际结账入账的金额汇总",
      tone: "green"
    },
    {
      title: "待回款任务",
      value: unsettledCount,
      detail: "已经完成但还没结账，适合重点跟进",
      tone: "sky"
    }
  ];
});
</script>

<template>
  <div class="page-header">
    <div>
      <div class="eyebrow">收入回顾</div>
      <h2>统计报表</h2>
      <p>按时间范围和对接人回看接单量、已收金额，以及仍待跟进的任务。</p>
    </div>
  </div>

  <section class="panel stats-filter-panel">
    <div class="section-head">
      <div>
        <div class="section-title">筛选条件</div>
        <p class="section-desc">默认适合回看最近一段时间的工作记录，也可以单独查看某位对接人的合作情况。</p>
      </div>
      <button class="btn btn-primary" @click="refresh">刷新统计</button>
    </div>
    <div class="toolbar">
      <label class="field">
        <span>开始日期</span>
        <input v-model="store.filters.start_date" type="date" />
      </label>
      <label class="field">
        <span>结束日期</span>
        <input v-model="store.filters.end_date" type="date" />
      </label>
      <label class="field">
        <span>对接人</span>
        <select v-model="contactId">
          <option value="">全部对接人</option>
          <option v-for="contact in store.contacts" :key="contact.id" :value="String(contact.id)">
            {{ contact.name }}
          </option>
        </select>
      </label>
    </div>
  </section>

  <section class="overview-grid">
    <article
      v-for="card in statsCards"
      :key="card.title"
      class="overview-card"
      :data-tone="card.tone"
    >
      <div class="overview-label">{{ card.title }}</div>
      <div class="overview-value">{{ card.value }}</div>
      <div class="overview-detail">{{ card.detail }}</div>
    </article>
  </section>

  <section class="panel stats-summary-panel">
    <div class="section-head">
      <div>
        <div class="section-title">接单详情</div>
        <p class="section-desc">每一条记录都可以帮助回看某段时间到底做了什么、钱有没有到账。</p>
      </div>
      <span class="tag">共 {{ store.stats?.tasks.length ?? 0 }} 条</span>
    </div>

    <div v-if="store.stats?.tasks.length" class="stats-task-list stats-task-window">
      <article v-for="task in store.stats?.tasks || []" :key="task.id" class="stats-task-card">
        <div class="stats-task-top">
          <div>
            <h3>{{ task.title }}</h3>
            <p>{{ task.contact_name }}</p>
          </div>
          <span class="tag">{{ statusText[task.status] }}</span>
        </div>
        <div class="stats-task-grid">
          <div>
            <span>接单日期</span>
            <strong>{{ task.received_date }}</strong>
          </div>
          <div>
            <span>结账金额</span>
            <strong>{{ task.settled_amount ?? "-" }}</strong>
          </div>
          <div>
            <span>预计费用</span>
            <strong>{{ task.fee ?? "-" }}</strong>
          </div>
          <div>
            <span>打印费用</span>
            <strong>{{ task.print_fee ?? "-" }}</strong>
          </div>
          <div>
            <span>任务状态</span>
            <strong>{{ statusText[task.status] }}</strong>
          </div>
        </div>
      </article>
    </div>

    <div v-else class="empty-state">当前筛选条件下还没有统计结果。</div>
  </section>
</template>
