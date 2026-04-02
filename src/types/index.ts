export interface Contact {
  id: number;
  name: string;
  remark: string | null;
  created_at: string;
}

export interface TaskItem {
  id: number;
  title: string;
  contact_id: number;
  contact_name: string;
  received_date: string;
  completed_date: string | null;
  fee: number | null;
  print_fee: number | null;
  settled_amount: number | null;
  is_settled: boolean;
  is_completed: boolean;
  page_count: number | null;
  word_count: number | null;
  status: "IN_PROGRESS" | "COMPLETED_UNSETTLED" | "SETTLED";
}

export interface TaskForm {
  id?: number;
  title: string;
  contact_id: number | null;
  received_date: string;
  completed_date: string;
  fee: string;
  print_fee: string;
  settled_amount: string;
  is_settled: boolean;
  is_completed: boolean;
  page_count: string;
  word_count: string;
}

export interface TaskFilters {
  status: string;
  contact_id: string;
  start_date: string;
  end_date: string;
}

export interface StatsSummary {
  task_count: number;
  settled_total: number;
}

export interface StatsPayload {
  summary: StatsSummary;
  tasks: TaskItem[];
}
