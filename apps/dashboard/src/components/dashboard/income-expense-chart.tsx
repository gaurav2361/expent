"use client";

import type { TransactionWithDetail } from "@expent/types";
import * as React from "react";
import { Bar, BarChart, CartesianGrid, Legend, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";

interface IncomeExpenseChartProps {
  transactions: TransactionWithDetail[] | undefined;
}

export function IncomeExpenseChart({ transactions }: IncomeExpenseChartProps) {
  const data = React.useMemo(() => {
    if (!transactions) return [];
    const now = new Date();
    const map = new Map<string, { income: number; expense: number }>();

    for (let i = 5; i >= 0; i--) {
      const d = new Date(now.getFullYear(), now.getMonth() - i, 1);
      const key = d.toLocaleString("default", { month: "short" });
      map.set(key, { income: 0, expense: 0 });
    }

    transactions.forEach((txn) => {
      const d = new Date(txn.date);
      const key = d.toLocaleString("default", { month: "short" });
      const entry = map.get(key);
      if (entry) {
        const amt = parseFloat(txn.amount);
        if (txn.direction === "IN") entry.income += amt;
        else entry.expense += amt;
      }
    });

    return Array.from(map.entries()).map(([name, vals]) => ({
      name,
      income: Math.round(vals.income),
      expense: Math.round(vals.expense),
    }));
  }, [transactions]);

  return (
    <ResponsiveContainer width="100%" height={300}>
      <BarChart data={data}>
        <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
        <XAxis dataKey="name" stroke="#888888" fontSize={12} tickLine={false} axisLine={false} />
        <YAxis stroke="#888888" fontSize={12} tickLine={false} axisLine={false} tickFormatter={(v) => `₹${v}`} />
        <Tooltip contentStyle={{ borderRadius: "8px" }} />
        <Legend />
        <Bar dataKey="income" fill="#10b981" radius={[4, 4, 0, 0]} name="Income" />
        <Bar dataKey="expense" fill="#ef4444" radius={[4, 4, 0, 0]} name="Expense" />
      </BarChart>
    </ResponsiveContainer>
  );
}
