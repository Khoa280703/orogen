"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { Check } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader } from "@/components/ui/card";

interface Plan {
  id: number;
  name: string;
  slug: string;
  requests_per_day: number | null;
  requests_per_month: number | null;
  price_usd: number | null;
  price_vnd: number | null;
}

const featureMap: Record<string, string[]> = {
  free: ["Studio chat access", "Image generation basics", "Saved history", "Usage dashboard"],
  pro: ["Higher chat + image quota", "Faster response priority", "More monthly capacity", "Best for active creators"],
  enterprise: ["Custom quotas and billing", "Team-ready support", "Highest throughput", "Expanded model access"],
};

function planFeatures(plan: Plan) {
  return featureMap[plan.slug] || [
    `${plan.requests_per_day === -1 ? "Unlimited" : plan.requests_per_day || 0} requests per day`,
    `${plan.requests_per_month === -1 ? "Unlimited" : plan.requests_per_month || 0} requests per month`,
    "Chat + image studio access",
    "Usage and billing controls",
  ];
}

const comparisonRows = [
  {
    label: "Daily requests",
    getValue: (plan: Plan) => (plan.requests_per_day === -1 ? "Unlimited" : plan.requests_per_day || 0),
  },
  {
    label: "Monthly requests",
    getValue: (plan: Plan) => (plan.requests_per_month === -1 ? "Unlimited" : plan.requests_per_month || 0),
  },
  {
    label: "Studio access",
    getValue: () => "Chat + Images",
  },
  {
    label: "Billing controls",
    getValue: (plan: Plan) => (plan.price_usd === 0 ? "Self-serve" : plan.slug === "enterprise" ? "Custom" : "Included"),
  },
];

export default function PricingPage() {
  const [plans, setPlans] = useState<Plan[]>([]);
  const [loading, setLoading] = useState(true);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  useEffect(() => {
    async function fetchPlans() {
      try {
        setErrorMessage(null);
        const response = await fetch("/api/plans");
        if (!response.ok) {
          setErrorMessage(`Failed to load plans (${response.status}).`);
          return;
        }
        setPlans(await response.json());
      } catch (error) {
        setErrorMessage(error instanceof Error ? error.message : "Failed to load plans.");
      } finally {
        setLoading(false);
      }
    }

    void fetchPlans();
  }, []);

  return (
    <div className="px-4 py-20">
      <div className="container mx-auto max-w-6xl">
        <div className="mx-auto max-w-3xl text-center">
          <h1 className="text-4xl font-bold text-white">Pricing for a studio workflow</h1>
          <p className="mt-4 text-xl text-slate-400">
            Pick the plan that matches how often you chat, generate images, and review usage with your team.
          </p>
        </div>

        {errorMessage ? (
          <div className="mx-auto mt-10 max-w-3xl border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-200">
            {errorMessage}
          </div>
        ) : null}

        {loading ? (
          <div className="mt-10 text-center text-slate-400">Loading plans...</div>
        ) : (
          <div className="mt-14 grid gap-8 md:grid-cols-2 xl:grid-cols-3">
            {plans.map((plan) => (
              <Card key={plan.id} className="flex flex-col border-white/10 bg-slate-950 text-white">
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <h2 className="text-2xl font-bold">{plan.name}</h2>
                    {plan.slug === "free" ? <Badge className="bg-emerald-500/20 text-emerald-300">Starter</Badge> : null}
                    {plan.slug === "pro" ? <Badge className="bg-blue-500/20 text-blue-300">Most used</Badge> : null}
                    {plan.slug === "enterprise" ? <Badge className="bg-slate-200/10 text-slate-200">Scale</Badge> : null}
                  </div>
                  <div className="mt-4">
                    {plan.price_usd === 0 ? (
                      <div className="text-4xl font-bold">Free</div>
                    ) : (
                      <>
                        <div className="text-4xl font-bold">${plan.price_usd || "Custom"}</div>
                        {plan.price_vnd ? <div className="mt-1 text-sm text-slate-400">or ₫{plan.price_vnd.toLocaleString()}</div> : null}
                        <div className="mt-1 text-sm text-slate-500">per month</div>
                      </>
                    )}
                  </div>
                </CardHeader>
                <CardContent className="flex-1">
                  <ul className="space-y-3">
                    {planFeatures(plan).map((feature) => (
                      <li key={feature} className="flex items-start gap-2">
                        <Check className="mt-0.5 h-5 w-5 shrink-0 text-emerald-400" />
                        <span className="text-slate-300">{feature}</span>
                      </li>
                    ))}
                    <li className="flex items-start gap-2">
                      <Check className="mt-0.5 h-5 w-5 shrink-0 text-emerald-400" />
                      <span className="text-slate-300">
                        {plan.requests_per_day === -1 ? "Unlimited" : plan.requests_per_day || 0} daily requests
                      </span>
                    </li>
                    <li className="flex items-start gap-2">
                      <Check className="mt-0.5 h-5 w-5 shrink-0 text-emerald-400" />
                      <span className="text-slate-300">
                        {plan.requests_per_month === -1 ? "Unlimited" : plan.requests_per_month || 0} monthly requests
                      </span>
                    </li>
                  </ul>
                </CardContent>
                <CardContent className="pt-0">
                  <Link
                    href="/login"
                    className={`inline-flex h-8 w-full items-center justify-center rounded-[var(--radius)] border px-2.5 text-sm font-medium transition ${
                      plan.slug === "pro"
                        ? "border-transparent bg-primary text-primary-foreground hover:bg-primary/80"
                        : "border-white/10 bg-transparent text-white hover:bg-white/[0.06]"
                    }`}
                  >
                    {plan.price_usd === 0 ? "Start free" : `Choose ${plan.name}`}
                  </Link>
                </CardContent>
              </Card>
            ))}
          </div>
        )}

        {plans.length ? (
          <div className="mt-20 overflow-x-auto">
            <table className="w-full text-left text-sm text-slate-300">
              <thead>
                <tr className="border-b border-slate-800">
                  <th className="px-4 py-4">Feature</th>
                  {plans.map((plan) => (
                    <th key={plan.id} className="px-4 py-4">{plan.name}</th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {comparisonRows.map((row) => (
                  <tr key={row.label} className="border-b border-slate-900">
                    <td className="px-4 py-4">{row.label}</td>
                    {plans.map((plan) => (
                      <td key={`${row.label}-${plan.id}`} className="px-4 py-4">{row.getValue(plan)}</td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : null}
      </div>
    </div>
  );
}
