<script setup lang="ts">
import { computed, reactive, watch } from "vue";
import type { Contact, TaskForm as TaskFormType, TaskItem } from "../types";

const props = defineProps<{
  contacts: Contact[];
  task?: TaskItem | null;
  saving?: boolean;
}>();

const isNonNegativeNumber = (value: string) => {
  if (!value.trim()) return true;
  const parsed = Number(value);
  return Number.isFinite(parsed) && parsed >= 0;
};

const isNonNegativeInteger = (value: string) => {
  if (!value.trim()) return true;
  const parsed = Number(value);
  return Number.isInteger(parsed) && parsed >= 0;
};

const emit = defineEmits<{
  save: [payload: TaskFormType];
  cancel: [];
}>();

const emptyForm = (): TaskFormType => ({
  title: "",
  contact_id: null,
  received_date: "",
  completed_date: "",
  fee: "",
  print_fee: "",
  settled_amount: "",
  is_settled: false,
  is_completed: false,
  page_count: "",
  word_count: ""
});

const form = reactive<TaskFormType>(emptyForm());

watch(
  () => props.task,
  (task) => {
    Object.assign(form, emptyForm());
    if (!task) return;
    Object.assign(form, {
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
      word_count: task.word_count?.toString() ?? ""
    });
  },
  { immediate: true }
);

watch(
  () => form.is_completed,
  (value) => {
    if (value) {
      if (!form.completed_date) {
        form.completed_date = new Date().toISOString().slice(0, 10);
      }
      return;
    }
    if (!form.is_settled) {
      form.completed_date = "";
    }
  }
);

watch(
  () => form.is_settled,
  (value) => {
    if (value) {
      form.is_completed = true;
      if (!form.completed_date) {
        form.completed_date = new Date().toISOString().slice(0, 10);
      }
    } else {
      form.settled_amount = "";
    }
  }
);

const submitText = computed(() => (form.id ? "保存修改" : "创建任务"));
const modeText = computed(() => (form.id ? "正在编辑已有任务" : "录入新的编辑任务"));
const isEditing = computed(() => Boolean(form.id));
const completedSteps = computed(
  () =>
    [
      form.title.trim(),
      form.contact_id,
      form.received_date,
      form.fee || form.page_count || form.word_count,
      form.is_completed || form.is_settled
    ].filter(Boolean).length
);

const validationMessage = computed(() => {
  if (!form.title.trim()) return "请先填写编辑书名。";
  if (!form.contact_id) return "请选择对接人。";
  if (!form.received_date) return "请选择接单日期。";
  if (form.is_completed && !form.completed_date) {
    return "标记为已完成后，请补上完成日期。";
  }
  if (form.completed_date && form.completed_date < form.received_date) {
    return "完成日期不能早于接单日期。";
  }
  if (form.is_settled && !form.settled_amount.trim()) {
    return "已结账时，请填写结账金额。";
  }
  if (!isNonNegativeNumber(form.fee)) return "预计费用必须是大于等于 0 的数字。";
  if (!isNonNegativeNumber(form.print_fee)) return "打印费用必须是大于等于 0 的数字。";
  if (!isNonNegativeNumber(form.settled_amount)) return "结账金额必须是大于等于 0 的数字。";
  if (!isNonNegativeInteger(form.page_count)) return "页码必须是大于等于 0 的整数。";
  if (!isNonNegativeInteger(form.word_count)) return "字数必须是大于等于 0 的整数。";
  return "";
});

const canSubmit = computed(() => !validationMessage.value);

const submit = () => {
  if (!canSubmit.value) return;
  emit("save", { ...form });
};

const reset = () => {
  Object.assign(form, emptyForm());
  emit("cancel");
};
</script>

<template>
  <section class="panel task-form-panel">
    <div class="task-form-banner">
      <div class="task-form-copy">
        <div class="eyebrow">任务录入</div>
        <h2>{{ submitText }}</h2>
        <p>{{ modeText }}</p>
        <div class="task-form-progress">
          <div class="task-form-progress-bar">
            <span :style="{ width: `${(completedSteps / 5) * 100}%` }"></span>
          </div>
          <small>录入完成度 {{ completedSteps }}/5</small>
        </div>
      </div>
      <div class="toolbar">
        <button class="btn btn-secondary" :disabled="props.saving" @click="reset">
          {{ isEditing ? "取消编辑" : "清空表单" }}
        </button>
        <button class="btn btn-primary" :disabled="!canSubmit || props.saving" @click="submit">
          {{ props.saving ? "保存中..." : submitText }}
        </button>
      </div>
    </div>
    <p class="form-feedback" :class="{ 'is-error': Boolean(validationMessage) }">
      {{ validationMessage || "基础信息完整后就可以先保存，其他字段后面随时补。" }}
    </p>

    <div class="task-form-grid">
      <section class="task-section">
        <div class="section-title">基础信息</div>
        <p class="section-desc">先把任务主体记下来，后面金额和状态可以慢慢补。</p>
        <div class="grid grid-2">
          <label class="field field-wide">
            <span>编辑书名</span>
            <input v-model="form.title" placeholder="例如：儿童文学精选集" />
          </label>
          <label class="field">
            <span>对接人</span>
            <select v-model.number="form.contact_id">
              <option :value="null">请选择对接人</option>
              <option v-for="contact in contacts" :key="contact.id" :value="contact.id">
                {{ contact.name }}
              </option>
            </select>
          </label>
          <label class="field">
            <span>接单日期</span>
            <input v-model="form.received_date" type="date" />
          </label>
        </div>
      </section>

      <section class="task-section">
        <div class="section-title">任务规模</div>
        <p class="section-desc">页码和字数不是必须，但后面回顾工作量时会很有帮助。</p>
        <div class="grid grid-2">
          <label class="field">
            <span>页码</span>
            <input v-model="form.page_count" type="number" min="0" placeholder="可不填" />
          </label>
          <label class="field">
            <span>字数</span>
            <input v-model="form.word_count" type="number" min="0" placeholder="可不填" />
          </label>
        </div>
      </section>

      <section class="task-section">
        <div class="section-title">结算信息</div>
        <p class="section-desc">如果暂时还没谈好报酬，也可以先空着，后续再补。</p>
        <div class="grid grid-2">
          <label class="field">
            <span>预计费用</span>
            <input v-model="form.fee" type="number" min="0" step="0.01" placeholder="可不填" />
          </label>
          <label class="field">
            <span>打印费用</span>
            <input v-model="form.print_fee" type="number" min="0" step="0.01" placeholder="可不填" />
          </label>
          <label class="field">
            <span>结账金额</span>
            <input
              v-model="form.settled_amount"
              :disabled="!form.is_settled"
              type="number"
              min="0"
              step="0.01"
              :placeholder="form.is_settled ? '请输入已收金额' : '结账后可填写'"
            />
          </label>
        </div>
      </section>

      <section class="task-section">
        <div class="section-title">状态设置</div>
        <p class="section-desc">状态会影响下方三分栏的展示位置，完成日期也会在这里一起维护。</p>
        <div class="status-switches">
          <button
            class="switch-chip"
            :class="{ active: form.is_completed }"
            type="button"
            @click="form.is_completed = !form.is_completed"
          >
            已完成
          </button>
          <button
            class="switch-chip"
            :class="{ active: form.is_settled }"
            type="button"
            @click="form.is_settled = !form.is_settled"
          >
            已结账
          </button>
        </div>
        <label v-if="form.is_completed" class="field status-date-field">
          <span>完成日期</span>
          <input v-model="form.completed_date" type="date" />
        </label>
        <p class="task-form-tip">勾选“已结账”后会自动同步为已完成。</p>
      </section>
    </div>
  </section>
</template>
