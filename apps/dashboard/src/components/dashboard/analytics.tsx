"use client";

import type { TransactionWithDetail } from "@expent/types";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@expent/ui/components/card";
import { BarChart3Icon, TrendingDownIcon, TrendingUpIcon, WalletIcon } from "lucide-react";
import * as React from "react";
import { Area, AreaChart, CartesianGrid, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import { useTransactions } from "@/hooks/use-transactions";

export function Analytics() {
  const { transactions } = useTransactions();

  // Weekly income + expense area chart (last 7 days)
  const weeklyData = React.useMemo(() => {
    if (!transactions) return [];
    const now = new Date();
    const days = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    const map = new Map<string, { income: number; expense: number }>();

    for (let i = 6; i >= 0; i--) {
      const d = new Date(now);
      d.setDate(d.getDate() - i);
      map.set(days[d.getDay()], { income: 0, expense: 0 });
    }

    transactions.forEach((txn: TransactionWithDetail) => {
      const d = new Date(txn.date);
      const daysSince = Math.floor((now.getTime() - d.getTime()) / (1000 * 60 * 60 * 24));
      if (daysSince >= 0 && daysSince < 7) {
        const key = days[d.getDay()];
        const entry = map.get(key);
        if (entry) {
          const amt = parseFloat(txn.amount);
          if (txn.direction === "IN") entry.income += amt;
          else entry.expense += amt;
        }
      }
    });

    return Array.from(map.entries()).map(([name, vals]) => ({
      name,
      income: Math.round(vals.income),
      expense: Math.round(vals.expense),
    }));
  }, [transactions]);

  // Summary metrics
  const metrics = React.useMemo(() => {
    if (!transactions) return { totalIncome: 0, totalExpense: 0, txnCount: 0, avgTxn: 0 };
    let totalIncome = 0;
    let totalExpense = 0;

    transactions.forEach((txn: TransactionWithDetail) => {
      const amt = parseFloat(txn.amount);
      if (txn.direction === "IN") totalIncome += amt;
      else totalExpense += amt;
    });

    return {
      totalIncome,
      totalExpense,
      txnCount: transactions.length,
      avgTxn: transactions.length > 0 ? (totalIncome + totalExpense) / transactions.length : 0,
    };
  }, [transactions]);

  // Top expense sources (by contact or description)
  const topExpenses = React.useMemo(() => {
    if (!transactions) return [];
    const map = new Map<string, number>();
    transactions.forEach((txn: TransactionWithDetail) => {
      if (txn.direction === "OUT") {
        const key = ((txn as Record<string, unknown>).contact_name as string) || txn.source || "Unknown";
        map.set(key, (map.get(key) || 0) + parseFloat(txn.amount));
      }
    });
    return Array.from(map.entries())
      .map(([name, value]) => ({ name, value: Math.round(value) }))
      .sort((a, b) => b.value - a.value)
      .slice(0, 5);
  }, [transactions]);

  // Top income sources
  const topIncome = React.useMemo(() => {
    if (!transactions) return [];
    const map = new Map<string, number>();
    transactions.forEach((txn: TransactionWithDetail) => {
      if (txn.direction === "IN") {
        const key = ((txn as Record<string, unknown>).contact_name as string) || txn.source || "Unknown";
        map.set(key, (map.get(key) || 0) + parseFloat(txn.amount));
      }
    });
    return Array.from(map.entries())
      .map(([name, value]) => ({ name, value: Math.round(value) }))
      .sort((a, b) => b.value - a.value)
      .slice(0, 5);
  }, [transactions]);

  return (
    <div className="space-y-4">
      {/* Weekly Trend Area Chart */}
      <Card>
        <CardHeader>
          <CardTitle>Weekly Trend</CardTitle>
          <CardDescription>Income and expenses over the past 7 days</CardDescription>
        </CardHeader>
        <CardContent className="px-6">
          <ResponsiveContainer width="100%" height={300}>
            <AreaChart data={weeklyData}>
              <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
              <XAxis dataKey="name" stroke="#888888" fontSize={12} tickLine={false} axisLine={false} />
              <YAxis stroke="#888888" fontSize={12} tickLine={false} axisLine={false} tickFormatter={(v) => `₹${v}`} />
              <Tooltip contentStyle={{ borderRadius: "8px" }} />
              <defs>
                <linearGradient id="colorIncome" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#10b981" stopOpacity={0.3} />
                  <stop offset="95%" stopColor="#10b981" stopOpacity={0} />
                </linearGradient>
                <linearGradient id="colorExpense" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#ef4444" stopOpacity={0.3} />
                  <stop offset="95%" stopColor="#ef4444" stopOpacity={0} />
                </linearGradient>
              </defs>
              <Area
                type="monotone"
                dataKey="income"
                stroke="#10b981"
                fillOpacity={1}
                fill="url(#colorIncome)"
                name="Income"
              />
              <Area
                type="monotone"
                dataKey="expense"
                stroke="#ef4444"
                fillOpacity={1}
                fill="url(#colorExpense)"
                name="Expense"
              />
            </AreaChart>
          </ResponsiveContainer>
        </CardContent>
      </Card>

      {/* Summary Metric Cards */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Income</CardTitle>
            <TrendingUpIcon className="h-4 w-4 text-emerald-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-emerald-600">
              ₹{metrics.totalIncome.toLocaleString(undefined, { minimumFractionDigits: 2 })}
            </div>
            <p className="text-xs text-muted-foreground">All time earnings</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Expenses</CardTitle>
            <TrendingDownIcon className="h-4 w-4 text-rose-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-rose-600">
              ₹{metrics.totalExpense.toLocaleString(undefined, { minimumFractionDigits: 2 })}
            </div>
            <p className="text-xs text-muted-foreground">All time spending</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Transactions</CardTitle>
            <BarChart3Icon className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{metrics.txnCount}</div>
            <p className="text-xs text-muted-foreground">Total recorded</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Avg. Transaction</CardTitle>
            <WalletIcon className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              ₹{metrics.avgTxn.toLocaleString(undefined, { minimumFractionDigits: 2 })}
            </div>
            <p className="text-xs text-muted-foreground">Mean value per transaction</p>
          </CardContent>
        </Card>
      </div>

      {/* Horizontal Bar Lists */}
      <div className="grid grid-cols-1 gap-4 lg:grid-cols-7">
        <Card className="col-span-1 lg:col-span-4">
          <CardHeader>
            <CardTitle>Top Expenses</CardTitle>
            <CardDescription>Highest spending contacts/sources</CardDescription>
          </CardHeader>
          <CardContent>
            <SimpleBarList
              items={topExpenses}
              barClass="bg-rose-500"
              valueFormatter={(n) => `₹${n.toLocaleString()}`}
            />
          </CardContent>
        </Card>
        <Card className="col-span-1 lg:col-span-3">
          <CardHeader>
            <CardTitle>Top Income</CardTitle>
            <CardDescription>Highest earning contacts/sources</CardDescription>
          </CardHeader>
          <CardContent>
            <SimpleBarList
              items={topIncome}
              barClass="bg-emerald-500"
              valueFormatter={(n) => `₹${n.toLocaleString()}`}
            />
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

function SimpleBarList({
  items,
  valueFormatter,
  barClass,
}: {
  items: { name: string; value: number }[];
  valueFormatter: (n: number) => string;
  barClass: string;
}) {
  const max = Math.max(...items.map((i) => i.value), 1);

  if (items.length === 0) {
    return <p className="text-sm text-muted-foreground text-center py-6">No data yet.</p>;
  }

  return (
    <ul className="space-y-3">
      {items.map((i) => {
        const width = `${Math.round((i.value / max) * 100)}%`;
        return (
          <li key={i.name} className="flex items-center justify-between gap-3">
            <div className="min-w-0 flex-1">
              <div className="mb-1 truncate text-xs text-muted-foreground">{i.name}</div>
              <div className="h-2.5 w-full rounded-full bg-muted">
                <div className={`h-2.5 rounded-full transition-all ${barClass}`} style={{ width }} />
              </div>
            </div>
            <div className="ps-2 text-xs font-medium tabular-nums">{valueFormatter(i.value)}</div>
          </li>
        );
      })}
    </ul>
  );
}
