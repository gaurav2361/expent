import { AuthPage } from "@/components/auth-page";
import { SignIn } from "@/components/signIn";

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
