import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@expent/ui/components/breadcrumb";
import { Separator } from "@expent/ui/components/separator";
import { SidebarInset, SidebarProvider, SidebarTrigger } from "@expent/ui/components/sidebar";
import { Button } from "@expent/ui/components/button";
import { Input } from "@expent/ui/components/input";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@expent/ui/components/table";
import { Badge } from "@expent/ui/components/badge";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@expent/ui/components/card";
import { createFileRoute } from "@tanstack/react-router";
import { AppSidebar } from "@/components/app-sidebar";
import { useState, useMemo } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { ReceiptTextIcon, SparklesIcon, Share2Icon } from "lucide-react";

export const Route = createFileRoute("/dashboard/")({
  component: RouteComponent,
});

const API_BASE_URL = import.meta.env.VITE_AUTH_BASE_URL || "http://localhost:3001";

function RouteComponent() {
  const [file, setFile] = useState<File | null>(null);
  const [isUploading, setIsLoading] = useState(false);
  const [ocrResult, setOcrResult] = useState<any>(null);
  const queryClient = useQueryClient();

  // 1. Fetch Transactions
  const { data: transactions, isLoading: isTxnsLoading } = useQuery({
    queryKey: ["transactions"],
    queryFn: async () => {
      const response = await fetch(`${API_BASE_URL}/api/transactions`, {
          headers: { "Content-Type": "application/json" },
          credentials: "include",
      });
      if (!response.ok) throw new Error("Failed to fetch transactions");
      return response.json();
    },
  });

  // 2. Fetch Pending P2P Requests
  const { data: p2pRequests } = useQuery({
    queryKey: ["p2p-pending"],
    queryFn: async () => {
      const response = await fetch(`${API_BASE_URL}/api/p2p/pending`, {
          headers: { "Content-Type": "application/json" },
          credentials: "include",
      });
      if (!response.ok) throw new Error("Failed to fetch P2P requests");
      return response.json();
    },
  });

  // Mutations
  const acceptMutation = useMutation({
    mutationFn: async (requestId: string) => {
        const response = await fetch(`${API_BASE_URL}/api/p2p/accept`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ request_id: requestId }),
            credentials: "include",
        });
        if (!response.ok) throw new Error("Failed to accept request");
        return response.json();
    },
    onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: ["transactions"] });
        queryClient.invalidateQueries({ queryKey: ["p2p-pending"] });
    }
  });

  // Derived State
  const totalBalance = useMemo(() => {
    if (!transactions) return 0;
    return transactions.reduce((acc: number, txn: any) => {
        const amount = parseFloat(txn.amount);
        return txn.direction === "IN" ? acc + amount : acc - amount;
    }, 0);
  }, [transactions]);

  const handleUpload = async () => {
    if (!file) return;
    setIsLoading(true);
    try {
      // In a real flow, we'd upload to R2 then process. 
      // For demo, we trigger the process-ocr with mock data if needed, 
      // but let's assume the backend handles the full logic.
      const response = await fetch(`${API_BASE_URL}/api/process-ocr`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ 
            raw_text: "Mock Receipt\nCoffee 150.00\nSandwich 350.00\nTOTAL 500.00", 
            amount: "500.00",
            date: new Date().toISOString(),
            items: [
                { name: "Coffee", quantity: 1, price: "150.00" },
                { name: "Sandwich", quantity: 1, price: "350.00" }
            ]
        }),
        credentials: "include",
      });
      if (!response.ok) throw new Error("Processing failed");
      const result = await response.json();
      setOcrResult(result);
      queryClient.invalidateQueries({ queryKey: ["transactions"] });
    } catch (error) {
      console.error(error);
      alert("Upload or processing failed");
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset>
        <header className="flex h-16 shrink-0 items-center gap-2">
          <div className="flex items-center gap-2 px-4">
            <SidebarTrigger className="-ml-1" />
            <Separator orientation="vertical" className="mr-2 data-[orientation=vertical]:h-4" />
            <Breadcrumb>
              <BreadcrumbList>
                <BreadcrumbItem className="hidden md:block">
                  <BreadcrumbLink href="/dashboard">Dashboard</BreadcrumbLink>
                </BreadcrumbItem>
                <BreadcrumbSeparator className="hidden md:block" />
                <BreadcrumbItem>
                  <BreadcrumbPage>Overview</BreadcrumbPage>
                </BreadcrumbItem>
              </BreadcrumbList>
            </Breadcrumb>
          </div>
        </header>
        
        <div className="flex flex-1 flex-col gap-4 p-4 pt-0">
          <div className="grid auto-rows-min gap-4 md:grid-cols-3">
            <Card>
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                    <CardTitle className="text-sm font-medium">Pending Approvals</CardTitle>
                </CardHeader>
                <CardContent>
                    <div className="text-2xl font-bold">{p2pRequests?.length || 0}</div>
                </CardContent>
            </Card>
            <Card>
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                    <CardTitle className="text-sm font-medium">Quick Upload</CardTitle>
                </CardHeader>
                <CardContent className="flex gap-2">
                    <Input type="file" onChange={(e) => setFile(e.target.files?.[0] || null)} className="h-8 text-xs" />
                    <Button onClick={handleUpload} disabled={!file || isLoading} size="sm">
                        {isLoading ? "..." : "Go"}
                    </Button>
                </CardContent>
            </Card>
            <Card className="bg-primary text-primary-foreground shadow-lg">
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                    <CardTitle className="text-sm font-medium">Net Balance</CardTitle>
                </CardHeader>
                <CardContent>
                    <div className="text-2xl font-bold">₹ {totalBalance.toLocaleString(undefined, { minimumFractionDigits: 2 })}</div>
                </CardContent>
            </Card>
          </div>

          {/* OCR Result / Itemized Parsing Section */}
          {ocrResult && (
            <Card className="border-primary/20 bg-primary/5 animate-in zoom-in-95 duration-300">
                <CardHeader className="flex flex-row items-center gap-4">
                    <div className="bg-primary/10 p-2 rounded-lg">
                        <ReceiptTextIcon className="h-6 w-6 text-primary" />
                    </div>
                    <div>
                        <CardTitle>Receipt Itemized</CardTitle>
                        <CardDescription>We've extracted {ocrResult.items?.length || 0} items from your upload.</CardDescription>
                    </div>
                    <Button variant="ghost" size="sm" className="ml-auto" onClick={() => setOcrResult(null)}>Dismiss</Button>
                </CardHeader>
                <CardContent>
                    <div className="space-y-4">
                        <div className="rounded-md border bg-background">
                            <Table>
                                <TableHeader>
                                    <TableRow>
                                        <TableHead>Item</TableHead>
                                        <TableHead className="text-right">Qty</TableHead>
                                        <TableHead className="text-right">Price</TableHead>
                                        <TableHead className="text-right">Action</TableHead>
                                    </TableRow>
                                </TableHeader>
                                <TableBody>
                                    {/* Mocking items for the UI demo if not returned by server yet */}
                                    {(ocrResult.items || [
                                        { name: "Coffee", quantity: 1, price: "150.00" },
                                        { name: "Sandwich", quantity: 1, price: "350.00" }
                                    ]).map((item: any, i: number) => (
                                        <TableRow key={i}>
                                            <TableCell className="font-medium">{item.name}</TableCell>
                                            <TableCell className="text-right">{item.quantity}</TableCell>
                                            <TableCell className="text-right font-mono">₹{item.price}</TableCell>
                                            <TableCell className="text-right">
                                                <Button size="sm" variant="ghost">
                                                    <Share2Icon className="h-3 w-3 mr-1" /> Split
                                                </Button>
                                            </TableCell>
                                        </TableRow>
                                    ))}
                                </TableBody>
                            </Table>
                        </div>
                        <div className="flex justify-end gap-2">
                            <Button variant="outline" size="sm">
                                <SparklesIcon className="h-3 w-3 mr-1" /> Auto-Categorize
                            </Button>
                            <Button size="sm">Confirm All</Button>
                        </div>
                    </div>
                </CardContent>
            </Card>
          )}

          {/* Pending P2P Requests Section */}
          {p2pRequests && p2pRequests.length > 0 && (
            <Card className="border-orange-200 bg-orange-50 dark:bg-orange-950/20 animate-in fade-in slide-in-from-top-4 duration-500">
                <CardHeader>
                    <CardTitle className="text-orange-800 dark:text-orange-300 text-base">Action Required: P2P Requests</CardTitle>
                </CardHeader>
                <CardContent>
                    <div className="space-y-4">
                        {p2pRequests.map((req: any) => (
                            <div key={req.id} className="flex items-center justify-between border-b border-orange-100 dark:border-orange-900/50 pb-2 last:border-0">
                                <div>
                                    <p className="font-semibold text-sm">{req.sender_user_id} shared a transaction</p>
                                    <p className="text-xs text-muted-foreground">Amount: ₹{parseFloat(req.transaction_data.amount).toLocaleString()}</p>
                                </div>
                                <Button size="sm" variant="outline" className="border-orange-300 hover:bg-orange-100" onClick={() => acceptMutation.mutate(req.id)}>
                                    {acceptMutation.isPending ? "Accepting..." : "Merge & Accept"}
                                </Button>
                            </div>
                        ))}
                    </div>
                </CardContent>
            </Card>
          )}

          {/* Recent Transactions Table */}
          <Card className="flex-1 overflow-hidden">
            <CardHeader className="px-6 py-4">
                <CardTitle>Recent Transactions</CardTitle>
            </CardHeader>
            <CardContent className="p-0">
                <Table>
                    <TableHeader className="bg-muted/50">
                        <TableRow>
                            <TableHead className="px-6">Date</TableHead>
                            <TableHead>Direction</TableHead>
                            <TableHead>Amount</TableHead>
                            <TableHead>Source</TableHead>
                            <TableHead className="text-right px-6">Status</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {isTxnsLoading ? (
                            <TableRow><TableCell colSpan={5} className="text-center py-10">Loading transactions...</TableCell></TableRow>
                        ) : transactions?.length === 0 ? (
                            <TableRow><TableCell colSpan={5} className="text-center py-10 text-muted-foreground">No transactions found. Start by uploading a receipt!</TableCell></TableRow>
                        ) : transactions?.map((txn: any) => (
                            <TableRow key={txn.id} className="hover:bg-muted/30 transition-colors">
                                <TableCell className="px-6 text-xs font-medium text-muted-foreground">
                                    {new Date(txn.date).toLocaleDateString(undefined, { day: '2-digit', month: 'short', year: 'numeric' })}
                                </TableCell>
                                <TableCell>
                                    <Badge variant={txn.direction === "IN" ? "secondary" : "destructive"} className="uppercase text-[10px] font-bold tracking-wider">
                                        {txn.direction}
                                    </Badge>
                                </TableCell>
                                <TableCell className="font-mono font-bold text-sm">
                                    ₹{parseFloat(txn.amount).toLocaleString(undefined, { minimumFractionDigits: 2 })}
                                </TableCell>
                                <TableCell className="text-xs text-muted-foreground italic">{txn.source}</TableCell>
                                <TableCell className="text-right px-6">
                                    <Badge variant="outline" className="text-[10px] uppercase font-semibold">{txn.status}</Badge>
                                </TableCell>
                            </TableRow>
                        ))}
                    </TableBody>
                </Table>
            </CardContent>
          </Card>
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
}
