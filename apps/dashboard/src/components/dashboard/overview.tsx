"use client";

import type { TransactionWithDetail } from "@expent/types";
import * as React from "react";
import { Bar, BarChart, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import { useTransactions } from "@/hooks/use-transactions";

export function Overview() {
  const { transactions } = useTransactions();

  // Compute monthly data from the last 6 months
  const chartData = React.useMemo(() => {
    if (!transactions) return [];

    const now = new Date();
    const map = new Map<string, number>();

    // Initialize last 6 months
    for (let i = 5; i >= 0; i--) {
      const d = new Date(now.getFullYear(), now.getMonth() - i, 1);
      const monthStr = d.toLocaleString("default", { month: "short" });
      map.set(monthStr, 0);
    }

    transactions.forEach((txn: TransactionWithDetail) => {
      // Only count OUT transactions for the expense overview
      if (txn.direction === "OUT") {
        const d = new Date(txn.date);
        const monthStr = d.toLocaleString("default", { month: "short" });
        if (map.has(monthStr)) {
          map.set(monthStr, (map.get(monthStr) || 0) + parseFloat(txn.amount));
        }
      }
    });

    return Array.from(map.entries()).map(([name, total]) => ({
      name,
      total,
    }));
  }, [transactions]);

  return (
    <ResponsiveContainer width="100%" height={350}>
      <BarChart data={chartData}>
        <XAxis dataKey="name" stroke="#888888" fontSize={12} tickLine={false} axisLine={false} />
        <YAxis
          direction="ltr"
          stroke="#888888"
          fontSize={12}
          tickLine={false}
          axisLine={false}
          tickFormatter={(value) => `₹${value}`}
        />
        <Tooltip cursor={{ fill: "var(--muted)" }} contentStyle={{ borderRadius: "8px" }} />
        <Bar dataKey="total" fill="currentColor" radius={[4, 4, 0, 0]} className="fill-primary" />
      </BarChart>
    </ResponsiveContainer>
  );
}
