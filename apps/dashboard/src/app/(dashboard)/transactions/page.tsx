"use client";
import * as React from "react";
import { Badge } from "@expent/ui/components/badge";
import { Button } from "@expent/ui/components/button";
import { Card, CardContent, CardHeader, CardTitle } from "@expent/ui/components/card";
import { Checkbox } from "@expent/ui/components/checkbox";
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@expent/ui/components/dropdown-menu";
import { Input } from "@expent/ui/components/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@expent/ui/components/select";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@expent/ui/components/table";
import { Tabs, TabsList, TabsTrigger } from "@expent/ui/components/tabs";
import {
  flexRender,
  getCoreRowModel,
  getFacetedRowModel,
  getFacetedUniqueValues,
  getFilteredRowModel,
  getPaginationRowModel,
  getSortedRowModel,
  useReactTable,
} from "@tanstack/react-table";
import type { ColumnDef, ColumnFiltersState, SortingState, VisibilityState } from "@tanstack/react-table";
import {
  ArrowDownIcon,
  ArrowUpIcon,
  DownloadIcon,
  ScaleIcon,
  Columns3Icon,
  MoreVerticalIcon,
  SearchIcon,
  ChevronDownIcon,
  ChevronLeftIcon,
  ChevronRightIcon,
  ChevronsLeftIcon,
  ChevronsRightIcon,
  Share2Icon,
} from "lucide-react";
import { SplitDialog } from "@/components/transactions/split-dialog";
import { TransactionViewer } from "@/components/transactions/transaction-viewer";
import type { Transaction } from "@/components/transactions/transaction-viewer";
import { useTransactions } from "@/hooks/use-transactions";

// Route Component
export default function TransactionsPage() {
  const { transactions: rawTransactions, updateMutation, deleteMutation } = useTransactions();

  // Selected Txn for Split Action
  const [splitDialogOpen, setSplitDialogOpen] = React.useState(false);
  const [selectedTxn, setSelectedTxn] = React.useState<{ id: string; amount: string } | null>(null);

  // Table State
  const [rowSelection, setRowSelection] = React.useState({});
  const [columnVisibility, setColumnVisibility] = React.useState<VisibilityState>({});
  const [columnFilters, setColumnFilters] = React.useState<ColumnFiltersState>([]);
  const [sorting, setSorting] = React.useState<SortingState>([{ id: "date", desc: true }]);
  const [pagination, setPagination] = React.useState({ pageIndex: 0, pageSize: 15 });
  const [activeTab, setActiveTab] = React.useState("all");

  const data = React.useMemo<Transaction[]>(() => {
    if (!rawTransactions) return [];

    // Apply tab filtering manually before the table logic
    if (activeTab === "income") return rawTransactions.filter((t: Transaction) => t.direction === "IN");
    if (activeTab === "expense") return rawTransactions.filter((t: Transaction) => t.direction === "OUT");

    return rawTransactions;
  }, [rawTransactions, activeTab]);

  // Derived Metrics (based on raw data so they don't jump around strictly on search, but do jump on tabs)
  const { totalIncome, totalExpense, netBalance } = React.useMemo(() => {
    let income = 0;
    let expense = 0;

    (data || []).forEach((txn: Transaction) => {
      const amount = parseFloat(txn.amount);
      if (txn.direction === "IN") income += amount;
      else expense += amount;
    });

    return { totalIncome: income, totalExpense: expense, netBalance: income - expense };
  }, [data]);

  const triggerSplit = React.useCallback((id: string, amount: string) => {
    setSelectedTxn({ id, amount });
    setSplitDialogOpen(true);
  }, []);

  const columns = React.useMemo<ColumnDef<Transaction>[]>(
    () => [
      {
        id: "select",
        header: ({ table }) => (
          <div className="flex items-center justify-center">
            <Checkbox
              checked={table.getIsAllPageRowsSelected()}
              indeterminate={table.getIsSomePageRowsSelected() && !table.getIsAllPageRowsSelected()}
              onCheckedChange={(value) => table.toggleAllPageRowsSelected(!!value)}
              aria-label="Select all"
            />
          </div>
        ),
        cell: ({ row }) => (
          <div className="flex items-center justify-center">
            <Checkbox
              checked={row.getIsSelected()}
              onCheckedChange={(value) => row.toggleSelected(!!value)}
              aria-label="Select row"
            />
          </div>
        ),
        enableSorting: false,
        enableHiding: false,
      },
      {
        accessorKey: "date",
        header: "Date",
        cell: ({ row }) => {
          return (
            <span className="text-muted-foreground whitespace-nowrap">
              {new Date(row.original.date).toLocaleDateString("en-US", {
                month: "short",
                day: "numeric",
                year: "numeric",
              })}
            </span>
          );
        },
      },
      {
        accessorKey: "source",
        header: "Description",
        cell: ({ row }) => {
          return <TransactionViewer item={row.original} onUpdate={(id, data) => updateMutation.mutate({ id, data })} />;
        },
      },
      {
        accessorKey: "direction",
        header: "Direction",
        cell: ({ row }) => {
          const isIn = row.original.direction === "IN";
          return (
            <Badge
              variant={isIn ? "default" : "destructive"}
              className={
                isIn
                  ? "bg-green-100/50 text-green-700 hover:bg-green-200/50 border-green-200"
                  : "bg-red-100/50 text-red-700 hover:bg-red-200/50 border-red-200"
              }
            >
              {isIn ? "Income" : "Expense"}
            </Badge>
          );
        },
      },
      {
        accessorKey: "amount",
        header: () => <div className="text-right">Amount</div>,
        cell: ({ row }) => {
          const isIn = row.original.direction === "IN";
          return (
            <div
              className={`font-mono font-medium text-right tabular-nums ${isIn ? "text-green-600 dark:text-green-500" : ""}`}
            >
              ₹
              {parseFloat(row.original.amount).toLocaleString("en-IN", {
                minimumFractionDigits: 2,
                maximumFractionDigits: 2,
              })}
            </div>
          );
        },
      },
      {
        id: "actions",
        cell: ({ row }) => (
          <DropdownMenu>
            <DropdownMenuTrigger
              render={
                <Button
                  variant="ghost"
                  className="flex size-8 text-muted-foreground data-[state=open]:bg-muted ml-auto"
                  size="icon"
                >
                  <MoreVerticalIcon className="h-4 w-4" />
                  <span className="sr-only">Open menu</span>
                </Button>
              }
            />
            <DropdownMenuContent align="end" className="w-40">
              <DropdownMenuItem onClick={() => triggerSplit(row.original.id, row.original.amount)}>
                <Share2Icon className="mr-2 h-4 w-4" /> Split
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem
                variant="destructive"
                onClick={() => {
                  if (confirm("Are you sure you want to delete this transaction?")) {
                    deleteMutation.mutate(row.original.id);
                  }
                }}
              >
                Delete row
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        ),
      },
    ],
    [triggerSplit, updateMutation, deleteMutation]
  );

  const table = useReactTable({
    data,
    columns,
    state: {
      sorting,
      columnVisibility,
      rowSelection,
      columnFilters,
      pagination,
    },
    enableRowSelection: true,
    onRowSelectionChange: setRowSelection,
    onSortingChange: setSorting,
    onColumnFiltersChange: setColumnFilters,
    onColumnVisibilityChange: setColumnVisibility,
    onPaginationChange: setPagination,
    getCoreRowModel: getCoreRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getFacetedRowModel: getFacetedRowModel(),
    getFacetedUniqueValues: getFacetedUniqueValues(),
  });

  const handleExportCSV = () => {
    const rowsExport = table.getFilteredRowModel().rows.map((row) => row.original);
    if (!rowsExport.length) return;

    const headers = ["Date", "Direction", "Amount", "Source", "ID"];
    const rows = rowsExport.map((txn: Transaction) => [txn.date, txn.direction, txn.amount, txn.source, txn.id]);

    const csvContent = "data:text/csv;charset=utf-8," + [headers.join(","), ...rows.map((e) => e.join(","))].join("\n");
    const encodedUri = encodeURI(csvContent);
    const link = document.createElement("a");
    link.setAttribute("href", encodedUri);
    link.setAttribute("download", `transactions_${new Date().toISOString().split("T")[0]}.csv`);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  };

  return (
    <>
      <div className="flex flex-1 flex-col gap-6 p-4 md:p-6 lg:p-8 max-w-7xl mx-auto w-full">
        <div className="flex justify-end">
          <Button onClick={handleExportCSV} variant="outline" size="sm" disabled={data.length === 0}>
            <DownloadIcon className="h-4 w-4 mr-2" />
            <span className="hidden sm:inline">Export CSV</span>
          </Button>
        </div>
        {/* Summary Cards */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <Card className="bg-gradient-to-br from-green-50 to-green-100/30 dark:from-green-950/20 dark:to-green-900/10 border-green-100 shadow-sm">
            <CardHeader className="flex flex-row items-center justify-between pb-2 space-y-0">
              <CardTitle className="text-sm text-green-800 dark:text-green-300">Total Income</CardTitle>
              <ArrowUpIcon className="h-4 w-4 text-green-600 dark:text-green-400" />
            </CardHeader>
            <CardContent>
              <div className="text-3xl font-bold tracking-tight text-green-700 dark:text-green-400">
                ₹{totalIncome.toLocaleString("en-IN", { maximumFractionDigits: 2 })}
              </div>
            </CardContent>
          </Card>

          <Card className="bg-gradient-to-br from-red-50 to-red-100/30 dark:from-red-950/20 dark:to-red-900/10 border-red-100 shadow-sm">
            <CardHeader className="flex flex-row items-center justify-between pb-2 space-y-0">
              <CardTitle className="text-sm text-red-800 dark:text-red-300">Total Expense</CardTitle>
              <ArrowDownIcon className="h-4 w-4 text-red-600 dark:text-red-400" />
            </CardHeader>
            <CardContent>
              <div className="text-3xl font-bold tracking-tight text-red-700 dark:text-red-400">
                ₹{totalExpense.toLocaleString("en-IN", { maximumFractionDigits: 2 })}
              </div>
            </CardContent>
          </Card>

          <Card className="bg-gradient-to-br from-blue-50 to-blue-100/30 dark:from-blue-950/20 dark:to-blue-900/10 border-blue-100 shadow-sm">
            <CardHeader className="flex flex-row items-center justify-between pb-2 space-y-0">
              <CardTitle className="text-sm text-blue-800 dark:text-blue-300">Net Balance</CardTitle>
              <ScaleIcon className="h-4 w-4 text-blue-600 dark:text-blue-400" />
            </CardHeader>
            <CardContent>
              <div className="text-3xl font-bold tracking-tight text-blue-700 dark:text-blue-400">
                {netBalance < 0 ? "-" : ""}₹{Math.abs(netBalance).toLocaleString("en-IN", { maximumFractionDigits: 2 })}
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Table Area */}
        <div className="flex flex-col flex-1 shadow-sm border rounded-xl bg-card overflow-hidden">
          <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full flex-col justify-start">
            <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between p-4 border-b bg-muted/40 gap-4">
              <TabsList className="**:data-[slot=badge]:size-5 **:data-[slot=badge]:rounded-full **:data-[slot=badge]:bg-muted-foreground/30 **:data-[slot=badge]:px-1 h-9 items-center justify-start rounded-lg bg-muted p-1 text-muted-foreground">
                <TabsTrigger value="all" className="rounded-md">
                  All Transactions
                </TabsTrigger>
                <TabsTrigger value="income" className="rounded-md">
                  Income
                </TabsTrigger>
                <TabsTrigger value="expense" className="rounded-md">
                  Expense
                </TabsTrigger>
              </TabsList>

              <div className="flex items-center gap-2 w-full sm:w-auto">
                <div className="relative flex-1 sm:w-64">
                  <SearchIcon className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
                  <Input
                    placeholder="Search transactions..."
                    value={(table.getColumn("source")?.getFilterValue() as string) ?? ""}
                    onChange={(event) => table.getColumn("source")?.setFilterValue(event.target.value)}
                    className="pl-9 bg-background h-9 border-muted-foreground/20"
                  />
                </div>

                <DropdownMenu>
                  <DropdownMenuTrigger
                    render={
                      <Button variant="outline" size="sm" className="h-9 ml-auto hidden md:flex">
                        <Columns3Icon className="mr-2 h-4 w-4" />
                        Columns
                        <ChevronDownIcon className="ml-2 h-4 w-4" />
                      </Button>
                    }
                  />
                  <DropdownMenuContent align="end" className="w-48">
                    {table
                      .getAllColumns()
                      .filter((column) => typeof column.accessorFn !== "undefined" && column.getCanHide())
                      .map((column) => {
                        return (
                          <DropdownMenuCheckboxItem
                            key={column.id}
                            className="capitalize"
                            checked={column.getIsVisible()}
                            onCheckedChange={(value) => column.toggleVisibility(!!value)}
                          >
                            {column.id}
                          </DropdownMenuCheckboxItem>
                        );
                      })}
                  </DropdownMenuContent>
                </DropdownMenu>
              </div>
            </div>

            <div className="relative flex flex-col gap-4 overflow-auto min-h-[400px]">
              <Table>
                <TableHeader className="sticky top-0 z-10 bg-background shadow-xs">
                  {table.getHeaderGroups().map((headerGroup) => (
                    <TableRow key={headerGroup.id} className="border-b">
                      {headerGroup.headers.map((header) => (
                        <TableHead
                          key={header.id}
                          colSpan={header.colSpan}
                          className="text-xs font-semibold uppercase tracking-wider text-muted-foreground h-10"
                        >
                          {header.isPlaceholder
                            ? null
                            : flexRender(header.column.columnDef.header, header.getContext())}
                        </TableHead>
                      ))}
                    </TableRow>
                  ))}
                </TableHeader>
                <TableBody>
                  {table.getRowModel().rows?.length ? (
                    table.getRowModel().rows.map((row) => (
                      <TableRow
                        key={row.id}
                        data-state={row.getIsSelected() && "selected"}
                        className="hover:bg-muted/50 transition-colors"
                      >
                        {row.getVisibleCells().map((cell) => (
                          <TableCell key={cell.id} className="py-3">
                            {flexRender(cell.column.columnDef.cell, cell.getContext())}
                          </TableCell>
                        ))}
                      </TableRow>
                    ))
                  ) : (
                    <TableRow>
                      <TableCell colSpan={columns.length} className="h-48 text-center text-muted-foreground">
                        {rawTransactions ? "No transactions found." : "Loading..."}
                      </TableCell>
                    </TableRow>
                  )}
                </TableBody>
              </Table>
            </div>

            <div className="flex flex-col sm:flex-row items-center justify-between px-4 py-3 border-t bg-muted/20 gap-4">
              <div className="text-sm text-muted-foreground">
                {table.getFilteredSelectedRowModel().rows.length} of {table.getFilteredRowModel().rows.length} row(s)
                selected.
              </div>

              <div className="flex items-center gap-6">
                <div className="flex items-center gap-2">
                  <p className="text-sm font-medium">Rows per page</p>
                  <Select
                    value={`${table.getState().pagination.pageSize}`}
                    onValueChange={(value) => table.setPageSize(Number(value || 15))}
                  >
                    <SelectTrigger className="h-8 w-[70px]">
                      <SelectValue placeholder={table.getState().pagination.pageSize} />
                    </SelectTrigger>
                    <SelectContent side="top">
                      {[10, 15, 20, 30, 40, 50].map((pageSize) => (
                        <SelectItem key={pageSize} value={`${pageSize}`}>
                          {pageSize}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>

                <div className="flex items-center justify-center text-sm font-medium w-[100px]">
                  Page {table.getState().pagination.pageIndex + 1} of {table.getPageCount() || 1}
                </div>

                <div className="flex items-center gap-1">
                  <Button
                    variant="outline"
                    className="hidden h-8 w-8 p-0 lg:flex"
                    onClick={() => table.setPageIndex(0)}
                    disabled={!table.getCanPreviousPage()}
                  >
                    <span className="sr-only">Go to first page</span>
                    <ChevronsLeftIcon className="h-4 w-4" />
                  </Button>
                  <Button
                    variant="outline"
                    className="h-8 w-8 p-0"
                    onClick={() => table.previousPage()}
                    disabled={!table.getCanPreviousPage()}
                  >
                    <span className="sr-only">Go to previous page</span>
                    <ChevronLeftIcon className="h-4 w-4" />
                  </Button>
                  <Button
                    variant="outline"
                    className="h-8 w-8 p-0"
                    onClick={() => table.nextPage()}
                    disabled={!table.getCanNextPage()}
                  >
                    <span className="sr-only">Go to next page</span>
                    <ChevronRightIcon className="h-4 w-4" />
                  </Button>
                  <Button
                    variant="outline"
                    className="hidden h-8 w-8 p-0 lg:flex"
                    onClick={() => table.setPageIndex(table.getPageCount() - 1)}
                    disabled={!table.getCanNextPage()}
                  >
                    <span className="sr-only">Go to last page</span>
                    <ChevronsRightIcon className="h-4 w-4" />
                  </Button>
                </div>
              </div>
            </div>
          </Tabs>
        </div>
      </div>

      {selectedTxn && (
        <SplitDialog
          open={splitDialogOpen}
          onOpenChange={setSplitDialogOpen}
          transactionId={selectedTxn.id}
          totalAmount={selectedTxn.amount}
        />
      )}
    </>
  );
}
