"use client";

import { Button } from "@expent/ui/components/button";
import { InputGroup, InputGroupAddon, InputGroupInput } from "@expent/ui/components/input-group";
import { Link, useNavigate } from "@tanstack/react-router";
import { AtSignIcon, ChevronLeftIcon } from "lucide-react";
import { AuthDivider } from "@/components/auth-divider";
import { AuthShades } from "@/components/auth-shades";
import { SocialLogins } from "@/components/auth-social";
import { Logo } from "@/components/logo";
import { useState } from "react";
import { authClient } from "@/lib/auth-client";

export function SignIn() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const navigate = useNavigate();

  const handleSignIn = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsLoading(true);
    const { data, error } = await authClient.signIn.email({
      email,
      password,
    });

    setIsLoading(false);
    if (error) {
      alert(error.message || "Failed to sign in");
    } else {
      navigate({ to: "/dashboard" });
    }
  };

  return (
    <div className="relative flex min-h-screen flex-col justify-center px-8">
      <AuthShades />
      <Button className="absolute top-7 left-5" variant="ghost" render={<Link to="/" />} nativeButton={false}>
        <ChevronLeftIcon data-icon="inline-start" />
        Home
      </Button>

      <div className="mx-auto space-y-4 sm:w-sm">
        <Logo className="h-4.5 lg:hidden mx-auto" />
        <div className="flex flex-col space-y-1 text-center">
          <h1 className="font-bold text-2xl tracking-wide">Sign In or Join Now!</h1>
          <p className="text-base text-muted-foreground">login or create your expent account.</p>
        </div>

        <SocialLogins />

        <AuthDivider>OR</AuthDivider>

        <form className="space-y-2 text-center" onSubmit={handleSignIn}>
          <p className="text-muted-foreground text-xs">Enter your credentials to sign in</p>
          <InputGroup>
            <InputGroupInput
              placeholder="your.email@example.com"
              type="email"
              required
              value={email}
              onChange={(e) => setEmail(e.target.value)}
            />
            <InputGroupAddon align="inline-start">
              <AtSignIcon />
            </InputGroupAddon>
          </InputGroup>

          <InputGroup>
            <InputGroupInput
              placeholder="Password"
              type="password"
              required
              value={password}
              onChange={(e) => setPassword(e.target.value)}
            />
          </InputGroup>

          <Button className="w-full" type="submit" disabled={isLoading}>
            {isLoading ? "Signing in..." : "Continue With Email"}
          </Button>
        </form>

        <div className="flex flex-col space-y-4 mt-8 text-center">
          <p className="text-muted-foreground text-sm">
            By clicking continue, you agree to our{" "}
            <a className="underline underline-offset-4 hover:text-primary" href="#">
              Terms of Service
            </a>{" "}
            and{" "}
            <a className="underline underline-offset-4 hover:text-primary" href="#">
              Privacy Policy
            </a>
            .
          </p>

          <p className="text-muted-foreground text-sm">
            New here?{" "}
            <Link className="font-semibold text-primary underline underline-offset-4" to="/sign-up">
              Create an account
            </Link>
          </p>
        </div>
      </div>
    </div>
  );
}
