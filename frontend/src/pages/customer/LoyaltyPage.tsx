import { Star, TrendingUp, Award, Sparkles } from "lucide-react";
import { useLoyalty, useLoyaltyTransactions } from "@/hooks/useLoyalty";
import { clsx } from "clsx";

const tierColor: Record<string, string> = {
  silver: "from-gray-300 to-gray-500",
  gold: "from-amber-300 to-yellow-500",
  platinum: "from-violet-300 to-purple-500",
};

const tierIcon: Record<string, typeof Star> = {
  silver: Award,
  gold: Star,
  platinum: Sparkles,
};

export function LoyaltyPage() {
  const { data: account, isLoading } = useLoyalty();
  const { data: transactions } = useLoyaltyTransactions();

  if (isLoading || !account) {
    return <div className="p-8 text-center text-gray-500">Loading…</div>;
  }

  const TierIcon = tierIcon[account.tier] ?? Star;
  const progress =
    account.tier === "platinum"
      ? 100
      : Math.min(
          100,
          (account.lifetime_points / account.next_tier_points) * 100,
        );

  return (
    <div className="mx-auto max-w-md px-4 py-8">
      <h1 className="font-display text-2xl font-bold">Loyalty Rewards</h1>

      {/* Tier card */}
      <div
        className={clsx(
          "mt-4 rounded-xl bg-gradient-to-br p-5 text-white shadow-lg",
          tierColor[account.tier],
        )}
      >
        <div className="flex items-center justify-between">
          <div>
            <div className="text-xs uppercase tracking-wider opacity-80">Current tier</div>
            <div className="font-display text-2xl font-bold capitalize flex items-center gap-2">
              <TierIcon size={20} /> {account.tier}
            </div>
          </div>
          <div className="text-right">
            <div className="text-3xl font-bold">{account.points_balance}</div>
            <div className="text-xs opacity-80">points</div>
          </div>
        </div>
        <div className="mt-4">
          <div className="text-xs flex justify-between mb-1">
            <span>{account.lifetime_points} lifetime</span>
            {account.tier !== "platinum" && (
              <span>Next: {account.next_tier_points}</span>
            )}
          </div>
          <div className="h-2 rounded-full bg-white/30 overflow-hidden">
            <div
              className="h-full bg-white rounded-full transition-all"
              style={{ width: `${progress}%` }}
            />
          </div>
        </div>
      </div>

      {/* Benefits */}
      <section className="mt-6">
        <h2 className="font-semibold mb-2">Your benefits</h2>
        <ul className="space-y-1">
          {account.tier_benefits.map((b) => (
            <li key={b} className="text-sm text-gray-700 flex items-center gap-2">
              <Sparkles size={14} className="text-brand-500" />
              {b}
            </li>
          ))}
        </ul>
      </section>

      {/* How it works */}
      <section className="mt-6 card p-4">
        <h2 className="font-semibold mb-2 flex items-center gap-2">
          <TrendingUp size={16} /> How points work
        </h2>
        <ul className="text-sm text-gray-600 space-y-1">
          <li>• Earn 1 point for every $1 spent on orders</li>
          <li>• Platinum members earn 1.5x points</li>
          <li>• 100 points = $1 to redeem at checkout</li>
          <li>• Max 50% of order total redeemable via points</li>
        </ul>
      </section>

      {/* Transactions */}
      <section className="mt-6">
        <h2 className="font-semibold mb-2">Recent activity</h2>
        {transactions && transactions.length > 0 ? (
          <div className="space-y-2">
            {transactions.map((t) => (
              <div key={t.id} className="card p-3 flex items-center justify-between text-sm">
                <div>
                  <div className="font-medium capitalize">{t.reason.replace(/_/g, " ")}</div>
                  <div className="text-xs text-gray-500">
                    {new Date(t.created_at).toLocaleString()}
                  </div>
                </div>
                <div
                  className={clsx(
                    "font-semibold",
                    t.points_delta > 0 ? "text-green-600" : "text-red-600",
                  )}
                >
                  {t.points_delta > 0 ? "+" : ""}
                  {t.points_delta}
                </div>
              </div>
            ))}
          </div>
        ) : (
          <p className="text-sm text-gray-500">
            No transactions yet. Place an order to start earning!
          </p>
        )}
      </section>
    </div>
  );
}
