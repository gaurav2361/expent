import * as Haptics from "expo-haptics";
import { router } from "expo-router";
import { ArrowDownLeft, ArrowUpRight, Bell, FileText, Plus, Wallet } from "lucide-react-native";
import { ScrollView, TouchableOpacity, View } from "react-native";
import Animated, { FadeIn, FadeInDown } from "react-native-reanimated";
import { SafeAreaView } from "react-native-safe-area-context";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Text } from "@/components/ui/text";
import { useAuth } from "@/lib/auth/use-auth";

const RECENT_ACTIVITY = [
  { id: "1", title: "Cloudflare", amount: "-₹850.00", type: "OUT", date: "Today, 14:20" },
  { id: "2", title: "Salary Credit", amount: "+₹1,45,000.00", type: "IN", date: "Yesterday" },
  { id: "3", title: "GitHub Copilot", amount: "-₹1,650.00", type: "OUT", date: "3 days ago" },
];

export default function HomeScreen() {
  const { user } = useAuth();

  const handlePress = () => {
    Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium);
  };

  const firstName = user?.name?.split(" ")[0] || "Member";
  const initials = user?.name
    ? user.name
        .split(" ")
        .map((n: string) => n[0])
        .join("")
        .toUpperCase()
        .substring(0, 2)
    : user?.email?.substring(0, 2).toUpperCase() || "U";

  return (
    <SafeAreaView className="flex-1 bg-background" edges={["top"]}>
      <ScrollView
        showsVerticalScrollIndicator={false}
        className="flex-1 px-6"
        contentInsetAdjustmentBehavior="automatic"
      >
        {/* Header - User Context */}
        <Animated.View entering={FadeIn.delay(100)} className="flex-row items-center justify-between mt-6 mb-8">
          <View>
            <Text className="text-3xl font-bold tracking-tight text-foreground">Hi, {firstName}!</Text>
            <Text className="text-muted-foreground text-base mt-0.5">Your financial status looks good.</Text>
          </View>
          <View className="flex-row items-center gap-3">
            <TouchableOpacity className="w-10 h-10 items-center justify-center rounded-full bg-muted/50 border border-border/40">
              <Bell size={20} color="hsl(var(--foreground))" />
            </TouchableOpacity>
            <TouchableOpacity
              onPress={() => {
                handlePress();
                router.push("/(tabs)/more");
              }}
            >
              <Avatar alt="User Avatar" className="w-12 h-12 border-2 border-primary/10">
                {user?.image && <AvatarImage source={{ uri: user.image }} />}
                <AvatarFallback>
                  <Text className="font-bold">{initials}</Text>
                </AvatarFallback>
              </Avatar>
            </TouchableOpacity>
          </View>
        </Animated.View>

        {/* Total Balance Card - Matching Dashboard Theme */}
        <Animated.View
          entering={FadeInDown.delay(200)}
          style={{ borderCurve: "continuous" }}
          className="bg-primary rounded-[32px] p-8 mb-8 shadow-xl shadow-primary/20 overflow-hidden"
        >
          <View className="absolute -right-10 -top-10 w-40 h-40 bg-white/5 rounded-full" />

          <View className="flex-row items-center gap-2 mb-3">
            <View className="w-8 h-8 rounded-full bg-white/10 items-center justify-center">
              <Wallet size={14} color="white" />
            </View>
            <Text className="text-primary-foreground/80 font-semibold uppercase tracking-widest text-[10px]">
              Net Balance
            </Text>
          </View>

          <Text
            style={{ fontVariant: ["tabular-nums"] }}
            className="text-primary-foreground text-4xl font-bold tracking-tighter"
          >
            ₹1,82,450.25
          </Text>

          <View className="mt-8 pt-6 border-t border-white/10 flex-row justify-between">
            <View className="gap-1">
              <Text className="text-primary-foreground/60 text-[10px] uppercase font-bold tracking-wider">
                Monthly Spend
              </Text>
              <Text style={{ fontVariant: ["tabular-nums"] }} className="text-primary-foreground text-lg font-bold">
                ₹38,120.00
              </Text>
            </View>
            <View className="items-end gap-1">
              <Text className="text-primary-foreground/60 text-[10px] uppercase font-bold tracking-wider">
                P2P Pending
              </Text>
              <Text className="text-primary-foreground text-lg font-bold">2 Requests</Text>
            </View>
          </View>
        </Animated.View>

        {/* Quick Actions */}
        <Animated.View entering={FadeInDown.delay(300)} className="flex-row gap-4 mb-10">
          <Button
            className="flex-1 h-16 bg-primary rounded-3xl flex-row items-center gap-3 shadow-md shadow-primary/10"
            onPress={handlePress}
          >
            <Plus size={20} color="white" />
            <Text className="font-bold text-primary-foreground text-base">Add Transaction</Text>
          </Button>
          <Button
            variant="outline"
            className="w-16 h-16 border-border/60 rounded-3xl items-center justify-center bg-card/50"
            onPress={handlePress}
          >
            <FileText size={22} className="text-foreground" />
          </Button>
        </Animated.View>

        {/* Recent Activity Section */}
        <View className="mb-10">
          <Animated.View entering={FadeInDown.delay(400)} className="flex-row items-center justify-between mb-5 px-1">
            <Text className="text-2xl font-bold text-foreground tracking-tight">Recent Activity</Text>
            <TouchableOpacity onPress={() => router.push("/(tabs)/activity")}>
              <Text className="text-primary font-bold text-sm">View All</Text>
            </TouchableOpacity>
          </Animated.View>

          <View className="gap-4">
            {RECENT_ACTIVITY.map((item, index) => (
              <Animated.View key={item.id} entering={FadeInDown.delay(500 + index * 100)}>
                <Card
                  style={{ borderCurve: "continuous" }}
                  className="border-border/40 bg-card/40 rounded-3xl overflow-hidden border"
                >
                  <CardContent className="p-5 flex-row items-center justify-between">
                    <View className="flex-row items-center gap-4">
                      <View
                        className={`w-12 h-12 rounded-2xl items-center justify-center ${item.type === "IN" ? "bg-success-100/50" : "bg-danger-100/50"}`}
                      >
                        {item.type === "IN" ? (
                          <ArrowDownLeft size={22} color="hsl(var(--success-600))" />
                        ) : (
                          <ArrowUpRight size={22} color="hsl(var(--danger-600))" />
                        )}
                      </View>
                      <View>
                        <Text className="font-bold text-foreground text-lg tracking-tight">{item.title}</Text>
                        <Text className="text-xs text-muted-foreground mt-0.5">{item.date}</Text>
                      </View>
                    </View>
                    <Text
                      style={{ fontVariant: ["tabular-nums"] }}
                      className={`font-bold text-lg ${item.type === "IN" ? "text-success-600" : "text-danger-600"}`}
                    >
                      {item.amount}
                    </Text>
                  </CardContent>
                </Card>
              </Animated.View>
            ))}
          </View>
        </View>

        {/* Charts Preview */}
        <Animated.View entering={FadeInDown.delay(800)} className="mb-24">
          <Text className="text-2xl font-bold text-foreground tracking-tight mb-5 px-1">Insights</Text>
          <TouchableOpacity onPress={() => router.push("/(tabs)/insights")}>
            <Card
              style={{ borderCurve: "continuous" }}
              className="bg-card/30 border-border/40 rounded-[32px] p-8 h-56 justify-center items-center border-dashed border-2"
            >
              <View className="w-16 h-16 rounded-full bg-primary/5 items-center justify-center mb-4">
                <ArrowUpRight size={28} className="text-primary/40" />
              </View>
              <Text className="text-foreground font-bold text-lg">Spending Analysis</Text>
              <Text className="text-sm text-muted-foreground mt-1 text-center">
                See where your money goes every month
              </Text>
            </Card>
          </TouchableOpacity>
        </Animated.View>
      </ScrollView>
    </SafeAreaView>
  );
}
