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
import { createFileRoute } from "@tanstack/react-router";
import { AppSidebar } from "@/components/app-sidebar";
import { useState } from "react";

export const Route = createFileRoute("/dashboard/")({
  component: RouteComponent,
});

function RouteComponent() {
  const [file, setFile] = useState<File | null>(null);
  const [isUploading, setIsLoading] = useState(false);
  const [result, setResult] = useState<any>(null);

  const handleUpload = async () => {
    if (!file) return;

    setIsLoading(true);
    try {
      // 1. Get presigned URL
      const response = await fetch("http://localhost:8000/upload/presigned", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          contentType: file.type,
          fileName: file.name,
        }),
      });

      if (!response.ok) throw new Error("Failed to get presigned URL");
      const { url, key } = await response.json();

      // 2. Upload to R2
      const uploadResponse = await fetch(url, {
        method: "PUT",
        body: file,
        headers: {
          "Content-Type": file.type,
        },
      });

      if (!uploadResponse.ok) throw new Error("Failed to upload to R2");

      // 3. Trigger processing
      const processResponse = await fetch("http://localhost:8000/process", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          key,
          fileType: file.type.includes("pdf") ? "pdf" : "image",
        }),
      });

      if (!processResponse.ok) throw new Error("Failed to process file");
      const processResult = await processResponse.json();
      setResult(processResult);
      
      alert(`File processed! Extracted text: ${processResult.raw_text.substring(0, 100)}...`);
      
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
                  <BreadcrumbPage>Upload Receipts</BreadcrumbPage>
                </BreadcrumbItem>
              </BreadcrumbList>
            </Breadcrumb>
          </div>
        </header>
        <div className="flex flex-1 flex-col gap-4 p-4 pt-0">
          <div className="grid auto-rows-min gap-4 md:grid-cols-3">
            <div className="aspect-video rounded-xl bg-muted/50 p-4 flex flex-col items-center justify-center gap-4">
                <h3 className="font-semibold">Upload Screenshot / PDF</h3>
                <Input 
                    type="file" 
                    onChange={(e) => setFile(e.target.files?.[0] || null)}
                    accept="image/*,application/pdf"
                />
                <Button 
                    onClick={handleUpload} 
                    disabled={!file || isUploading}
                    className="w-full"
                >
                    {isUploading ? "Processing..." : "Upload & Process"}
                </Button>
            </div>
            {result && (
                <div className="col-span-2 aspect-video rounded-xl bg-muted/50 p-4 overflow-auto">
                    <h3 className="font-semibold mb-2">Processing Result</h3>
                    <pre className="text-xs whitespace-pre-wrap">{JSON.stringify(result, null, 2)}</pre>
                </div>
            )}
          </div>
          <div className="min-h-[100vh] flex-1 rounded-xl bg-muted/50 md:min-h-min" />
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
}
