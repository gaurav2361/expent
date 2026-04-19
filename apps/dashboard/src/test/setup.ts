import "@testing-library/jest-dom";
import { vi } from "vitest";

// Mock global fetch
global.fetch = vi.fn();

// Mock toast
vi.mock("@expent/ui/components/goey-toaster", () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
  },
}));

// Mock Next.js navigation if needed
vi.mock("next/navigation", () => ({
  useRouter: () => ({
    push: vi.fn(),
    replace: vi.fn(),
    prefetch: vi.fn(),
  }),
  useSearchParams: () => new URLSearchParams(),
  usePathname: () => "/",
}));
