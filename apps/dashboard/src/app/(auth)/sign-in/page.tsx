import { AuthPage } from "@/components/auth/auth-page";
import { SignIn } from "@/components/auth/signIn";

export default function SignInPage() {
  return (
    <AuthPage
      author="Ali Hassan"
      quote="This Platform has helped me to save time and serve my clients faster than ever before."
    >
      <SignIn />
    </AuthPage>
  );
}
