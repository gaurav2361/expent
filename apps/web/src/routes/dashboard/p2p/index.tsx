import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/dashboard/p2p/")({
  component: RouteComponent,
});

function RouteComponent() {
  return <div>Hello "/dashboard/p2p/"!</div>;
}
