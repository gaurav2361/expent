import { View, ScrollView, TouchableOpacity } from 'react-native';
import { Text } from '@/components/ui/text';
import { Button } from '@/components/ui/button';
import { ChevronLeft, MoreHorizontal, Calendar, CreditCard, Sparkles, AlertCircle } from 'lucide-react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { router } from 'expo-router';
import { FlashList } from '@shopify/flash-list';
import * as Haptics from 'expo-haptics';
import { Badge } from '@/components/ui/badge';
import Animated, { FadeIn } from 'react-native-reanimated';

const SUBSCRIPTIONS = [
  { id: '1', name: 'Dropbox', plan: 'Premium', price: '$16.96', period: 'monthly', color: 'transparent', icon: '📦' },
  { id: '2', name: 'Spotify', plan: 'Family', price: '$76.77', period: '3 months', color: 'transparent', icon: '🎵' },
  { 
    id: '3', 
    name: 'GitHub Copilot', 
    plan: 'Pro Business', 
    price: '$49.99', 
    period: 'monthly', 
    color: 'hsl(var(--primary))',
    isExpanded: true,
    paymentInfo: '•••• 8530',
    planDetails: 'Enterprise Tier',
    icon: '🤖'
  },
  { id: '4', name: 'Adobe CC', plan: 'All Apps', price: '$98.10', period: 'monthly', color: 'transparent', icon: '🎨' },
  { id: '5', name: 'Figma', plan: 'Professional', price: '$15.00', period: 'monthly', color: 'transparent', icon: '🖋️' },
];

export default function SubscriptionsScreen() {
  const handlePress = () => {
    Haptics.selectionAsync();
  };

  return (
    <SafeAreaView className="flex-1 bg-background" edges={['top']}>
      <Animated.View entering={FadeIn} className="flex-1">
        {/* Header */}
        <View className="flex-row items-center justify-between px-6 py-6">
          <TouchableOpacity 
            className="w-10 h-10 items-center justify-center rounded-full bg-muted/50 border border-border/40"
            onPress={() => {
              handlePress();
              router.back();
            }}
          >
            <ChevronLeft size={24} color="hsl(var(--foreground))" />
          </TouchableOpacity>
          <Text className="text-xl font-bold text-foreground">Subscriptions</Text>
          <TouchableOpacity 
            className="w-10 h-10 items-center justify-center rounded-full bg-muted/50 border border-border/40"
            onPress={handlePress}
          >
            <MoreHorizontal size={24} color="hsl(var(--foreground))" />
          </TouchableOpacity>
        </View>

        <View className="px-6 mb-6">
          <View className="bg-primary/5 border border-primary/10 rounded-3xl p-5 flex-row items-center gap-4">
            <View className="w-12 h-12 rounded-2xl bg-primary/10 items-center justify-center">
              <Sparkles size={24} className="text-primary" />
            </View>
            <View className="flex-1">
              <Text className="font-bold text-foreground">Scan for Subscriptions</Text>
              <Text className="text-xs text-muted-foreground mt-0.5">Let AI detect recurring payments</Text>
            </View>
            <Button variant="outline" size="sm" className="rounded-xl h-9 border-primary/20">
              <Text className="text-primary font-bold text-xs">Start</Text>
            </Button>
          </View>
        </View>

        <View className="flex-1 px-6">
          <FlashList
            data={SUBSCRIPTIONS}
            estimatedItemSize={120}
            keyExtractor={(item) => item.id}
            showsVerticalScrollIndicator={false}
            contentContainerStyle={{ paddingBottom: 100 }}
            renderItem={({ item: sub }) => {
              const isDark = sub.color !== 'transparent';
              return (
                <View 
                  style={{ 
                    backgroundColor: sub.color === 'transparent' ? 'transparent' : sub.color,
                    borderWidth: 1,
                    borderColor: 'hsl(var(--border)/40)',
                    borderCurve: 'continuous'
                  }} 
                  className={`rounded-[32px] p-6 mb-4 ${sub.color === 'transparent' ? 'bg-card/40' : ''} shadow-sm shadow-black/5`}
                >
                  <View className="flex-row items-center justify-between">
                    <View className="flex-row items-center gap-4">
                      <View className={`w-14 h-14 rounded-2xl items-center justify-center ${isDark ? 'bg-white/20' : 'bg-muted/50'}`}>
                        <Text className="text-2xl">{sub.icon}</Text>
                      </View>
                      <View>
                        <Text className={`font-bold text-xl tracking-tight ${isDark ? 'text-primary-foreground' : 'text-foreground'}`}>
                          {sub.name}
                        </Text>
                        <View className="flex-row items-center gap-2 mt-1">
                          <Badge variant={isDark ? "secondary" : "outline"} className="h-5 px-2">
                            <Text className={`text-[10px] font-bold ${isDark ? 'text-primary' : ''}`}>{sub.plan}</Text>
                          </Badge>
                        </View>
                      </View>
                    </View>
                    <View className="items-end">
                      <Text 
                        style={{ fontVariant: ['tabular-nums'] }}
                        className={`font-bold text-lg ${isDark ? 'text-primary-foreground' : 'text-foreground'}`}
                      >
                        {sub.price}
                      </Text>
                      <Text className={`text-[10px] font-bold uppercase tracking-wider ${isDark ? 'text-primary-foreground/60' : 'text-muted-foreground'}`}>
                        {sub.period}
                      </Text>
                    </View>
                  </View>

                  {sub.isExpanded && (
                    <Animated.View entering={FadeIn} className="mt-8 gap-6 pt-6 border-t border-white/10">
                      <View className="flex-row items-center justify-between">
                        <View className="gap-1">
                          <Text className={`${isDark ? 'text-primary-foreground/60' : 'text-muted-foreground'} text-[10px] font-bold uppercase tracking-widest`}>Payment Method</Text>
                          <View className="flex-row items-center gap-2">
                            <CreditCard size={14} color={isDark ? "white" : "black"} opacity={0.6} />
                            <Text className={`font-bold ${isDark ? 'text-primary-foreground' : 'text-foreground'}`}>
                              {sub.paymentInfo}
                            </Text>
                          </View>
                        </View>
                        <Button variant="outline" className={`rounded-xl h-10 px-4 border-white/20 ${isDark ? 'bg-white/10' : ''}`} onPress={handlePress}>
                          <Text className={`text-xs font-bold ${isDark ? 'text-primary-foreground' : 'text-foreground'}`}>Manage</Text>
                        </Button>
                      </View>

                      <View className="flex-row items-center justify-between">
                        <View className="gap-1">
                          <Text className={`${isDark ? 'text-primary-foreground/60' : 'text-muted-foreground'} text-[10px] font-bold uppercase tracking-widest`}>Next Renewal</Text>
                          <View className="flex-row items-center gap-2">
                            <Calendar size={14} color={isDark ? "white" : "black"} opacity={0.6} />
                            <Text className={`font-bold ${isDark ? 'text-primary-foreground' : 'text-foreground'}`}>April 12, 2026</Text>
                          </View>
                        </View>
                        <View className={`flex-row items-center gap-1 px-2 py-1 rounded-lg ${isDark ? 'bg-white/10' : 'bg-primary/5'}`}>
                          <AlertCircle size={12} color={isDark ? "white" : "hsl(var(--primary))"} />
                          <Text className={`text-[10px] font-bold ${isDark ? 'text-primary-foreground' : 'text-primary'}`}>Auto-pay on</Text>
                        </View>
                      </View>

                      <Button className={`rounded-2xl py-4 mt-2 h-14 ${isDark ? 'bg-white' : 'bg-primary'}`} onPress={handlePress}>
                        <Text className={`font-bold text-base ${isDark ? 'text-primary' : 'text-primary-foreground'}`}>Cancel Subscription</Text>
                      </Button>
                    </Animated.View>
                  )}
                </View>
              );
            }}
          />
        </View>
      </Animated.View>
    </SafeAreaView>
  );
}
