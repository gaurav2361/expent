"use client";

import * as React from "react";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@expent/ui/components/dialog";
import { Button } from "@expent/ui/components/button";
import { Input } from "@expent/ui/components/input";
import { toast } from "@expent/ui/components/goey-toaster";
import { CameraIcon, FileUpIcon, Loader2Icon, SparklesIcon } from "lucide-react";
import { useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api-client";
import { ProgressTracker } from "@/components/tool-ui/progress-tracker";
import { ReviewTransactionForm } from "./review-transaction-form";
import type { TypedProcessedOcr } from "@expent/types";

export function GlobalOCRDialog({ open, onOpenChange }: { open: boolean; onOpenChange: (o: boolean) => void }) {
  const [file, setFile] = React.useState<File | null>(null);
  const [isUploading, setIsUploading] = React.useState(false);
  const [uploadSteps, setUploadSteps] = React.useState<
    { id: string; label: string; status: "pending" | "in-progress" | "completed" | "failed" }[]
  >([]);
  const [processedOcr, setProcessedOcr] = React.useState<TypedProcessedOcr | null>(null);
  const [isSaving, setIsSaving] = React.useState(false);
  const queryClient = useQueryClient();

  const handleUpload = async (selectedFile: File) => {
    setIsUploading(true);
    setProcessedOcr(null);

    const steps: { id: string; label: string; status: "pending" | "in-progress" | "completed" | "failed" }[] = [
      { id: "1", label: "Uploading receipt…", status: "in-progress" as const },
      { id: "2", label: "Analyzing image…", status: "pending" },
      { id: "3", label: "Extracting data…", status: "pending" },
    ];
    setUploadSteps(steps);

    try {
      const formData = new FormData();
      formData.append("file", selectedFile);

      const API_BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL || "http://localhost:7878";
      const uploadRes = await fetch(`${API_BASE_URL}/api/upload`, {
        method: "POST",
        body: formData,
        credentials: "include",
      });

      if (!uploadRes.ok) throw new Error("Upload failed");
      const { key } = await uploadRes.json();

      setUploadSteps((prev) =>
        prev.map((s) =>
          s.id === "1" ? { ...s, status: "completed" } : s.id === "2" ? { ...s, status: "in-progress" } : s,
        ),
      );

      const result = await api.post<TypedProcessedOcr>("/api/ocr/process", { key });

      setUploadSteps((prev) =>
        prev.map((s) =>
          s.id === "2" ? { ...s, status: "completed" } : s.id === "3" ? { ...s, status: "in-progress" } : s,
        ),
      );

      setUploadSteps((prev) => prev.map((s) => (s.id === "3" ? { ...s, status: "completed" } : s)));
      setProcessedOcr(result);
      toast.success("Receipt scanned successfully!");
    } catch (error) {
      console.error(error);
      toast.error("Failed to process receipt");
      onOpenChange(false);
    } finally {
      setIsUploading(false);
    }
  };

  const handleConfirm = async (finalData: TypedProcessedOcr) => {
    setIsSaving(true);
    try {
      await api.post("/api/transactions/from-ocr", finalData);
      queryClient.invalidateQueries({ queryKey: ["transactions"] });
      queryClient.invalidateQueries({ queryKey: ["wallets"] });
      toast.success("Transaction saved!");
      onOpenChange(false);
      setProcessedOcr(null);
      setFile(null);
    } catch (error) {
      toast.error("Failed to save transaction");
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <Dialog
      open={open}
      onOpenChange={(val) => {
        if (!isUploading && !isSaving) {
          onOpenChange(val);
          if (!val) {
            setProcessedOcr(null);
            setFile(null);
          }
        }
      }}
    >
      <DialogContent className={processedOcr ? "sm:max-w-3xl" : "sm:max-w-md"}>
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <SparklesIcon className="h-5 w-5 text-primary animate-pulse" />
            Scan Receipt
          </DialogTitle>
          <DialogDescription>
            Upload a receipt image to automatically extract transaction details using AI.
          </DialogDescription>
        </DialogHeader>

        {!processedOcr && !isUploading && (
          <div className="flex flex-col items-center justify-center border-2 border-dashed border-muted-foreground/20 rounded-xl p-12 transition-colors hover:border-primary/50 group cursor-pointer relative">
            <Input
              type="file"
              accept="image/*"
              className="absolute inset-0 opacity-0 cursor-pointer z-10"
              onChange={(e) => {
                const f = e.target.files?.[0];
                if (f) handleUpload(f);
              }}
            />
            <div className="bg-primary/5 size-16 rounded-full flex items-center justify-center mb-4 group-hover:scale-110 transition-transform">
              <CameraIcon className="h-8 w-8 text-primary" />
            </div>
            <p className="font-semibold text-foreground">Click or Drag to Upload</p>
            <p className="text-sm text-muted-foreground mt-1 text-center">Supports JPG, PNG and PDF receipts</p>
          </div>
        )}

        {isUploading && (
          <div className="py-8 space-y-6">
            <div className="flex flex-col items-center justify-center text-center">
              <Loader2Icon className="h-10 w-10 text-primary animate-spin mb-4" />
              <p className="font-medium">Magically extracting data...</p>
            </div>
            <ProgressTracker id="global-ocr-progress" steps={uploadSteps} />
          </div>
        )}

        {processedOcr && (
          <div className="mt-4">
            <ReviewTransactionForm
              processedOcr={processedOcr}
              onConfirm={handleConfirm}
              onCancel={() => onOpenChange(false)}
              isSubmitting={isSaving}
            />
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
