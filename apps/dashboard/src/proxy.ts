import { NextResponse } from "next/server";
import type { NextRequest } from "next/server";

const publicRoutes = ["/sign-in", "/sign-up"];

export async function proxy(request: NextRequest) {
  const { pathname } = request.nextUrl;

  // Exclude static paths and api
  if (pathname.startsWith("/_next") || pathname.startsWith("/api") || pathname.includes(".")) {
    return NextResponse.next();
  }

  // better-auth cookie names vary:
  //   Development (HTTP):  better-auth.session_token
  //   Production (HTTPS):  __Secure-better-auth.session_token
  // Check all cookies for any that contain "session_token" as a fallback
  const allCookies = request.cookies.getAll();
  const sessionToken = allCookies.find(
    (c) =>
      c.name === "better-auth.session-token" ||
      c.name === "__Secure-better-auth.session-token" ||
      c.name.includes("session-token")
  );

  const isPublicRoute = publicRoutes.includes(pathname);


  if (!sessionToken && !isPublicRoute) {
    return NextResponse.redirect(new URL("/sign-in", request.url));
  }

  if (sessionToken && isPublicRoute) {
    return NextResponse.redirect(new URL("/", request.url));
  }

  return NextResponse.next();
}

export const config = {
  matcher: ["/((?!api|_next/static|_next/image|favicon.ico).*)"],
};
