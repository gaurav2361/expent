import { createFileRoute } from "@tanstack/react-router";
import { AuthPage } from "@/components/auth-page";
import { SignIn } from "@/components/signIn";

export const Route = createFileRoute("/sign-in")({
  component: RouteComponent,
});

function RouteComponent() {
  return (
    <AuthPage>
      <SignIn />
    </AuthPage>
  );
}
