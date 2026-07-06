import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { apiClient } from "@/api/client";
import { Button } from "@/components/ui/Button";
import { ShieldCheck, CheckCircle2, XCircle, FileText } from "lucide-react";

interface Verification {
  id: string;
  restaurant_id: string;
  restaurant_name: string;
  status: string;
  documents: Record<string, unknown>;
  reviewed_by: string | null;
  reviewed_at: string | null;
  notes: string | null;
  created_at: string;
}

export function VerificationsPage() {
  const qc = useQueryClient();
  const { data: verifications, isLoading } = useQuery({
    queryKey: ["admin", "verifications"],
    queryFn: async () => {
      const { data } = await apiClient.get<Verification[]>("/admin/verifications");
      return data;
    },
    refetchInterval: 30_000,
  });

  const approve = useMutation({
    mutationFn: (id: string) =>
      apiClient.post(`/admin/verifications/${id}/approve`),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["admin", "verifications"] });
      toast.success("Restaurant approved");
    },
    onError: () => toast.error("Failed to approve"),
  });

  const reject = useMutation({
    mutationFn: ({ id, notes }: { id: string; notes: string }) =>
      apiClient.post(`/admin/verifications/${id}/reject`, { notes }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["admin", "verifications"] });
      toast.success("Restaurant rejected");
    },
    onError: () => toast.error("Failed to reject"),
  });

  if (isLoading) return <div className="p-8 text-center text-gray-500">Loading…</div>;

  if (!verifications || verifications.length === 0) {
    return (
      <div className="mx-auto max-w-2xl px-4 py-12 text-center">
        <ShieldCheck size={48} className="mx-auto text-green-300" />
        <h1 className="mt-3 font-display text-xl font-semibold">No pending verifications</h1>
        <p className="mt-1 text-sm text-gray-500">
          New restaurant applications will appear here for review.
        </p>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-3xl px-4 py-8">
      <h1 className="font-display text-2xl font-bold mb-6">
        Restaurant Verifications ({verifications.length})
      </h1>
      <div className="space-y-3">
        {verifications.map((v) => (
          <div key={v.id} className="card p-4">
            <div className="flex items-start justify-between gap-3">
              <div className="flex-1">
                <div className="font-semibold">{v.restaurant_name}</div>
                <div className="text-xs text-gray-500">
                  Applied {new Date(v.created_at).toLocaleString()}
                </div>

                {/* Documents */}
                <div className="mt-3">
                  <div className="text-xs font-medium text-gray-500 flex items-center gap-1">
                    <FileText size={12} /> Submitted documents
                  </div>
                  <div className="mt-1 flex flex-wrap gap-2">
                    {Object.keys(v.documents).map((key) => (
                      <span
                        key={key}
                        className="rounded-full bg-gray-100 px-2 py-0.5 text-xs text-gray-700"
                      >
                        {key}
                      </span>
                    ))}
                    {Object.keys(v.documents).length === 0 && (
                      <span className="text-xs text-gray-400 italic">No documents</span>
                    )}
                  </div>
                </div>
              </div>

              {/* Actions */}
              <div className="flex flex-col gap-2 shrink-0">
                <Button
                  size="sm"
                  onClick={() => approve.mutate(v.id)}
                  disabled={approve.isPending}
                >
                  <CheckCircle2 size={14} /> Approve
                </Button>
                <Button
                  size="sm"
                  variant="secondary"
                  onClick={() => {
                    const notes = prompt("Reason for rejection:");
                    if (notes) reject.mutate({ id: v.id, notes });
                  }}
                  disabled={reject.isPending}
                >
                  <XCircle size={14} /> Reject
                </Button>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
