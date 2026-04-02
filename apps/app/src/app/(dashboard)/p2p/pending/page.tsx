export default function PendingPage() {
  return (
    <div className="flex flex-1 flex-col gap-4 p-4 lg:p-8">
      <div className="flex items-center justify-between">
        <h1 className="font-semibold text-lg md:text-2xl">Pending Requests</h1>
      </div>
      <div className="flex flex-1 items-center justify-center rounded-lg border border-dashed shadow-xs">
        <div className="flex flex-col items-center gap-1 text-center">
          <h3 className="font-bold text-2xl tracking-tight">No Pending Requests</h3>
          <p className="text-sm text-muted-foreground">
            You don't have any pending requests to approve right now.
          </p>
        </div>
      </div>
    </div>
  );
}
