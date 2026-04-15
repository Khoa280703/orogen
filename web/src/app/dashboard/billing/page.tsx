"use client";

export const dynamic = "force-dynamic";

import { useState, useEffect } from "react";
import { CreditCard, ArrowUpRight, History } from "lucide-react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { userApiRequest } from "@/lib/user-api";

interface Balance {
  id: number;
  amount: string;
  updated_at: string;
}

interface BillingData {
  balance: Balance;
  transactions: Array<{
    id: number;
    amount: string;
    type: string;
    reference?: string;
    created_at: string;
  }>;
}

export default function BillingPage() {
  const [billing, setBilling] = useState<BillingData | null>(null);
  const [loading, setLoading] = useState(true);
  const [notice, setNotice] = useState<{ type: "info" | "error"; message: string } | null>(null);

  useEffect(() => {
    const fetchBilling = async () => {
      try {
        setNotice(null);
        const data = await userApiRequest<BillingData>("/user/billing");
        setBilling(data);
      } catch (error) {
        setNotice({
          type: "error",
          message: error instanceof Error ? error.message : "Failed to load billing data.",
        });
      } finally {
        setLoading(false);
      }
    };

    fetchBilling();
  }, []);

  const handleTopUp = () => {
    setNotice({
      type: "info",
      message: "Payment integration coming soon. Contact support for manual top-up.",
    });
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Billing & Payments</h1>
        <p className="text-slate-400 mt-1">Manage your balance and view transaction history</p>
      </div>

      {notice && (
        <div
          className={`rounded-lg px-4 py-3 text-sm ${
            notice.type === "error"
              ? "border border-red-500/30 bg-red-500/10 text-red-200"
              : "border border-blue-500/30 bg-blue-500/10 text-blue-100"
          }`}
        >
          {notice.message}
        </div>
      )}

      {/* Balance Card */}
      <Card className="bg-gradient-to-br from-slate-900 to-slate-800 border-slate-700">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <CreditCard className="w-6 h-6" />
            Current Balance
          </CardTitle>
          <CardDescription>Your available credit for API usage</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-end justify-between">
            <div>
              <div className="text-5xl font-bold text-white">
                ${parseFloat(billing?.balance?.amount || "0").toFixed(2)}
              </div>
              <p className="text-sm text-slate-400 mt-2">
                Last updated: {billing?.balance?.updated_at
                  ? new Date(billing.balance.updated_at).toLocaleString()
                  : "N/A"}
              </p>
            </div>
            <Button size="lg" onClick={handleTopUp}>
              <ArrowUpRight className="w-5 h-5 mr-2" />
              Top Up
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Pricing Plans */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <Card className="bg-slate-900 border-slate-800">
          <CardHeader>
            <CardTitle className="text-blue-400">Free</CardTitle>
            <CardDescription>Get started for free</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold mb-4">$0<span className="text-lg text-slate-400">/mo</span></div>
            <ul className="space-y-2 text-sm text-slate-400">
              <li className="flex items-center gap-2">✓ 10 requests/day</li>
              <li className="flex items-center gap-2">✓ 300 requests/month</li>
              <li className="flex items-center gap-2">✓ Grok-3 model</li>
              <li className="flex items-center gap-2">✓ Streaming support</li>
            </ul>
          </CardContent>
        </Card>

        <Card className="bg-slate-900 border-blue-500/50 relative">
          <div className="absolute -top-3 left-1/2 -translate-x-1/2 bg-blue-500 text-white text-xs px-3 py-1 rounded-full">
            Popular
          </div>
          <CardHeader>
            <CardTitle className="text-blue-400">Pro</CardTitle>
            <CardDescription>For serious developers</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold mb-4">$29.99<span className="text-lg text-slate-400">/mo</span></div>
            <ul className="space-y-2 text-sm text-slate-400">
              <li className="flex items-center gap-2">✓ 1,000 requests/day</li>
              <li className="flex items-center gap-2">✓ 30,000 requests/month</li>
              <li className="flex items-center gap-2">✓ Grok-3 & Grok-4</li>
              <li className="flex items-center gap-2">✓ Priority support</li>
            </ul>
            <Button className="w-full mt-4" onClick={handleTopUp}>Upgrade</Button>
          </CardContent>
        </Card>

        <Card className="bg-slate-900 border-slate-800">
          <CardHeader>
            <CardTitle className="text-purple-400">Enterprise</CardTitle>
            <CardDescription>For high-volume usage</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold mb-4">$199.99<span className="text-lg text-slate-400">/mo</span></div>
            <ul className="space-y-2 text-sm text-slate-400">
              <li className="flex items-center gap-2">✓ Unlimited requests</li>
              <li className="flex items-center gap-2">✓ All models</li>
              <li className="flex items-center gap-2">✓ Dedicated support</li>
              <li className="flex items-center gap-2">✓ Custom SLA</li>
            </ul>
            <Button variant="outline" className="w-full mt-4" onClick={handleTopUp}>Contact Sales</Button>
          </CardContent>
        </Card>
      </div>

      {/* Transaction History */}
      <Card className="bg-slate-900 border-slate-800">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <History className="w-5 h-5" />
            Transaction History
          </CardTitle>
          <CardDescription>Your recent top-ups and charges</CardDescription>
        </CardHeader>
        <CardContent>
          {billing?.transactions?.length ? (
            <div className="space-y-4">
              {billing.transactions.map((tx) => (
                <div key={tx.id} className="flex items-center justify-between p-4 bg-slate-800/50 rounded-lg">
                  <div className="flex items-center gap-4">
                    <div
                      className={`w-10 h-10 rounded-full flex items-center justify-center ${
                        tx.type === "credit" ? "bg-green-500/10" : "bg-red-500/10"
                      }`}
                    >
                      {tx.type === "credit" ? (
                        <ArrowUpRight className="w-5 h-5 text-green-400 rotate-[-45deg]" />
                      ) : (
                        <ArrowUpRight className="w-5 h-5 text-red-400 rotate-[135deg]" />
                      )}
                    </div>
                    <div>
                      <div className="font-medium">
                        {tx.type === "credit" ? "Top-up" : "Charge"}
                      </div>
                      <div className="text-sm text-slate-400">
                        {new Date(tx.created_at).toLocaleString()}
                      </div>
                    </div>
                  </div>
                  <div className="text-right">
                    <div
                      className={`font-mono font-medium ${
                        tx.type === "credit" ? "text-green-400" : "text-red-400"
                      }`}
                    >
                      {tx.type === "credit" ? "+" : "-"}${parseFloat(tx.amount).toFixed(2)}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className="text-center py-8 text-slate-400">
              <History className="w-12 h-12 mx-auto mb-4 opacity-50" />
              <p>No transactions yet</p>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
