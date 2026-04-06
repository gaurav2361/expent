import { router } from "expo-router";
import { Bell, ChevronRight, Contact, LogOut, Palette, ShieldCheck, User, Wallet } from "lucide-react-native";
import { ScrollView, TouchableOpacity, View } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";
import { Text } from "@/components/ui/text";

const MENU_GROUPS = [
  {
    title: "Financials",
    items: [
      { id: "wallets", label: "Wallets", icon: Wallet, route: "/(tabs)/wallets" },
      { id: "contacts", label: "Contacts", icon: Contact, route: "/(tabs)/contacts" },
    ],
  },
  {
    title: "Account Settings",
    items: [
      { id: "profile", label: "My Profile", icon: User, route: "/(settings)/profile" },
      { id: "account", label: "Account Security", icon: ShieldCheck, route: "/(settings)/account" },
      { id: "appearance", label: "Appearance", icon: Palette, route: "/(settings)/appearance" },
      { id: "notifications", label: "Notifications", icon: Bell, route: "#" },
    ],
  },
];

export default function MoreScreen() {
  return (
    <SafeAreaView className="flex-1 bg-background" edges={["top"]}>
      <View className="px-4 py-4 mb-4">
        <Text className="text-2xl font-bold text-foreground">More</Text>
      </View>

      <ScrollView showsVerticalScrollIndicator={false} className="flex-1 px-4">
        <View className="gap-8 mb-24">
          {MENU_GROUPS.map((group) => (
            <View key={group.title} className="gap-4">
              <Text className="text-xs font-bold text-muted-foreground uppercase tracking-widest px-1">
                {group.title}
              </Text>
              <Card className="bg-card border-border rounded-[28px] overflow-hidden shadow-none">
                <CardContent className="p-0">
                  {group.items.map((item, index) => (
                    <View key={item.id}>
                      <TouchableOpacity
                        onPress={() => item.route !== "#" && router.push(item.route as any)}
                        className="flex-row items-center justify-between p-5"
                      >
                        <View className="flex-row items-center gap-4">
                          <View className="w-10 h-10 rounded-2xl bg-muted items-center justify-center">
                            <item.icon size={20} className="text-foreground" />
                          </View>
                          <Text className="font-semibold text-foreground text-base">{item.label}</Text>
                        </View>
                        <ChevronRight size={18} className="text-muted-foreground" />
                      </TouchableOpacity>
                      {index < group.items.length - 1 && <Separator className="mx-5" />}
                    </View>
                  ))}
                </CardContent>
              </Card>
            </View>
          ))}

          <Button
            variant="ghost"
            className="h-16 rounded-[28px] flex-row items-center gap-3 border border-border bg-card/50"
            onPress={() => router.replace("/(auth)/sign-in")}
          >
            <LogOut size={20} className="text-destructive" />
            <Text className="font-bold text-destructive">Sign Out</Text>
          </Button>
        </View>
      </ScrollView>
    </SafeAreaView>
  );
}
