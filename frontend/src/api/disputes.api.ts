import { apiClient } from "./client";

export type DisputeStatus = "open" | "resolved" | "rejected" | "escalated";

export interface Dispute {
  id: string;
  order_id: string;
  customer_id: string;
  issue_type: string;
  description: string;
  evidence_urls: string[];
  status: DisputeStatus;
  resolution: string | null;
  refund_amount_cents: number | null;
  resolved_by: string | null;
  resolved_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateDisputeInput {
  issue_type: "missing_items" | "wrong_order" | "cold_food" | "late" | "other";
  description: string;
  evidence_urls?: string[];
}

export interface ResolveDisputeInput {
  resolution: "full_refund" | "partial_refund" | "reject";
  amount_cents?: number;
  notes?: string;
}

export const disputesApi = {
  create: async (orderId: string, input: CreateDisputeInput) => {
    const { data } = await apiClient.post<Dispute>(`/orders/${orderId}/dispute`, input);
    return data;
  },
  mine: async () => {
    const { data } = await apiClient.get<Dispute[]>("/disputes");
    return data;
  },
  listOpen: async () => {
    const { data } = await apiClient.get<Dispute[]>("/admin/disputes");
    return data;
  },
  resolve: async (id: string, input: ResolveDisputeInput) => {
    const { data } = await apiClient.post<{ dispute: Dispute; refunded: boolean }>(
      `/admin/disputes/${id}/resolve`,
      input,
    );
    return data;
  },
};
