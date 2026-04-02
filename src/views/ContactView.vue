<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { createContact, deleteContact } from "../api";
import { useTaskStore } from "../stores/task";

const store = useTaskStore();
const name = ref("");
const remark = ref("");
const notice = ref("");

onMounted(() => {
  store.refreshContacts();
});

const contactCards = computed(() => [
  {
    title: "对接人总数",
    value: store.contacts.length,
    detail: "已经维护在台账里的合作联系人",
    tone: "warm"
  },
  {
    title: "已写备注",
    value: store.contacts.filter((contact) => contact.remark?.trim()).length,
    detail: "带有出版社或关系说明，更方便后续辨认",
    tone: "sky"
  },
  {
    title: "待补信息",
    value: store.contacts.filter((contact) => !contact.remark?.trim()).length,
    detail: "当前没有备注信息的联系人",
    tone: "green"
  }
]);

const canSubmit = computed(() => Boolean(name.value.trim()));

const formatToMinute = (value: string) => value.slice(0, 16).replace("T", " ");

const submit = async () => {
  if (!canSubmit.value) return;
  await createContact(name.value, remark.value);
  name.value = "";
  remark.value = "";
  notice.value = "对接人已新增。";
  await store.refreshContacts();
};

const remove = async (id: number) => {
  if (!window.confirm("删除后将无法恢复。确认删除这个对接人吗？")) {
    return;
  }
  try {
    await deleteContact(id);
    notice.value = "对接人已删除。";
    await store.refreshContacts();
  } catch {
    notice.value = "该对接人下还有任务，暂时不能删除。";
  }
};
</script>

<template>
  <div class="page-header">
    <div>
      <div class="eyebrow">合作关系</div>
      <h2>对接人管理</h2>
      <p>把经常合作的出版社或联系人整理清楚，后面筛选任务和统计收入都会更顺手。</p>
    </div>
  </div>

  <section class="overview-grid">
    <article
      v-for="card in contactCards"
      :key="card.title"
      class="overview-card"
      :data-tone="card.tone"
    >
      <div class="overview-label">{{ card.title }}</div>
      <div class="overview-value">{{ card.value }}</div>
      <div class="overview-detail">{{ card.detail }}</div>
    </article>
  </section>

  <section class="panel contact-form-panel">
    <div class="section-head">
      <div>
        <div class="section-title">新增对接人</div>
        <p class="section-desc">姓名必填，备注按需要填写即可。</p>
      </div>
      <button class="btn btn-primary" :disabled="!canSubmit" @click="submit">新增对接人</button>
    </div>
    <p v-if="notice" class="notice-text">{{ notice }}</p>
    <div class="grid grid-2">
      <label class="field">
        <span>姓名</span>
        <input v-model="name" placeholder="例如：王老师" />
      </label>
      <label class="field">
        <span>备注</span>
        <input v-model="remark" placeholder="例如：xx 出版社少儿编辑" />
      </label>
    </div>
  </section>

  <section class="contact-list">
    <article v-for="contact in store.contacts" :key="contact.id" class="panel contact-card">
      <div class="contact-card-top">
        <div>
          <h3>{{ contact.name }}</h3>
          <p>{{ contact.remark || "暂未补充备注信息" }}</p>
        </div>
        <button class="btn btn-secondary" @click="remove(contact.id)">删除</button>
      </div>
      <div class="contact-meta">
        <div>
          <span>创建时间</span>
          <strong>{{ formatToMinute(contact.created_at) }}</strong>
        </div>
        <div>
          <span>状态</span>
          <strong>{{ contact.remark ? "已填写备注" : "未填写备注" }}</strong>
        </div>
      </div>
    </article>
    <div v-if="!store.contacts.length" class="empty-state empty-state-large">
      还没有对接人，可以先把常合作的出版社联系人录进去。
    </div>
  </section>
</template>
