"use client";

import { Badge } from "@expent/ui/components/badge";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@expent/ui/components/breadcrumb";
import { Button } from "@expent/ui/components/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@expent/ui/components/card";
import { Separator } from "@expent/ui/components/separator";
import { SidebarTrigger } from "@expent/ui/components/sidebar";
import { useQuery } from "@tanstack/react-query";
import { useRouter } from "next/navigation";
import { CalendarIcon, CreditCardIcon, SparklesIcon } from "lucide-react";
import { useEffect } from "react";
import { useSession } from "@/lib/auth-client";

const API_BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL || "http://localhost:8080";

export default function SubscriptionsComponent() {
  const router = useRouter();
  const session = useSession();

  const {
    data: potentialSubs,
    isLoading,
    refetch,
  } = useQuery({
    queryKey: ["subscriptions-detect"],
    queryFn: async () => {
      const response = await fetch(`${API_BASE_URL}/api/subscriptions/detect`, {
        headers: { "Content-Type": "application/json" },
        credentials: "include",
      });
      if (!response.ok) throw new Error("Failed to detect subscriptions");
      return response.json();
    },
    enabled: !!session.data,
  });

  useEffect(() => {
    if (!session.isPending && !session.data) {
      router.push("/sign-in");
    }
  }, [session.data, session.isPending, router]);

  if (session.isPending) {
    return <div className="flex h-screen items-center justify-center">Loading session...</div>;
  }

  if (!session.data) {
    return null;
  }

  return (


      <div className="flex flex-1 flex-col gap-4 p-4 pt-0">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-bold tracking-tight">Subscriptions</h2>
            <p className="text-muted-foreground text-sm">Manage and track your recurring payments.</p>
          </div>
          <Button onClick={() => refetch()} variant="outline">
            <SparklesIcon className="mr-2 h-4 w-4" /> Run Detection
          </Button>
        </div>

        {isLoading ? (
          <div className="text-center py-20 text-muted-foreground">Scanning transactions for patterns...</div>
        ) : potentialSubs?.length === 0 ? (
          <Card className="border-dashed">
            <CardContent className="flex flex-col items-center justify-center py-20 text-center">
              <CalendarIcon className="h-12 w-12 text-muted-foreground mb-4" />
              <h3 className="text-lg font-semibold">No subscriptions detected</h3>
              <p className="text-muted-foreground max-w-xs">
                We couldn't find any recurring transaction patterns in the last 90 days.
              </p>
            </CardContent>
          </Card>
        ) : (
          <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
            {potentialSubs?.map((sub: any) => (
              <Card key={sub.id} className="relative overflow-hidden group">
                <div className="absolute top-0 left-0 w-1 h-full bg-primary" />
                <CardHeader className="pb-2">
                  <div className="flex justify-between items-start">
                    <CardTitle className="text-lg">{sub.name}</CardTitle>
                    <Badge variant="secondary">{sub.cycle}</Badge>
                  </div>
                  <CardDescription>Detected recurring pattern</CardDescription>
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold mb-4">₹{parseFloat(sub.amount).toLocaleString()}</div>
                  <div className="space-y-2 text-sm text-muted-foreground">
                    <div className="flex items-center">
                      <CalendarIcon className="mr-2 h-4 w-4" />
                      Next: {new Date(sub.next_charge_date).toLocaleDateString()}
                    </div>
                    <div className="flex items-center">
                      <CreditCardIcon className="mr-2 h-4 w-4" />
                      Auto-tracked from ledger
                    </div>
                  </div>
                  <div className="mt-4 pt-4 border-t flex gap-2">
                    <Button size="sm" className="flex-1">
                      Confirm Sub
                    </Button>
                    <Button size="sm" variant="ghost">
                      Ignore
                    </Button>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        )}
      </div>
  );
}
