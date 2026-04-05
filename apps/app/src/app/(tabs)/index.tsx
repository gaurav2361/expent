import { View, ScrollView, TouchableOpacity } from 'react-native';
import { Text } from '@/components/ui/text';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { Plus, Wallet, FileText, ArrowUpRight, ArrowDownLeft } from 'lucide-react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { router } from 'expo-router';
import * as Haptics from 'expo-haptics';
import Animated, { FadeInDown } from 'react-native-reanimated';
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar';

const RECENT_ACTIVITY = [
  { id: '1', title: 'Open AI', amount: '-₹3,500.00', type: 'OUT', date: 'Today, 12:30' },
  { id: '2', title: 'Salary Credit', amount: '+₹1,25,000.00', type: 'IN', date: 'Yesterday' },
  { id: '3', title: 'Adobe', amount: '-₹650.00', type: 'OUT', date: '2 days ago' },
];

export default function HomeScreen() {
  const handlePress = () => {
    Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium);
  };

  return (
    <SafeAreaView className="flex-1 bg-background" edges={['top']}>
      <ScrollView 
        showsVerticalScrollIndicator={false} 
        className="flex-1 px-4"
        contentInsetAdjustmentBehavior="automatic"
      >
        {/* Header - User Context */}
        <Animated.View entering={FadeInDown.delay(100)} className="flex-row items-center justify-between mt-4 mb-8">
          <View>
            <Text className="text-2xl font-bold tracking-tight text-foreground">Overview</Text>
            <Text className="text-muted-foreground text-sm">Welcome back, Adrian!</Text>
          </View>
          <TouchableOpacity 
            onPress={() => {
              handlePress();
              router.push('/(tabs)/more');
            }}
          >
            <Avatar alt="Adrian Hajdin's Avatar" className="w-12 h-12 border border-border">
              <AvatarImage 
                source={{ uri: 'https://github.com/adrianhajdin.png' }} 
              />
              <AvatarFallback>
                <Text>AH</Text>
              </AvatarFallback>
            </Avatar>
          </TouchableOpacity>
        </Animated.View>

        {/* Total Balance Card */}
        <Animated.View 
          entering={FadeInDown.delay(200)}
          style={{ borderCurve: 'continuous' }}
          className="bg-primary rounded-[32px] p-6 mb-6 shadow-sm"
        >
          <View className="flex-row items-center justify-between opacity-80 mb-2">
            <Text className="text-primary-foreground font-medium">Total Balance</Text>
            <Wallet size={18} color="white" />
          </View>
          <Text 
            style={{ fontVariant: ['tabular-nums'] }}
            className="text-primary-foreground text-4xl font-bold tracking-tighter"
          >
            ₹1,45,290.53
          </Text>
          <View className="mt-4 pt-4 border-t border-white/10 flex-row justify-between">
            <View>
              <Text className="text-primary-foreground/70 text-xs uppercase tracking-wider">Monthly Spend</Text>
              <Text 
                style={{ fontVariant: ['tabular-nums'] }}
                className="text-primary-foreground font-bold mt-0.5"
              >
                ₹42,200.00
              </Text>
            </View>
            <View className="items-end">
              <Text className="text-primary-foreground/70 text-xs uppercase tracking-wider">P2P Pending</Text>
              <Text className="text-primary-foreground font-bold mt-0.5">3 Requests</Text>
            </View>
          </View>
        </Animated.View>

        {/* Quick Actions */}
        <Animated.View entering={FadeInDown.delay(300)} className="flex-row gap-4 mb-8">
          <Button 
            className="flex-1 h-14 bg-secondary rounded-2xl flex-row items-center gap-2"
            onPress={handlePress}
          >
            <Plus size={20} className="text-secondary-foreground" />
            <Text className="font-bold text-secondary-foreground">Add txn</Text>
          </Button>
          <Button 
            variant="outline"
            className="flex-1 h-14 border-border rounded-2xl flex-row items-center gap-2"
            onPress={handlePress}
          >
            <FileText size={20} className="text-foreground" />
            <Text className="font-bold text-foreground">Upload OCR</Text>
          </Button>
        </Animated.View>

        {/* Recent Activity Section */}
        <View className="mb-8">
          <Animated.View entering={FadeInDown.delay(400)} className="flex-row items-center justify-between mb-4 px-1">
            <Text className="text-xl font-bold text-foreground">Recent Activity</Text>
            <TouchableOpacity onPress={() => router.push('/(tabs)/activity')}>
              <Text className="text-primary font-semibold">View All</Text>
            </TouchableOpacity>
          </Animated.View>
          <View className="gap-3">
            {RECENT_ACTIVITY.map((item, index) => (
              <Animated.View key={item.id} entering={FadeInDown.delay(500 + index * 100)}>
                <Card 
                  style={{ borderCurve: 'continuous' }}
                  className="border-border bg-card/50 rounded-3xl overflow-hidden"
                >
                  <CardContent className="p-4 flex-row items-center justify-between">
                    <View className="flex-row items-center gap-3">
                      <View className={`w-12 h-12 rounded-2xl items-center justify-center ${item.type === 'IN' ? 'bg-success-50' : 'bg-danger-50'}`}>
                        {item.type === 'IN' ? (
                          <ArrowDownLeft size={22} color="#16a34a" />
                        ) : (
                          <ArrowUpRight size={22} color="#dc2626" />
                        )}
                      </View>
                      <View>
                        <Text className="font-bold text-foreground text-base">{item.title}</Text>
                        <Text className="text-xs text-muted-foreground">{item.date}</Text>
                      </View>
                    </View>
                    <Text 
                      style={{ fontVariant: ['tabular-nums'] }}
                      className={`font-bold text-base ${item.type === 'IN' ? 'text-success-600' : 'text-danger-600'}`}
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
          <Text className="text-xl font-bold text-foreground mb-4 px-1">Analytics</Text>
          <TouchableOpacity onPress={() => router.push('/(tabs)/insights')}>
            <Card 
              style={{ borderCurve: 'continuous' }}
              className="bg-card border-border rounded-[32px] p-6 h-48 justify-center items-center border-dashed border-2"
            >
              <Text className="text-muted-foreground font-medium">Spending by Category Chart</Text>
              <Text className="text-xs text-muted-foreground mt-1">Tap to see full breakdown</Text>
            </Card>
          </TouchableOpacity>
        </Animated.View>
      </ScrollView>
    </SafeAreaView>
  );
}
