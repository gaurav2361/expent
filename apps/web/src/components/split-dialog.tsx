import { useState } from "react";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@expent/ui/components/dialog";
import { Button } from "@expent/ui/components/button";
import { Input } from "@expent/ui/components/input";
import { Label } from "@expent/ui/components/label";
import { PlusIcon, Trash2Icon } from "lucide-react";
import { useMutation, useQueryClient } from "@tanstack/react-query";

interface SplitDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    transactionId: string;
    totalAmount: string;
}

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || "http://localhost:3001";

export function SplitDialog({ open, onOpenChange, transactionId, totalAmount }: SplitDialogProps) {
    const queryClient = useQueryClient();
    const [splits, setSplits] = useState<{ receiver_email: string; amount: string }[]>([
        { receiver_email: "", amount: "" }
    ]);

    const splitMutation = useMutation({
        mutationFn: async () => {
            const response = await fetch(`${API_BASE_URL}/api/transactions/split`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({
                    transaction_id: transactionId,
                    splits: splits.map(s => ({ 
                        receiver_email: s.receiver_email, 
                        amount: s.amount 
                    }))
                }),
                credentials: "include",
            });
            if (!response.ok) throw new Error("Failed to split transaction");
            return response.json();
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ["p2p-pending"] });
            onOpenChange(false);
            alert("Splits created successfully!");
        }
    });

    const addSplit = () => setSplits([...splits, { receiver_email: "", amount: "" }]);
    const removeSplit = (index: number) => setSplits(splits.filter((_, i) => i !== index));
    const updateSplit = (index: number, field: "receiver_email" | "amount", value: string) => {
        const newSplits = [...splits];
        newSplits[index][field] = value;
        setSplits(newSplits);
    };

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="sm:max-w-[425px]">
                <DialogHeader>
                    <DialogTitle>Split Transaction</DialogTitle>
                    <DialogDescription>
                        Divide ₹{parseFloat(totalAmount).toLocaleString()} among your contacts.
                    </DialogDescription>
                </DialogHeader>
                <div className="grid gap-4 py-4">
                    {splits.map((split, index) => (
                        <div key={index} className="flex gap-2 items-end border-b pb-4 last:border-0">
                            <div className="grid gap-2 flex-1">
                                <Label htmlFor={`email-${index}`}>Email</Label>
                                <Input 
                                    id={`email-${index}`} 
                                    placeholder="friend@example.com" 
                                    value={split.receiver_email}
                                    onChange={(e) => updateSplit(index, "receiver_email", e.target.value)}
                                />
                            </div>
                            <div className="grid gap-2 w-24">
                                <Label htmlFor={`amount-${index}`}>Amount</Label>
                                <Input 
                                    id={`amount-${index}`} 
                                    placeholder="0.00" 
                                    value={split.amount}
                                    onChange={(e) => updateSplit(index, "amount", e.target.value)}
                                />
                            </div>
                            <Button 
                                variant="ghost" 
                                size="icon" 
                                className="text-destructive" 
                                onClick={() => removeSplit(index)}
                                disabled={splits.length === 1}
                            >
                                <Trash2Icon className="h-4 w-4" />
                            </Button>
                        </div>
                    ))}
                    <Button variant="outline" size="sm" onClick={addSplit} className="w-full">
                        <PlusIcon className="h-4 w-4 mr-2" /> Add Person
                    </Button>
                </div>
                <DialogFooter>
                    <Button variant="outline" onClick={() => onOpenChange(false)}>Cancel</Button>
                    <Button 
                        onClick={() => splitMutation.mutate()} 
                        disabled={splitMutation.isPending || splits.some(s => !s.receiver_email || !s.amount)}
                    >
                        {splitMutation.isPending ? "Splitting..." : "Send Split Requests"}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
