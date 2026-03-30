import { createFileRoute } from "@tanstack/react-router";
import { AuthPage } from "@/components/auth-page";
import { SignUp } from "@/components/signUp";

export const Route = createFileRoute("/sign-up")({
  component: RouteComponent,
});

function RouteComponent() {
  return (
    <AuthPage
      author="Sarah Jenkins"
      quote="Setting up my business profile was incredibly intuitive. The best onboarding experience I've had."
    >
      <SignUp />
    </AuthPage>
  );
}
