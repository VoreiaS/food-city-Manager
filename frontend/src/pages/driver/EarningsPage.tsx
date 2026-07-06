import { useQuery } from "@tanstack/react-query";
import { apiClient } from "@/api/client";
import { DollarSign, Bike, Clock, TrendingUp } from "lucide-react";

interface DriverEarnings {
  today_cents: number;
  week_cents: number;
  month_cents: number;
  today_deliveries: number;
  week_deliveries: number;
  month_deliveries: number;
  pending_payouts_cents: number;
}

export function EarningsPage() {
  const { data, isLoading } = useQuery({
    queryKey: ["driver", "earnings"],
    queryFn: async () => {
      const { data } = await apiClient.get<DriverEarnings>("/drivers/me/earnings");
      return data;
    },
  });

  if (isLoading || !data) {
    return <div className="p-8 text-center text-gray-500">Loading…</div>;
  }

  return (
    <div className="mx-auto max-w-md px-4 py-8">
      <h1 className="font-display text-2xl font-bold mb-6">Earnings</h1>

      {/* Pending payout */}
      <div className="card p-4 mb-6 bg-gradient-to-br from-blue-50 to-purple-50">
        <div className="text-sm text-gray-600">Pending payout</div>
        <div className="font-display text-3xl font-bold text-blue-700">
          ${(data.pending_payouts_cents / 100).toFixed(2)}
        </div>
        <div className="mt-1 text-xs text-gray-500 flex items-center gap-1">
          <Clock size={12} /> Paid out weekly on Mondays
        </div>
      </div>

      {/* Period cards */}
      <div className="grid grid-cols-3 gap-2 mb-6">
        <Card
          label="Today"
          amount={data.today_cents}
          deliveries={data.today_deliveries}
        />
        <Card
          label="Week"
          amount={data.week_cents}
          deliveries={data.week_deliveries}
        />
        <Card
          label="Month"
          amount={data.month_cents}
          deliveries={data.month_deliveries}
        />
      </div>

      {/* Earnings breakdown */}
      <div className="card p-4 mb-4">
        <h2 className="font-semibold mb-2 flex items-center gap-2">
          <TrendingUp size={16} /> How you earn
        </h2>
        <ul className="text-sm text-gray-600 space-y-1">
          <li>• Delivery fee per order (varies by distance)</li>
          <li>• 100% of customer tips</li>
          <li>• Surge bonuses during peak hours</li>
          <li>• Weekly payout via Stripe Connect</li>
        </ul>
      </div>

      <div className="card p-4">
        <h2 className="font-semibold mb-2 flex items-center gap-2">
          <Bike size={16} /> Tips to earn more
        </h2>
        <ul className="text-sm text-gray-600 space-y-1">
          <li>• Stay online during lunch (11am-2pm) and dinner (6-9pm)</li>
          <li>• Accept orders quickly to boost your acceptance rate</li>
          <li>• Be polite — higher ratings get priority for offers</li>
          <li>• Deliver during surge hours for bonus pay</li>
        </ul>
      </div>
    </div>
  );
}

function Card({
  label,
  amount,
  deliveries,
}: {
  label: string;
  amount: number;
  deliveries: number;
}) {
  return (
    <div className="card p-3 text-center">
      <div className="text-xs text-gray-500">{label}</div>
      <div className="mt-1 flex items-center justify-center gap-0.5">
        <DollarSign size={12} className="text-green-600" />
        <span className="font-bold text-sm">{(amount / 100).toFixed(0)}</span>
      </div>
      <div className="mt-0.5 text-xs text-gray-500">{deliveries} trips</div>
    </div>
  );
}
