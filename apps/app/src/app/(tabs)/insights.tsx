import { router } from "expo-router";
import { ChevronLeft, MoreHorizontal } from "lucide-react-native";
import { ScrollView, View } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";
import { Button } from "@/components/ui/button";
import { Text } from "@/components/ui/text";

const CHART_DATA = [
  { day: "Mon", value: 35, color: "hsl(var(--foreground))" },
  { day: "Tue", value: 30, color: "hsl(var(--foreground))" },
  { day: "Wed", value: 20, color: "hsl(var(--foreground))" },
  { day: "Thr", value: 40, color: "#ea7a53", highlight: true },
  { day: "Fri", value: 33, color: "hsl(var(--foreground))" },
  { day: "Sat", value: 18, color: "hsl(var(--foreground))" },
  { day: "Sun", value: 22, color: "hsl(var(--foreground))" },
];

const HISTORY = [
  { id: "1", name: "Claude", price: "$9.84", date: "June 25, 12:00", period: "per month", color: "#f7d44c" },
  { id: "2", name: "Canva", price: "$43.89", date: "June 30, 16:00", period: "per month", color: "#8bcbb8" },
  { id: "3", name: "Grammarly", price: "$16.96", date: "June 24, 13:00", period: "per month", color: "#99b7dd" },
];

export default function InsightsScreen() {
  return (
    <SafeAreaView className="flex-1 bg-background">
      {/* Header */}
      <View className="flex-row items-center justify-between px-4 mt-4 mb-8">
        <Button variant="ghost" size="icon" className="rounded-full" onPress={() => router.back()}>
          <ChevronLeft size={24} color="hsl(var(--foreground))" />
        </Button>
        <Text className="text-xl font-bold text-foreground">Monthly Insights</Text>
        <Button variant="ghost" size="icon" className="rounded-full">
          <MoreHorizontal size={24} color="hsl(var(--foreground))" />
        </Button>
      </View>

      <ScrollView showsVerticalScrollIndicator={false} className="flex-1 px-4">
        {/* Chart Section */}
        <View className="mb-8">
          <View className="flex-row items-center justify-between mb-4">
            <Text className="text-xl font-bold text-foreground">Spending Trend</Text>
            <Button variant="outline" size="sm" className="rounded-full border-border h-9">
              <Text className="text-foreground">View all</Text>
            </Button>
          </View>
          <View className="bg-card border border-border rounded-[28px] p-6 h-[280px] justify-end">
            <View className="flex-row items-end justify-between px-2">
              {CHART_DATA.map((item, index) => (
                <View key={index} className="items-center">
                  {item.highlight && (
                    <View className="bg-primary/10 px-2 py-1 rounded-md mb-2 border border-primary/20">
                      <Text className="text-[10px] font-bold text-primary">$40</Text>
                    </View>
                  )}
                  <View
                    style={{ height: item.value * 4, backgroundColor: item.color }}
                    className="w-3 rounded-full mb-2"
                  />
                  <Text className="text-[10px] font-semibold text-muted-foreground">{item.day}</Text>
                </View>
              ))}
            </View>
          </View>
        </View>

        {/* Expenses Card */}
        <View className="bg-card border border-border rounded-[28px] p-6 mb-8 flex-row items-center justify-between">
          <View>
            <Text className="text-lg font-bold text-foreground">Expenses</Text>
            <Text className="text-sm text-muted-foreground">March 2026</Text>
          </View>
          <View className="items-end">
            <Text className="text-lg font-bold text-foreground">-$424.63</Text>
            <Text className="text-sm text-success-600">+12%</Text>
          </View>
        </View>

        {/* History Section */}
        <View className="mb-24">
          <View className="flex-row items-center justify-between mb-4">
            <Text className="text-xl font-bold text-foreground">History</Text>
            <Button variant="outline" size="sm" className="rounded-full border-border h-9">
              <Text className="text-foreground">View all</Text>
            </Button>
          </View>
          <View className="gap-4">
            {HISTORY.map((sub) => (
              <View
                key={sub.id}
                style={{ backgroundColor: sub.color }}
                className="flex-row items-center justify-between p-5 rounded-[24px]"
              >
                <View className="flex-row items-center gap-4">
                  <View className="w-14 h-14 rounded-[14px] bg-white/30 items-center justify-center">
                    <Text className="text-xl">⚡</Text>
                  </View>
                  <View>
                    <Text className="font-bold text-[#081226] text-lg">{sub.name}</Text>
                    <Text className="text-xs text-[#435875]">{sub.date}</Text>
                  </View>
                </View>
                <View className="items-end">
                  <Text className="font-bold text-[#081226] text-lg">{sub.price}</Text>
                  <Text className="text-xs text-[#435875]">{sub.period}</Text>
                </View>
              </View>
            ))}
          </View>
        </View>
      </ScrollView>
    </SafeAreaView>
  );
}
