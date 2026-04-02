import { invoke } from "@tauri-apps/api/core";
import type { Contact, StatsPayload, TaskFilters, TaskForm, TaskItem } from "./types";

export const listContacts = () => invoke<Contact[]>("list_contacts");

export const createContact = (name: string, remark: string) =>
  invoke<Contact>("create_contact", { payload: { name, remark } });

export const deleteContact = (id: number) => invoke<void>("delete_contact", { id });
export const deleteTask = (id: number) => invoke<void>("delete_task", { id });

export const listTasks = (filters: TaskFilters) =>
  invoke<TaskItem[]>("list_tasks", { filters });

export const saveTask = (payload: TaskForm) => invoke<TaskItem>("save_task", { payload });

export const getStats = (filters: Omit<TaskFilters, "status">) =>
  invoke<StatsPayload>("stats_tasks", { filters });

export const importTasks = (path: string) => invoke<number>("import_tasks", { path });

export const exportTasks = (path: string) => invoke<number>("export_tasks", { path });
