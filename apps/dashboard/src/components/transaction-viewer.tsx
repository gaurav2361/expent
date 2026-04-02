import * as React from "react";
import {
  Drawer,
  DrawerClose,
  DrawerContent,
  DrawerDescription,
  DrawerHeader,
  DrawerTitle,
  DrawerTrigger,
  DrawerFooter,
} from "@expent/ui/components/drawer";
import { Button } from "@expent/ui/components/button";
import { Badge } from "@expent/ui/components/badge";
import { Separator } from "@expent/ui/components/separator";
import { Input } from "@expent/ui/components/input";
import { Label } from "@expent/ui/components/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@expent/ui/components/select";
import { useIsMobile } from "@expent/ui/hooks/use-mobile";

export interface Transaction {
  id: string;
  date: string;
  direction: "IN" | "OUT" | string;
  amount: string;
  source: string;
  category?: string;
  status?: string;
}

interface TransactionViewerProps {
  item: Transaction;
  onUpdate: (id: string, data: Partial<Transaction>) => void;
}

export function TransactionViewer({
  item,
  onUpdate,
}: TransactionViewerProps) {
  const isMobile = useIsMobile();
  const [source, setSource] = React.useState(item.source);
  const [category, setCategory] = React.useState(item.category || "Uncategorized");
  const [status, setStatus] = React.useState(item.status || "Completed");
  const [amount, setAmount] = React.useState(item.amount);

  const title = source || "Unknown Source";
  const formattedDate = new Date(item.date).toLocaleDateString("en-IN", {
    year: "numeric",
    month: "long",
    day: "numeric",
  });

  return (
    <Drawer direction={isMobile ? "bottom" : "right"}>
      <DrawerTrigger asChild>
        <Button variant="link" className="w-fit px-0 text-left text-foreground truncate max-w-[200px] block font-normal">
          {title}
        </Button>
      </DrawerTrigger>
      <DrawerContent className={isMobile ? "h-[80vh]" : "h-full w-[400px] ml-auto top-0"}>
        <DrawerHeader className="gap-1 text-left">
          <DrawerTitle className="text-xl">{title}</DrawerTitle>
          <DrawerDescription>
            Transaction from {formattedDate}
          </DrawerDescription>
        </DrawerHeader>
        <div className="flex flex-col gap-4 overflow-y-auto px-4 text-sm mt-4">
          <div className="flex items-center justify-between p-4 bg-muted rounded-xl border">
            <div>
              <div className="text-sm text-muted-foreground">Amount</div>
              <div className={`text-2xl font-bold tracking-tight ${item.direction === "IN" ? "text-green-600" : ""}`}>
                {item.direction === "OUT" ? "-" : "+"}₹
                {parseFloat(item.amount).toLocaleString("en-IN", {
                  minimumFractionDigits: 2,
                  maximumFractionDigits: 2,
                })}
              </div>
            </div>
            <Badge variant={item.direction === "IN" ? "default" : "secondary"}>
              {item.direction === "IN" ? "Income" : "Expense"}
            </Badge>
          </div>

          <Separator className="my-2" />

          <form
            className="flex flex-col gap-4"
            onSubmit={(e) => {
              e.preventDefault();
              onUpdate(item.id, {
                source,
                category,
                status,
                amount,
              });
            }}
          >
            <div className="flex flex-col gap-3">
              <Label htmlFor="source">Source / Description</Label>
              <Input
                id="source"
                value={source}
                onChange={(e) => setSource(e.target.value)}
              />
            </div>

            <div className="flex flex-col gap-3">
              <Label htmlFor="amount">Amount</Label>
              <Input
                id="amount"
                type="number"
                step="0.01"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
              />
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div className="flex flex-col gap-3">
                <Label htmlFor="category">Category</Label>
                <Select value={category} onValueChange={(val) => setCategory(val || "Uncategorized")}>
                  <SelectTrigger id="category" className="w-full">
                    <SelectValue placeholder="Select a category" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="Uncategorized">Uncategorized</SelectItem>
                    <SelectItem value="Food">Food & Alerts</SelectItem>
                    <SelectItem value="Travel">Travel & Auto</SelectItem>
                    <SelectItem value="Shopping">Shopping</SelectItem>
                    <SelectItem value="Entertainment">Entertainment</SelectItem>
                    <SelectItem value="Salary">Salary</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              <div className="flex flex-col gap-3">
                <Label htmlFor="status">Classification Status</Label>
                <Select value={status} onValueChange={(val) => setStatus(val || "Completed")}>
                  <SelectTrigger id="status" className="w-full">
                    <SelectValue placeholder="Select status" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="Completed">Completed</SelectItem>
                    <SelectItem value="Pending">Pending Review</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>

            <div className="flex flex-col gap-3 mt-2">
              <Label htmlFor="note">Personal Note</Label>
              <Input id="note" placeholder="Add a note about this transaction..." />
            </div>

            <DrawerFooter className="mt-auto px-0 pt-6 border-t border-border/50">
              <Button type="submit">Save Changes</Button>
              <DrawerClose asChild>
                <Button variant="outline">Close</Button>
              </DrawerClose>
            </DrawerFooter>
          </form>
        </div>
      </DrawerContent>
    </Drawer>
  );
}
