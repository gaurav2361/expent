"use client";

import * as React from "react";
import { Separator } from "@expent/ui/components/separator";
import { Button } from "@expent/ui/components/button";
import { useCategories } from "@/hooks/use-categories";
import { CreateCategoryDialog, ICON_MAP, COLOR_PALETTE } from "@/components/categories/create-category-dialog";
import { toast } from "@expent/ui/components/goey-toaster";
import { PlusIcon, Trash2Icon, TagIcon, LockIcon } from "lucide-react";
import type { Category } from "@expent/types";

export default function SettingsCategoriesPage() {
  const { categories, isLoading, deleteMutation } = useCategories();
  const [createOpen, setCreateOpen] = React.useState(false);

  const systemCategories = categories?.filter((c) => c.user_id === "system") ?? [];
  const userCategories = categories?.filter((c) => c.user_id !== "system") ?? [];

  const handleDelete = (cat: Category) => {
    if (cat.user_id === "system") {
      toast.error("System categories cannot be deleted.");
      return;
    }
    deleteMutation.mutate(cat.id, {
      onSuccess: () => toast.success(`"${cat.name}" deleted.`),
      onError: (err) => toast.error(err.message || "Failed to delete category"),
    });
  };

  const getIcon = (iconName: string | null) => {
    if (!iconName) return TagIcon;
    return ICON_MAP[iconName] || TagIcon;
  };

  const getColor = (colorHex: string | null) => {
    return colorHex || "#64748b";
  };

  return (
    <div className="space-y-6 w-full max-w-2xl">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-medium">Categories</h3>
          <p className="text-sm text-muted-foreground">
            Manage the categories used to tag your transactions.
          </p>
        </div>
        <Button size="sm" onClick={() => setCreateOpen(true)}>
          <PlusIcon className="mr-2 h-4 w-4" /> New Category
        </Button>
      </div>
      <Separator />

      {isLoading ? (
        <div className="space-y-3">
          {[1, 2, 3].map((i) => (
            <div key={i} className="h-16 animate-pulse rounded-lg bg-muted/50" />
          ))}
        </div>
      ) : (
        <div className="space-y-6">
          {/* System Categories */}
          {systemCategories.length > 0 && (
            <div className="space-y-3">
              <h4 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground flex items-center gap-2">
                <LockIcon className="h-3 w-3" /> System Categories
              </h4>
              <p className="text-xs text-muted-foreground">
                These are built-in categories used for core features. They cannot be modified or deleted.
              </p>
              <div className="space-y-2">
                {systemCategories.map((cat) => {
                  const Icon = getIcon(cat.icon);
                  const color = getColor(cat.color);
                  return (
                    <div
                      key={cat.id}
                      className="flex items-center justify-between rounded-lg border p-3 bg-muted/20"
                    >
                      <div className="flex items-center gap-3">
                        <div
                          className="flex size-9 items-center justify-center rounded-lg shrink-0"
                          style={{ backgroundColor: color + "20", color }}
                        >
                          <Icon className="size-4" />
                        </div>
                        <div>
                          <p className="text-sm font-medium">{cat.name}</p>
                          <p className="text-xs text-muted-foreground">System · Cannot be deleted</p>
                        </div>
                      </div>
                      <LockIcon className="h-4 w-4 text-muted-foreground/40" />
                    </div>
                  );
                })}
              </div>
            </div>
          )}

          {/* User Categories */}
          <div className="space-y-3">
            <h4 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
              Your Categories
            </h4>
            {userCategories.length === 0 ? (
              <div className="flex flex-col items-center justify-center rounded-lg border border-dashed p-8 text-center">
                <TagIcon className="h-8 w-8 text-muted-foreground/40 mb-3" />
                <p className="text-sm text-muted-foreground">No custom categories yet.</p>
                <p className="text-xs text-muted-foreground mt-1">
                  Create your first category to start organizing transactions your way.
                </p>
                <Button size="sm" variant="outline" className="mt-4" onClick={() => setCreateOpen(true)}>
                  <PlusIcon className="mr-2 h-4 w-4" /> Create Category
                </Button>
              </div>
            ) : (
              <div className="space-y-2">
                {userCategories.map((cat) => {
                  const Icon = getIcon(cat.icon);
                  const color = getColor(cat.color);
                  return (
                    <div
                      key={cat.id}
                      className="flex items-center justify-between rounded-lg border p-3 group hover:border-primary/30 transition-colors"
                    >
                      <div className="flex items-center gap-3">
                        <div
                          className="flex size-9 items-center justify-center rounded-lg shrink-0"
                          style={{ backgroundColor: color + "20", color }}
                        >
                          <Icon className="size-4" />
                        </div>
                        <div>
                          <p className="text-sm font-medium">{cat.name}</p>
                          <div className="flex items-center gap-2 mt-0.5">
                            <span
                              className="inline-block size-2.5 rounded-full"
                              style={{ backgroundColor: color }}
                            />
                            <span className="text-xs text-muted-foreground capitalize">
                              {COLOR_PALETTE.find((c) => c.hex === color)?.id || "Custom"}
                            </span>
                          </div>
                        </div>
                      </div>
                      <Button
                        variant="ghost"
                        size="icon-xs"
                        className="h-8 w-8 opacity-0 group-hover:opacity-100 transition-opacity text-muted-foreground hover:text-destructive"
                        onClick={() => handleDelete(cat)}
                        disabled={deleteMutation.isPending}
                        aria-label={`Delete ${cat.name}`}
                      >
                        <Trash2Icon className="h-4 w-4" />
                      </Button>
                    </div>
                  );
                })}
              </div>
            )}
          </div>
        </div>
      )}

      <CreateCategoryDialog open={createOpen} onOpenChange={setCreateOpen} />
    </div>
  );
}
