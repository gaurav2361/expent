"use client";

import { Button } from "@expent/ui/components/button";
import { InputGroup, InputGroupAddon, InputGroupInput } from "@expent/ui/components/input-group";
import { Link } from "@tanstack/react-router";
import { AtSignIcon, ChevronLeftIcon, SparklesIcon } from "lucide-react";
import { AuthDivider } from "@/components/auth-divider";
import { AuthShades } from "@/components/auth-shades";
import { SocialLogins } from "@/components/auth-social";
import { Logo } from "@/components/logo";

export function SignUp() {
  return (
    <div className="relative flex min-h-screen flex-col justify-center px-8">
      <AuthShades variant="flipped" />
      <Button className="absolute top-7 left-5" variant="ghost" render={<Link to="/" />} nativeButton={false}>
        <ChevronLeftIcon data-icon="inline-start" />
        Home
      </Button>

      <div className="mx-auto space-y-4 sm:w-sm">
        <Logo className="h-4.5 lg:hidden" />
        <div className="flex flex-col space-y-1">
          <div className="flex items-center gap-2 text-primary">
            <SparklesIcon className="h-4 w-4" />
            <span className="font-mono font-medium text-xs uppercase tracking-wider">Start for free</span>
          </div>
          <h1 className="font-bold text-2xl tracking-wide">Create your account</h1>
        </div>

        <form className="space-y-2">
          <InputGroup>
            <InputGroupInput placeholder="your.email@example.com" type="email" />
            <InputGroupAddon align="inline-start">
              <AtSignIcon />
            </InputGroupAddon>
          </InputGroup>

          <Button className="w-full" type="button">
            Create account
          </Button>
        </form>

        <AuthDivider>OR CONTINUE WITH</AuthDivider>

        <SocialLogins />

        <div className="flex flex-col space-y-4 mt-8">
          <p className="text-muted-foreground text-sm text-center">
            By signing up, you agree to our{" "}
            <a className="underline underline-offset-4 hover:text-primary" href="#">
              Terms
            </a>{" "}
            and{" "}
            <a className="underline underline-offset-4 hover:text-primary" href="#">
              Privacy
            </a>
            .
          </p>

          <p className="text-muted-foreground text-sm text-center">
            Already have an account?{" "}
            <Link className="font-semibold text-primary underline underline-offset-4" to="/sign-in">
              Sign in
            </Link>
          </p>
        </div>
      </div>
    </div>
  );
}
