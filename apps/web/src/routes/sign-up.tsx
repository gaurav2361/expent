import { createFileRoute } from "@tanstack/react-router";
import { AuthPage } from "@/components/auth-page";
import { SignUp } from "@/components/signUp";

export const Route = createFileRoute("/sign-up")({
  component: RouteComponent,
});

function RouteComponent() {
  return (
    <AuthPage>
      <SignUp />
    </AuthPage>
  );
}
