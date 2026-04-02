import { defineStore } from "pinia";
import { deleteTask, getStats, listContacts, listTasks, saveTask } from "../api";
import type { Contact, StatsPayload, TaskFilters, TaskForm, TaskItem } from "../types";

const currentMonth = () => {
  const now = new Date();
  const end = now.toISOString().slice(0, 10);
  const start = new Date(now.getFullYear(), now.getMonth(), 1).toISOString().slice(0, 10);
  return { start, end };
};

export const useTaskStore = defineStore("task", {
  state: () => ({
    contacts: [] as Contact[],
    tasks: [] as TaskItem[],
    stats: null as StatsPayload | null,
    filters: {
      status: "",
      contact_id: "",
      start_date: "",
      end_date: ""
    } as TaskFilters
  }),
  actions: {
    async bootstrap() {
      this.contacts = await listContacts();
      await this.fetchTasks();
    },
    async fetchTasks() {
      this.tasks = await listTasks(this.filters);
    },
    async refreshContacts() {
      this.contacts = await listContacts();
    },
    async submitTask(payload: TaskForm) {
      const task = await saveTask(payload);
      await this.fetchTasks();
      if (task) {
        return task;
      }

      if (payload.id) {
        return this.tasks.find((item) => item.id === payload.id) ?? null;
      }

      return (
        this.tasks.find(
          (item) =>
            item.title === payload.title &&
            item.contact_id === payload.contact_id &&
            item.received_date === payload.received_date
        ) ?? null
      );
    },
    async removeTask(id: number) {
      await deleteTask(id);
      await this.fetchTasks();
    },
    async fetchStats(contactId: string) {
      const range = currentMonth();
      this.stats = await getStats({
        contact_id: contactId,
        start_date: this.filters.start_date || range.start,
        end_date: this.filters.end_date || range.end
      });
    }
  }
});
