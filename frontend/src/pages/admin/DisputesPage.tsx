import { useOpenDisputes, useResolveDispute } from "@/hooks/useAdmin";
import { Button } from "@/components/ui/Button";
import { AlertTriangle, CheckCircle2, XCircle } from "lucide-react";
import { useState } from "react";

export function DisputesPage() {
  const { data: disputes, isLoading } = useOpenDisputes();
  const resolve = useResolveDispute();
  const [partialAmounts, setPartialAmounts] = useState<Record<string, string>>({});

  if (isLoading) return <div className="p-8 text-center text-gray-500">Loading…</div>;

  if (!disputes || disputes.length === 0) {
    return (
      <div className="mx-auto max-w-2xl px-4 py-12 text-center">
        <CheckCircle2 size={48} className="mx-auto text-green-300" />
        <h1 className="mt-3 font-display text-xl font-semibold">No open disputes</h1>
        <p className="mt-1 text-sm text-gray-500">
          Customer complaints will appear here for review.
        </p>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-3xl px-4 py-8">
      <h1 className="font-display text-2xl font-bold mb-6">Open Disputes</h1>
      <div className="space-y-3">
        {disputes.map((d) => (
          <div key={d.id} className="card p-4">
            <div className="flex items-start justify-between gap-3">
              <div>
                <div className="flex items-center gap-2">
                  <AlertTriangle size={14} className="text-amber-500" />
                  <span className="font-medium capitalize">{d.issue_type.replace(/_/g, " ")}</span>
                  <span className="text-xs text-gray-500">· order {d.order_id.slice(0, 8)}</span>
                </div>
                <p className="mt-2 text-sm text-gray-700">{d.description}</p>
                <p className="mt-1 text-xs text-gray-500">
                  Filed {new Date(d.created_at).toLocaleString()}
                </p>
                {d.evidence_urls.length > 0 && (
                  <div className="mt-2 flex gap-2">
                    {d.evidence_urls.slice(0, 3).map((url, i) => (
                      <img
                        key={i}
                        src={url}
                        alt={`Evidence ${i + 1}`}
                        className="h-16 w-16 rounded object-cover"
                      />
                    ))}
                  </div>
                )}
              </div>
            </div>

            {/* Resolution actions */}
            <div className="mt-4 border-t border-gray-100 pt-3 flex flex-wrap items-center gap-2">
              <Button
                size="sm"
                onClick={() => resolve.mutate({ id: d.id, resolution: "full_refund" })}
                disabled={resolve.isPending}
              >
                <CheckCircle2 size={14} /> Full refund
              </Button>

              <div className="flex items-center gap-1">
                <input
                  type="number"
                  placeholder="$"
                  className="input w-16 px-2 py-1 text-xs"
                  value={partialAmounts[d.id] ?? ""}
                  onChange={(e) =>
                    setPartialAmounts((s) => ({ ...s, [d.id]: e.target.value }))
                  }
                />
                <Button
                  size="sm"
                  variant="secondary"
                  onClick={() => {
                    const amt = parseFloat(partialAmounts[d.id] ?? "0");
                    if (amt > 0) {
                      resolve.mutate({
                        id: d.id,
                        resolution: "partial_refund",
                        amount_cents: Math.round(amt * 100),
                      });
                    }
                  }}
                  disabled={resolve.isPending}
                >
                  Partial refund
                </Button>
              </div>

              <Button
                size="sm"
                variant="ghost"
                onClick={() => {
                  if (confirm("Reject this dispute? Customer will be notified.")) {
                    resolve.mutate({ id: d.id, resolution: "reject" });
                  }
                }}
                disabled={resolve.isPending}
              >
                <XCircle size={14} /> Reject
              </Button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
