"use client";

import * as React from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@expent/ui/components/card";
import { Button } from "@expent/ui/components/button";
import { Badge } from "@expent/ui/components/badge";
import { Progress } from "@expent/ui/components/progress";
import {
  UploadIcon,
  CheckCircle2Icon,
  AlertCircleIcon,
  HistoryIcon,
  FileTextIcon,
  ArrowRightIcon,
  CheckIcon,
  XIcon,
} from "lucide-react";
import { toast } from "@expent/ui/components/goey-toaster";

export default function ReconciliationPage() {
  const [file, setFile] = React.useState<File | null>(null);
  const [isUploading, setIsUploading] = React.useState(false);
  const [showResults, setShowResults] = React.useState(false);

  const handleUpload = () => {
    if (!file) return;
    setIsUploading(true);
    // Simulate upload and matching
    setTimeout(() => {
      setIsUploading(false);
      setShowResults(true);
      toast.success("Statement processed! Found 12 matches.");
    }, 2000);
  };

  return (
    <div className="flex flex-1 flex-col gap-6 p-4 md:p-6 lg:p-8 max-w-7xl mx-auto w-full">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">Bank Reconciliation</h1>
          <p className="text-muted-foreground text-sm">Match your bank statements with recorded transactions.</p>
        </div>
        <Button variant="outline" size="sm">
          <HistoryIcon className="mr-2 h-4 w-4" /> View History
        </Button>
      </div>

      {!showResults ? (
        <Card className="border-dashed bg-muted/5">
          <CardContent className="flex flex-col items-center justify-center py-16 text-center gap-4">
            <div className="size-16 rounded-full bg-primary/10 flex items-center justify-center text-primary mb-2">
              <UploadIcon className="h-8 w-8" />
            </div>
            <div className="space-y-1">
              <h3 className="text-lg font-semibold">Upload Bank Statement</h3>
              <p className="text-sm text-muted-foreground max-w-sm">
                Drop your CSV or PDF statement here. We'll automatically identify matches and highlight discrepancies.
              </p>
            </div>
            <div className="flex flex-col items-center gap-2">
              <Input
                type="file"
                className="hidden"
                id="statement-upload"
                onChange={(e) => setFile(e.target.files?.[0] || null)}
              />
              <Label
                htmlFor="statement-upload"
                className="inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 border border-input bg-background hover:bg-accent hover:text-accent-foreground h-10 px-4 py-2 cursor-pointer"
              >
                {file ? file.name : "Select File"}
              </Label>
              {file && (
                <Button onClick={handleUpload} disabled={isUploading} className="w-full">
                  {isUploading ? "Processing…" : "Start Matching"}
                </Button>
              )}
            </div>
            {isUploading && (
              <div className="w-full max-w-xs mt-4">
                <Progress value={66} className="h-2" />
                <p className="text-[10px] text-muted-foreground mt-2 italic">
                  Scanning rows & computing confidence scores…
                </p>
              </div>
            )}
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500">
          <div className="grid gap-4 md:grid-cols-3">
            <Card className="bg-green-50/50 dark:bg-green-500/5 border-green-100 dark:border-green-500/20">
              <CardHeader className="p-4 pb-2">
                <CardTitle className="text-xs font-semibold uppercase text-green-700 dark:text-green-400">
                  Matched
                </CardTitle>
              </CardHeader>
              <CardContent className="p-4 pt-0">
                <div className="text-2xl font-bold">12</div>
                <p className="text-[10px] text-muted-foreground mt-1">High confidence auto-links</p>
              </CardContent>
            </Card>
            <Card className="bg-orange-50/50 dark:bg-orange-500/5 border-orange-100 dark:border-orange-500/20">
              <CardHeader className="p-4 pb-2">
                <CardTitle className="text-xs font-semibold uppercase text-orange-700 dark:text-orange-400">
                  Needs Review
                </CardTitle>
              </CardHeader>
              <CardContent className="p-4 pt-0">
                <div className="text-2xl font-bold">3</div>
                <p className="text-[10px] text-muted-foreground mt-1">Multiple potential matches</p>
              </CardContent>
            </Card>
            <Card className="bg-rose-50/50 dark:bg-rose-500/5 border-rose-100 dark:border-rose-500/20">
              <CardHeader className="p-4 pb-2">
                <CardTitle className="text-xs font-semibold uppercase text-rose-700 dark:text-rose-400">
                  Missing
                </CardTitle>
              </CardHeader>
              <CardContent className="p-4 pt-0">
                <div className="text-2xl font-bold">1</div>
                <p className="text-[10px] text-muted-foreground mt-1">No transaction found in Expent</p>
              </CardContent>
            </Card>
          </div>

          <div className="space-y-4">
            <h2 className="text-lg font-semibold flex items-center gap-2">
              <AlertCircleIcon className="h-5 w-5 text-orange-500" /> Pending Matches
            </h2>

            <ReconciliationRow
              bankRow={{ date: "2026-03-12", desc: "GOOGLE *SERVICES G.CO", amount: 699.0 }}
              match={{ source: "Google One Subscription", amount: 699.0, confidence: 98 }}
            />

            <ReconciliationRow
              bankRow={{ date: "2026-03-14", desc: "ZOMATO*ORDER 1234", amount: 452.5 }}
              match={{ source: "Zomato Dinner", amount: 452.5, confidence: 95 }}
            />

            <ReconciliationRow bankRow={{ date: "2026-03-15", desc: "ATM WITHDRAWAL", amount: 2000.0 }} isMissing />
          </div>

          <div className="flex justify-center pt-4">
            <Button variant="ghost" onClick={() => setShowResults(false)}>
              Upload another statement
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}

function ReconciliationRow({ bankRow, match, isMissing }: { bankRow: any; match?: any; isMissing?: boolean }) {
  return (
    <Card className="overflow-hidden border-l-4 border-l-muted">
      <div className="flex flex-col md:flex-row">
        {/* Bank Side */}
        <div className="flex-1 p-4 bg-muted/10">
          <div className="flex items-center gap-2 text-[10px] text-muted-foreground uppercase font-bold tracking-wider mb-1">
            <FileTextIcon className="h-3 w-3" /> Bank Statement
          </div>
          <p className="text-sm font-medium truncate">{bankRow.desc}</p>
          <div className="flex justify-between items-end mt-2">
            <span className="text-[11px] text-muted-foreground">{bankRow.date}</span>
            <span className="font-mono font-bold">₹{bankRow.amount.toLocaleString()}</span>
          </div>
        </div>

        {/* Transition */}
        <div className="flex items-center justify-center px-4 bg-background">
          <ArrowRightIcon className="h-5 w-5 text-muted-foreground/30 rotate-90 md:rotate-0" />
        </div>

        {/* App Side */}
        <div className={`flex-1 p-4 ${isMissing ? "bg-rose-50/30 dark:bg-rose-500/5" : "bg-primary/5"}`}>
          <div className="flex items-center justify-between mb-1">
            <div className="flex items-center gap-2 text-[10px] text-muted-foreground uppercase font-bold tracking-wider">
              <CheckCircle2Icon className="h-3 w-3" /> Expent Entry
            </div>
            {match && (
              <Badge variant="outline" className="h-4 text-[9px] bg-green-50 text-green-700 border-green-200">
                {match.confidence}% Match
              </Badge>
            )}
          </div>

          {isMissing ? (
            <div className="h-full flex flex-col justify-center">
              <p className="text-sm text-rose-600 font-medium italic">No matching transaction found</p>
              <Button variant="link" className="h-auto p-0 text-xs justify-start mt-1">
                Create missing transaction?
              </Button>
            </div>
          ) : (
            <>
              <p className="text-sm font-medium truncate">{match.source}</p>
              <div className="flex justify-between items-end mt-2">
                <span className="text-[11px] text-muted-foreground">Recorded {bankRow.date}</span>
                <span className="font-mono font-bold">₹{match.amount.toLocaleString()}</span>
              </div>
            </>
          )}
        </div>

        {/* Actions */}
        {!isMissing && (
          <div className="flex md:flex-col border-t md:border-t-0 md:border-l p-2 gap-2 bg-muted/5">
            <Button
              size="icon-sm"
              className="flex-1 rounded-full h-9 w-9 bg-green-600 hover:bg-green-700"
              aria-label="Confirm match"
            >
              <CheckIcon className="h-4 w-4" />
            </Button>
            <Button variant="outline" size="icon-sm" className="flex-1 rounded-full h-9 w-9" aria-label="Reject match">
              <XIcon className="h-4 w-4" />
            </Button>
          </div>
        )}
      </div>
    </Card>
  );
}

// Needed for the hidden input
import { Input } from "@expent/ui/components/input";
import { Label } from "@expent/ui/components/label";
