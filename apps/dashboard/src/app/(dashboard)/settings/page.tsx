"use client";

import { useSession } from "@/lib/auth-client";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@expent/ui/components/card";
import { Button } from "@expent/ui/components/button";
import { Separator } from "@expent/ui/components/separator";
import { UserIcon, BellIcon, ShieldIcon } from "lucide-react";
import { useTheme } from "next-themes";
import { PreferencesPanel } from "@/components/tool-ui/preferences-panel";

export default function SettingsPage() {
  const session = useSession();
  const { theme, setTheme } = useTheme();

  const user = session.data?.user;

  return (
    <div className="flex flex-1 flex-col gap-6 p-4 md:p-6 lg:p-8 max-w-3xl mx-auto w-full">
      {/* Profile */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-3">
            <div className="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary">
              <UserIcon className="h-5 w-5" />
            </div>
            <div>
              <CardTitle>Profile</CardTitle>
              <CardDescription>Manage your account information</CardDescription>
            </div>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <p className="text-sm font-medium text-muted-foreground">Name</p>
              <p className="text-sm">{user?.name || "—"}</p>
            </div>
            <div>
              <p className="text-sm font-medium text-muted-foreground">Email</p>
              <p className="text-sm">{user?.email || "—"}</p>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Appearance using PreferencesPanel */}
      <PreferencesPanel
        id="appearance-settings"
        title="Appearance"
        sections={[
          {
            items: [
              {
                id: "theme",
                type: "toggle",
                label: "Theme",
                description: "Select your preferred color theme.",
                options: [
                  { value: "light", label: "Light" },
                  { value: "dark", label: "Dark" },
                  { value: "system", label: "System" },
                ],
                defaultValue: theme,
              },
            ],
          },
        ]}
        onAction={(actionId, values) => {
          if (actionId === "save") {
            setTheme(values.theme as string);
          }
        }}
      />

      {/* Notifications */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-3">
            <div className="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary">
              <BellIcon className="h-5 w-5" />
            </div>
            <div>
              <CardTitle>Notifications</CardTitle>
              <CardDescription>Configure how you receive alerts</CardDescription>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground">Notification preferences coming soon.</p>
        </CardContent>
      </Card>

      {/* Security */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-3">
            <div className="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary">
              <ShieldIcon className="h-5 w-5" />
            </div>
            <div>
              <CardTitle>Security</CardTitle>
              <CardDescription>Manage your password and sessions</CardDescription>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground">Security settings coming soon.</p>
        </CardContent>
      </Card>

      <Separator />

      <div className="flex justify-end pb-8">
        <Button
          variant="destructive"
          size="sm"
          onClick={() => {
            window.location.href = "/sign-in";
          }}
        >
          Sign Out
        </Button>
      </div>
    </div>
  );
}
