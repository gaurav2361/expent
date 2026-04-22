"use client";

import { useState, useCallback } from "react";
import { toast } from "@expent/ui/components/goey-toaster";
import type { TypedProcessedOcr } from "@expent/types";
import { api } from "@/lib/api-client";

export type UploadStepStatus = "pending" | "in-progress" | "completed" | "failed";

export interface UploadStep {
  id: string;
  label: string;
  status: UploadStepStatus;
}

export function useOcrUpload() {
  const [isUploading, setIsUploading] = useState(false);
  const [uploadSteps, setUploadSteps] = useState<UploadStep[]>([]);
  const [processedOcr, setProcessedOcr] = useState<TypedProcessedOcr | null>(null);

  const uploadFile = useCallback(async (file: File) => {
    setIsUploading(true);
    setProcessedOcr(null);

    const steps: UploadStep[] = [
      { id: "1", label: "Uploading file…", status: "in-progress" },
      { id: "2", label: "Classifying document…", status: "pending" },
      { id: "3", label: "Extracting transaction data…", status: "pending" },
    ];
    setUploadSteps(steps);

    try {
      const formData = new FormData();
      formData.append("file", file);

      // Using the base fetch for multipart upload since api.post expects JSON by default
      // and we want to ensure credentials/headers are handled correctly for the proxy.
      const uploadRes = await fetch("/api/upload", {
        method: "POST",
        body: formData,
      });

      if (!uploadRes.ok) {
        const errorBody = await uploadRes.text().catch(() => "Upload failed");
        throw new Error(errorBody || "Upload failed");
      }
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
      toast.success("Data extracted successfully! Please review.");
      setTimeout(() => setIsUploading(false), 1000);
      return result;
    } catch (error) {
      console.error(error);
      setUploadSteps((prev) => prev.map((s) => (s.status === "in-progress" ? { ...s, status: "failed" } : s)));
      toast.error(error instanceof Error ? error.message : "Upload or processing failed.");
      setTimeout(() => setIsUploading(false), 2000);
      return null;
    }
  }, []);

  const reset = useCallback(() => {
    setIsUploading(false);
    setUploadSteps([]);
    setProcessedOcr(null);
  }, []);

  return {
    isUploading,
    uploadSteps,
    processedOcr,
    uploadFile,
    setProcessedOcr,
    reset,
  };
}
