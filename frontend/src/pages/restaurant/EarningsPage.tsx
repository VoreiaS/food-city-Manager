import { useQuery } from "@tanstack/react-query";
import { apiClient } from "@/api/client";
import { DollarSign, TrendingUp, Clock, Package } from "lucide-react";

interface Earnings {
  today_cents: number;
  week_cents: number;
  month_cents: number;
  today_orders: number;
  week_orders: number;
  month_orders: number;
  pending_payouts_cents: number;
  next_payout_at: string | null;
}

export function EarningsPage() {
  const { data, isLoading } = useQuery({
    queryKey: ["restaurant", "earnings"],
    queryFn: async () => {
      const { data } = await apiClient.get<Earnings>("/restaurant/earnings");
      return data;
    },
  });

  if (isLoading || !data) {
    return <div className="p-8 text-center text-gray-500">Loading…</div>;
  }

  return (
    <div className="mx-auto max-w-3xl px-4 py-8">
      <h1 className="font-display text-2xl font-bold mb-6">Earnings</h1>

      {/* Pending payout banner */}
      <div className="card p-4 mb-6 bg-gradient-to-br from-brand-50 to-amber-50">
        <div className="flex items-center justify-between">
          <div>
            <div className="text-sm text-gray-600">Pending payout</div>
            <div className="font-display text-2xl font-bold text-brand-700">
              ${(data.pending_payouts_cents / 100).toFixed(2)}
            </div>
          </div>
          {data.next_payout_at && (
            <div className="text-right text-sm">
              <Clock size={16} className="inline mr-1 text-gray-400" />
              <div className="text-gray-500">Next payout</div>
              <div className="font-medium">
                {new Date(data.next_payout_at).toLocaleDateString()}
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Earnings grid */}
      <div className="grid gap-4 sm:grid-cols-3 mb-6">
        <EarningsCard
          label="Today"
          amount={data.today_cents}
          orders={data.today_orders}
        />
        <EarningsCard
          label="This week"
          amount={data.week_cents}
          orders={data.week_orders}
        />
        <EarningsCard
          label="This month"
          amount={data.month_cents}
          orders={data.month_orders}
        />
      </div>

      {/* Notes */}
      <div className="card p-4 text-sm text-gray-500">
        <p className="mb-2">
          <TrendingUp size={14} className="inline mr-1" />
          Earnings shown are net of platform commission (15%).
        </p>
        <p>
          <Package size={14} className="inline mr-1" />
          Payouts are processed weekly on Mondays via Stripe Connect.
        </p>
      </div>

      <div className="mt-6 card p-4">
        <h2 className="font-semibold mb-2">How payouts work</h2>
        <ul className="text-sm text-gray-600 space-y-1">
          <li>• Earnings accrue after each delivered order</li>
          <li>• Platform commission (15%) is deducted from subtotal</li>
          <li>• Delivery fees + tips go to the driver, not the restaurant</li>
          <li>• Weekly bank transfer via Stripe Connect on Mondays</li>
          <li>• Instant payout available for 1% fee (min $10 balance)</li>
        </ul>
      </div>
    </div>
  );
}

function EarningsCard({
  label,
  amount,
  orders,
}: {
  label: string;
  amount: number;
  orders: number;
}) {
  return (
    <div className="card p-4">
      <div className="text-xs text-gray-500">{label}</div>
      <div className="mt-1 flex items-center gap-1">
        <DollarSign size={16} className="text-green-600" />
        <span className="font-display text-xl font-bold">
          {(amount / 100).toFixed(2)}
        </span>
      </div>
      <div className="mt-1 text-xs text-gray-500">
        {orders} order{orders === 1 ? "" : "s"}
      </div>
    </div>
  );
}
