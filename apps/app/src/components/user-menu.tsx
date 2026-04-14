import type { TriggerRef } from "@rn-primitives/popover";
import { router } from "expo-router";
import { LogOutIcon, PlusIcon, SettingsIcon } from "lucide-react-native";
import * as React from "react";
import { View } from "react-native";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import { Icon } from "@/components/ui/icon";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { Text } from "@/components/ui/text";
import { useAuth } from "@/lib/auth/use-auth";
import { cn } from "@/lib/utils";

export function UserMenu() {
  const { user, signOut } = useAuth();
  const popoverTriggerRef = React.useRef<TriggerRef>(null);

  async function onSignOut() {
    popoverTriggerRef.current?.close();
    try {
      await signOut();
      router.replace("/(auth)/sign-in");
    } catch (error) {
      console.error("Sign out failed:", error);
    }
  }

  const initials = user?.name
    ? user.name
        .split(" ")
        .map((n: string) => n[0])
        .join("")
        .toUpperCase()
        .substring(0, 2)
    : user?.email?.substring(0, 2).toUpperCase() || "U";

  return (
    <Popover>
      <PopoverTrigger asChild ref={popoverTriggerRef}>
        <Button variant="ghost" size="icon" className="size-8 rounded-full">
          <UserAvatar user={user} initials={initials} />
        </Button>
      </PopoverTrigger>
      <PopoverContent align="center" side="bottom" className="w-80 p-0">
        <View className="border-border gap-3 border-b p-3">
          <View className="flex-row items-center gap-3">
            <UserAvatar className="size-10" user={user} initials={initials} />
            <View className="flex-1">
              <Text className="font-medium leading-5">{user?.name || "Member"}</Text>
              <Text className="text-muted-foreground text-sm font-normal leading-4">{user?.email}</Text>
            </View>
          </View>
          <View className="flex-row flex-wrap gap-3 py-0.5">
            <Button
              variant="outline"
              size="sm"
              onPress={() => {
                router.push("/(settings)" as any);
              }}
            >
              <Icon as={SettingsIcon} className="size-4" />
              <Text>Settings</Text>
            </Button>
            <Button variant="outline" size="sm" className="flex-1" onPress={onSignOut}>
              <Icon as={LogOutIcon} className="size-4" />
              <Text>Sign Out</Text>
            </Button>
          </View>
        </View>
        <Button
          variant="ghost"
          size="lg"
          className="h-16 justify-start gap-3 rounded-none rounded-b-md px-3 sm:h-14"
          onPress={() => {
            popoverTriggerRef.current?.close();
            router.push("/(auth)/sign-in" as any);
          }}
        >
          <View className="size-10 items-center justify-center">
            <View className="border-border bg-muted/50 size-7 items-center justify-center rounded-full border border-dashed">
              <Icon as={PlusIcon} className="size-5" />
            </View>
          </View>
          <Text>Add account</Text>
        </Button>
      </PopoverContent>
    </Popover>
  );
}

function UserAvatar({ className, user, initials, ...props }: any) {
  return (
    <Avatar alt={`${user?.name || "User"}'s avatar`} className={cn("size-8", className)} {...props}>
      {user?.image && <AvatarImage source={{ uri: user.image }} />}
      <AvatarFallback>
        <Text>{initials}</Text>
      </AvatarFallback>
    </Avatar>
  );
}
