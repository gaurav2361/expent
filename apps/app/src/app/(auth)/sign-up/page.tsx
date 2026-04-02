import { AuthPage } from "@/components/auth-page";
import { SignUp } from "@/components/signUp";

export default function SignUpPage() {
  return (
    <AuthPage
      author="Sarah Jenkins"
      quote="Setting up my business profile was incredibly intuitive. The best onboarding experience I've had."
    >
      <SignUp />
    </AuthPage>
  );
}
