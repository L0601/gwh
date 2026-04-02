<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from "vue";
import { open, save } from "@tauri-apps/plugin-dialog";
import TaskForm from "../components/TaskForm.vue";
import { exportTasks, importTasks } from "../api";
import { useTaskStore } from "../stores/task";
import type { TaskForm as TaskFormPayload, TaskItem } from "../types";

const store = useTaskStore();
const editingTask = ref<TaskItem | null>(null);
const notice = ref("");
const saving = ref(false);
const highlightedTaskId = ref<number | null>(null);
const drawerVisible = ref(false);

const statusMeta = {
  IN_PROGRESS: { title: "进行中", accent: "warm" },
  COMPLETED_UNSETTLED: { title: "完成未结账", accent: "sky" },
  SETTLED: { title: "已结账", accent: "green" }
} as const;

onMounted(() => {
  store.bootstrap();
});

const groupedTasks = computed(() => ({
  IN_PROGRESS: store.tasks.filter((task) => task.status === "IN_PROGRESS"),
  COMPLETED_UNSETTLED: store.tasks.filter((task) => task.status === "COMPLETED_UNSETTLED"),
  SETTLED: store.tasks.filter((task) => task.status === "SETTLED")
}));

const overviewCards = computed(() => [
  {
    title: "当前进行中",
    value: groupedTasks.value.IN_PROGRESS.length,
    detail: "手头仍在推进的任务",
    tone: "warm"
  },
  {
    title: "待结账",
    value: groupedTasks.value.COMPLETED_UNSETTLED.length,
    detail: "已经完成、可继续跟进回款",
    tone: "sky"
  },
  {
    title: "已结账",
    value: groupedTasks.value.SETTLED.length,
    detail: "已经完成闭环的任务",
    tone: "green"
  }
]);

const saveTask = async (payload: TaskFormPayload) => {
  saving.value = true;
  try {
    const savedTask = await store.submitTask(payload);
    if (!savedTask) {
      notice.value = payload.id ? "任务已保存，列表已刷新。" : "任务已创建，列表已刷新。";
      closeDrawer();
      highlightedTaskId.value = null;
      return;
    }

    if (store.filters.status && store.filters.status !== savedTask.status) {
      store.filters.status = "";
      await store.fetchTasks();
      notice.value = `任务已保存，状态变更为“${statusLabel(savedTask.status)}”，已切回全部状态方便您查看。`;
    } else {
      notice.value = payload.id ? "任务已更新。" : "任务已创建。";
    }
    closeDrawer();
    highlightedTaskId.value = savedTask.id;
    await nextTick();
    scrollToTask(savedTask.id);
  } catch (error) {
    notice.value = error instanceof Error ? error.message : "保存失败，请再试一次。";
  } finally {
    saving.value = false;
  }
};

const chooseImportFile = async () => {
  const selected = await open({
    multiple: false,
    filters: [{ name: "CSV", extensions: ["csv"] }]
  });
  if (!selected || Array.isArray(selected)) return;
  try {
    const count = await importTasks(selected);
    await store.fetchTasks();
    await store.refreshContacts();
    notice.value = `导入完成，共导入 ${count} 条记录。`;
  } catch (error) {
    notice.value = getErrorMessage(error, "导入失败，请检查文件内容。");
  }
};

const chooseExportFile = async () => {
  try {
    const selected = await save({
      defaultPath: "编辑任务导出.csv",
      filters: [{ name: "CSV", extensions: ["csv"] }]
    });
    if (!selected) return;
    await exportTasks(selected);
    notice.value = `导出完成：${selected}`;
  } catch (error) {
    notice.value = getErrorMessage(error, "导出失败，请稍后再试。");
  }
};

const chooseTemplateFile = async () => {
  try {
    const rows = [
      [
        "title",
        "contact_name",
        "received_date",
        "completed_date",
        "fee",
        "print_fee",
        "settled_amount",
        "is_settled",
        "is_completed",
        "page_count",
        "word_count"
      ],
      [
        "示例书稿",
        "示例对接人",
        "2026-04-01",
        "2026-04-08",
        "300",
        "20",
        "320",
        "true",
        "true",
        "120",
        "80000"
      ]
    ];
    downloadCsv("编辑任务导入模板.csv", rows);
    notice.value = "导入模板已下载到系统默认下载目录，通常在“下载/Downloads”文件夹里。";
  } catch (error) {
    notice.value = getErrorMessage(error, "模板生成失败，请稍后再试。");
  }
};

const editTask = (task: TaskItem) => {
  editingTask.value = task;
  drawerVisible.value = true;
  notice.value = `正在编辑《${task.title}》。`;
  highlightedTaskId.value = task.id;
};

const closeDrawer = () => {
  drawerVisible.value = false;
  editingTask.value = null;
};

const openCreateDrawer = () => {
  editingTask.value = null;
  drawerVisible.value = true;
  notice.value = "开始录入新任务。";
};

const statusLabel = (status: TaskItem["status"]) => statusMeta[status].title;

const scrollToTask = (taskId: number) => {
  const target = document.getElementById(`task-card-${taskId}`);
  target?.scrollIntoView({ behavior: "smooth", block: "center" });
};

const toTaskPayload = (task: TaskItem, patch: Partial<TaskFormPayload> = {}): TaskFormPayload => ({
  id: task.id,
  title: task.title,
  contact_id: task.contact_id,
  received_date: task.received_date,
  completed_date: task.completed_date ?? "",
  fee: task.fee?.toString() ?? "",
  print_fee: task.print_fee?.toString() ?? "",
  settled_amount: task.settled_amount?.toString() ?? "",
  is_settled: task.is_settled,
  is_completed: task.is_completed,
  page_count: task.page_count?.toString() ?? "",
  word_count: task.word_count?.toString() ?? "",
  ...patch
});

const markTaskCompleted = async (task: TaskItem) => {
  await saveTask(
    toTaskPayload(task, {
      is_completed: true,
      completed_date: task.completed_date ?? new Date().toISOString().slice(0, 10)
    })
  );
};

const markTaskSettled = async (task: TaskItem) => {
  await saveTask(
    toTaskPayload(task, {
      is_completed: true,
      is_settled: true,
      completed_date: task.completed_date ?? new Date().toISOString().slice(0, 10),
      settled_amount: task.settled_amount?.toString() || task.fee?.toString() || ""
    })
  );
};

const removeTask = async (task: TaskItem) => {
  if (!window.confirm(`确认删除《${task.title}》吗？删除后将无法恢复。`)) {
    return;
  }
  try {
    await store.removeTask(task.id);
    if (editingTask.value?.id === task.id) {
      closeDrawer();
    }
    highlightedTaskId.value = null;
    notice.value = "任务已删除。";
  } catch (error) {
    notice.value = getErrorMessage(error, "删除失败，请稍后再试。");
  }
};

const downloadCsv = (filename: string, rows: string[][]) => {
  const csv = rows
    .map((row) => row.map((value) => `"${value.split("\"").join("\"\"")}"`).join(","))
    .join("\n");
  const blob = new Blob([`\uFEFF${csv}`], { type: "text/csv;charset=utf-8;" });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = filename;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
};

const getErrorMessage = (error: unknown, fallback: string) => {
  if (error instanceof Error) return error.message;
  if (typeof error === "string") return error;
  if (error && typeof error === "object") {
    return JSON.stringify(error);
  }
  return fallback;
};
</script>

<template>
  <div class="page-header">
    <div>
      <div class="eyebrow">任务中心</div>
      <h2>任务管理</h2>
      <p>把正在做的、待回款的、已结账的任务分开看，日常处理会更清楚。</p>
    </div>
    <div class="toolbar">
      <button class="btn btn-primary" @click="openCreateDrawer">新建任务</button>
      <button class="btn btn-secondary" @click="chooseTemplateFile">下载模板</button>
      <button class="btn btn-secondary" @click="chooseImportFile">导入 CSV</button>
      <button class="btn btn-secondary" @click="chooseExportFile">导出 CSV</button>
    </div>
  </div>

  <section class="overview-grid">
    <article
      v-for="card in overviewCards"
      :key="card.title"
      class="overview-card"
      :data-tone="card.tone"
    >
      <div class="overview-label">{{ card.title }}</div>
      <div class="overview-value">{{ card.value }}</div>
      <div class="overview-detail">{{ card.detail }}</div>
    </article>
  </section>

  <section class="panel filter-panel">
    <div class="toolbar">
      <label class="field">
        <span>只看状态</span>
        <select v-model="store.filters.status" @change="store.fetchTasks()">
          <option value="">全部状态</option>
          <option value="IN_PROGRESS">进行中</option>
          <option value="COMPLETED_UNSETTLED">完成未结账</option>
          <option value="SETTLED">已结账</option>
        </select>
      </label>
      <label class="field">
        <span>对接人</span>
        <select v-model="store.filters.contact_id" @change="store.fetchTasks()">
          <option value="">全部对接人</option>
          <option v-for="contact in store.contacts" :key="contact.id" :value="String(contact.id)">
            {{ contact.name }}
          </option>
        </select>
      </label>
      <label class="field">
        <span>开始日期</span>
        <input v-model="store.filters.start_date" type="date" @change="store.fetchTasks()" />
      </label>
      <label class="field">
        <span>结束日期</span>
        <input v-model="store.filters.end_date" type="date" @change="store.fetchTasks()" />
      </label>
    </div>
    <p v-if="notice" class="notice-text">{{ notice }}</p>
  </section>

  <section class="kanban-board">
    <article
      v-for="(tasks, status) in groupedTasks"
      :key="status"
      class="kanban-column panel"
      :data-accent="statusMeta[status as keyof typeof statusMeta].accent"
    >
      <div class="kanban-header">
        <div>
          <h3>{{ statusMeta[status as keyof typeof statusMeta].title }}</h3>
          <p>{{ tasks.length }} 条任务</p>
        </div>
      </div>

      <div v-if="tasks.length" class="kanban-list">
        <article
          v-for="task in tasks"
          :id="`task-card-${task.id}`"
          :key="task.id"
          class="task-card"
          :class="{ 'task-card-highlight': highlightedTaskId === task.id }"
        >
          <div class="task-card-top">
            <h4>{{ task.title }}</h4>
            <span class="tag">{{ statusLabel(task.status) }}</span>
          </div>
          <dl class="task-meta">
            <div>
              <dt>对接人</dt>
              <dd>{{ task.contact_name }}</dd>
            </div>
            <div>
              <dt>接单</dt>
              <dd>{{ task.received_date }}</dd>
            </div>
            <div>
              <dt>预计费用</dt>
              <dd>{{ task.fee ?? "-" }}</dd>
            </div>
            <div>
              <dt>打印费用</dt>
              <dd>{{ task.print_fee ?? "-" }}</dd>
            </div>
            <div>
              <dt>已收</dt>
              <dd>{{ task.settled_amount ?? "-" }}</dd>
            </div>
          </dl>
          <p class="task-card-note">
            {{
              task.status === "IN_PROGRESS"
                ? "还在处理中，可以继续补充规模和金额信息。"
                : task.status === "COMPLETED_UNSETTLED"
                  ? "任务已经完成，建议留意结账进度。"
                  : "这笔任务已经完成闭环，可留作收入记录。"
            }}
          </p>
          <div class="toolbar task-actions">
            <button
              v-if="task.status === 'IN_PROGRESS'"
              class="btn btn-secondary btn-compact"
              :disabled="saving"
              @click="markTaskCompleted(task)"
            >
              标记完成
            </button>
            <button
              v-if="task.status === 'COMPLETED_UNSETTLED'"
              class="btn btn-secondary btn-compact"
              :disabled="saving"
              @click="markTaskSettled(task)"
            >
              标记结账
            </button>
            <button class="btn btn-secondary btn-compact" @click="editTask(task)">编辑任务</button>
            <button class="btn btn-danger btn-compact" :disabled="saving" @click="removeTask(task)">删除</button>
          </div>
        </article>
      </div>
      <div v-else class="empty-state">
        {{
          status === "IN_PROGRESS"
            ? "先创建一条任务，正在进行的工作会先出现在这里。"
            : status === "COMPLETED_UNSETTLED"
              ? "完成后但还没结账的任务，会自动出现在这里。"
              : "已经结账的任务会沉淀在这里，方便后面回看收入。"
        }}
      </div>
    </article>
  </section>

  <div v-if="drawerVisible" class="drawer-mask" @click="closeDrawer"></div>
  <aside class="task-drawer" :class="{ open: drawerVisible }">
    <div class="task-drawer-header">
      <div>
        <div class="eyebrow">任务表单</div>
        <h3>{{ editingTask ? "编辑任务" : "新建任务" }}</h3>
      </div>
      <button class="btn btn-secondary btn-compact" @click="closeDrawer">返回任务页</button>
    </div>
    <TaskForm
      :contacts="store.contacts"
      :task="editingTask"
      :saving="saving"
      @save="saveTask"
      @cancel="closeDrawer"
    />
  </aside>
</template>
