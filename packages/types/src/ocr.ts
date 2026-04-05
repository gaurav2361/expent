import type { GPayExtraction, OcrResult, ProcessedOcr } from "./db";

export type TypedProcessedOcr =
  | {
      doc_type: "GPAY";
      data: GPayExtraction;
      r2_key: string | null;
    }
  | {
      doc_type: "GENERIC";
      data: OcrResult;
      r2_key: string | null;
    };

export type UnifiedProcessedOcr = ProcessedOcr & {
  data: GPayExtraction | OcrResult;
};
