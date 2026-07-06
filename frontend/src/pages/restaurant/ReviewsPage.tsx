import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { Star, MessageSquare } from "lucide-react";
import { apiClient } from "@/api/client";
import { Button } from "@/components/ui/Button";
import { useState } from "react";
import type { Review } from "@/api/reviews.api";

export function ReviewsPage() {
  const qc = useQueryClient();
  const { data: reviews, isLoading } = useQuery({
    queryKey: ["restaurant", "reviews"],
    queryFn: async () => {
      const { data } = await apiClient.get<Review[]>("/restaurant/reviews");
      return data;
    },
  });
  const [replyingId, setReplyingId] = useState<string | null>(null);
  const [replyText, setReplyText] = useState("");

  const replyMutation = useMutation({
    mutationFn: async ({ id, reply }: { id: string; reply: string }) => {
      const { data } = await apiClient.post<Review>(`/reviews/${id}/reply`, { reply });
      return data;
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["restaurant", "reviews"] });
      toast.success("Reply posted");
      setReplyingId(null);
      setReplyText("");
    },
    onError: () => toast.error("Failed to post reply"),
  });

  if (isLoading) return <div className="p-8 text-center text-gray-500">Loading…</div>;

  if (!reviews || reviews.length === 0) {
    return (
      <div className="mx-auto max-w-2xl px-4 py-12 text-center">
        <MessageSquare size={48} className="mx-auto text-gray-300" />
        <h1 className="mt-3 font-display text-xl font-semibold">No reviews yet</h1>
        <p className="mt-1 text-sm text-gray-500">
          Once customers start reviewing your restaurant, their feedback will appear here.
        </p>
      </div>
    );
  }

  const avgRating =
    reviews.reduce((sum, r) => sum + r.rating_overall, 0) / reviews.length;

  return (
    <div className="mx-auto max-w-3xl px-4 py-8">
      <div className="flex items-center justify-between mb-6">
        <h1 className="font-display text-2xl font-bold">Reviews</h1>
        <div className="flex items-center gap-2">
          <Star size={20} fill="currentColor" className="text-amber-500" />
          <span className="font-semibold">{avgRating.toFixed(1)}</span>
          <span className="text-sm text-gray-500">({reviews.length} reviews)</span>
        </div>
      </div>

      <div className="space-y-3">
        {reviews.map((r) => (
          <div key={r.id} className="card p-4">
            <div className="flex items-start justify-between gap-3">
              <div>
                <div className="flex items-center gap-2">
                  <span className="flex items-center gap-0.5">
                    {Array.from({ length: 5 }).map((_, i) => (
                      <Star
                        key={i}
                        size={12}
                        fill={i < r.rating_overall ? "currentColor" : "none"}
                        className={
                          i < r.rating_overall ? "text-amber-500" : "text-gray-300"
                        }
                      />
                    ))}
                  </span>
                  <span className="text-xs text-gray-500">
                    {new Date(r.created_at).toLocaleDateString()}
                  </span>
                </div>
                {r.body && <p className="mt-2 text-sm text-gray-700">{r.body}</p>}
              </div>
            </div>

            {/* Dimensions */}
            <div className="mt-3 grid grid-cols-3 gap-2 text-xs">
              <Dim label="Food" value={r.rating_food} />
              <Dim label="Delivery" value={r.rating_delivery} />
              <Dim label="Packaging" value={r.rating_packaging} />
            </div>

            {/* Reply */}
            {r.reply_body ? (
              <div className="mt-3 border-l-2 border-brand-200 pl-3 text-sm">
                <div className="text-xs font-medium text-gray-500">Your reply</div>
                <p className="mt-0.5 text-gray-700">{r.reply_body}</p>
              </div>
            ) : replyingId === r.id ? (
              <div className="mt-3">
                <textarea
                  className="input min-h-[60px] resize-none"
                  placeholder="Write a public reply…"
                  value={replyText}
                  onChange={(e) => setReplyText(e.target.value)}
                />
                <div className="mt-2 flex gap-2">
                  <Button
                    size="sm"
                    onClick={() =>
                      replyMutation.mutate({ id: r.id, reply: replyText.trim() })
                    }
                    disabled={!replyText.trim() || replyMutation.isPending}
                  >
                    Post reply
                  </Button>
                  <Button
                    size="sm"
                    variant="ghost"
                    onClick={() => {
                      setReplyingId(null);
                      setReplyText("");
                    }}
                  >
                    Cancel
                  </Button>
                </div>
              </div>
            ) : (
              <button
                onClick={() => {
                  setReplyingId(r.id);
                  setReplyText("");
                }}
                className="mt-3 text-sm text-brand-600 hover:underline"
              >
                Reply
              </button>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}

function Dim({ label, value }: { label: string; value: number }) {
  return (
    <div className="rounded bg-gray-50 p-2 text-center">
      <div className="text-gray-500">{label}</div>
      <div className="font-semibold">{value}/5</div>
    </div>
  );
}
