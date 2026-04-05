"use client";

import * as React from "react";
import {
  Cell,
  Pie,
  PieChart,
  ResponsiveContainer,
  Tooltip,
} from "recharts";
import type { TransactionWithDetail } from "@expent/types";

const COLORS = [
  "#3b82f6", "#ef4444", "#10b981", "#f97316", "#8b5cf6",
  "#ec4899", "#06b6d4", "#eab308", "#64748b", "#14b8a6",
];

interface CategoryChartProps {
  transactions: TransactionWithDetail[] | undefined;
}

export function CategoryChart({ transactions }: CategoryChartProps) {
  const data = React.useMemo(() => {
    if (!transactions) return [];
    const map = new Map<string, number>();

    transactions.forEach((txn) => {
      if (txn.direction === "OUT") {
        const cat = (txn as Record<string, unknown>).category_name as string || "Uncategorized";
        map.set(cat, (map.get(cat) || 0) + parseFloat(txn.amount));
      }
    });

    return Array.from(map.entries())
      .map(([name, value]) => ({ name, value: Math.round(value) }))
      .sort((a, b) => b.value - a.value)
      .slice(0, 8);
  }, [transactions]);

  if (data.length === 0) {
    return (
      <div className="flex items-center justify-center h-[300px] text-muted-foreground text-sm">
        No expense data to display.
      </div>
    );
  }

  return (
    <ResponsiveContainer width="100%" height={300}>
      <PieChart>
        <Pie
          data={data}
          cx="50%"
          cy="50%"
          innerRadius={60}
          outerRadius={100}
          paddingAngle={4}
          dataKey="value"
          label={({ name, percent }) => `${name} ${((percent ?? 0) * 100).toFixed(0)}%`}
          labelLine={false}
        >
          {data.map((entry) => (
            <Cell key={entry.name} fill={COLORS[data.indexOf(entry) % COLORS.length]} />
          ))}
        </Pie>
        <Tooltip contentStyle={{ borderRadius: "8px" }} />
      </PieChart>
    </ResponsiveContainer>
  );
}
