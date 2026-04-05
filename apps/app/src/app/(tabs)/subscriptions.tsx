import { View } from 'react-native';
import { Text } from '@/components/ui/text';
import { Button } from '@/components/ui/button';
import { ChevronLeft, MoreHorizontal } from 'lucide-react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { router } from 'expo-router';
import { FlashList } from '@shopify/flash-list';
import * as Haptics from 'expo-haptics';
import { Badge } from '@/components/ui/badge';

const SUBSCRIPTIONS = [
  { id: '1', name: 'Dropbox', plan: 'Premium', price: '$16.96', period: '1 month', color: 'transparent', borderColor: 'hsl(var(--border))' },
  { id: '2', name: 'Spotify', plan: 'Family Plan', price: '$76.77', period: '3 months', color: 'transparent', borderColor: 'hsl(var(--border))' },
  { 
    id: '3', 
    name: 'GitHub Copilot', 
    plan: 'Pro Business', 
    price: '$49.99', 
    period: '1 month', 
    color: '#8bcbb8',
    isExpanded: true,
    paymentInfo: '*****8530',
    planDetails: 'Premium'
  },
  { id: '4', name: 'Adobe', plan: 'Family Plan', price: '$98.10', period: '2 months', color: 'transparent', borderColor: 'hsl(var(--border))' },
  { id: '5', name: 'Figma', plan: 'Premium', price: '$19.23', period: '1 month', color: 'transparent', borderColor: 'hsl(var(--border))' },
];

export default function SubscriptionsScreen() {
  const handlePress = () => {
    Haptics.selectionAsync();
  };

  return (
    <SafeAreaView className="flex-1 bg-background" edges={['top']}>
      {/* Header */}
      <View className="flex-row items-center justify-between px-4 py-4 mb-4">
        <Button 
          variant="ghost" 
          size="icon" 
          className="rounded-full"
          onPress={() => {
            handlePress();
            router.back();
          }}
        >
          <ChevronLeft size={24} color="hsl(var(--foreground))" />
        </Button>
        <Text className="text-xl font-bold text-foreground">Subscriptions</Text>
        <Button variant="ghost" size="icon" className="rounded-full" onPress={handlePress}>
          <MoreHorizontal size={24} color="hsl(var(--foreground))" />
        </Button>
      </View>

      <View className="flex-1 px-4">
        <FlashList
          data={SUBSCRIPTIONS}
          estimatedItemSize={100}
          keyExtractor={(item) => item.id}
          showsVerticalScrollIndicator={false}
          contentContainerStyle={{ paddingBottom: 100 }}
          renderItem={({ item: sub }) => (
            <View 
              style={{ 
                backgroundColor: sub.color === 'transparent' ? undefined : sub.color,
                borderWidth: sub.borderColor ? 1 : 0,
                borderColor: sub.borderColor,
                borderCurve: 'continuous'
              }} 
              className={`rounded-[28px] p-5 mb-4 ${sub.color === 'transparent' ? 'bg-card' : ''}`}
            >
              <View className="flex-row items-center justify-between">
                <View className="flex-row items-center gap-4">
                  <View className={`w-14 h-14 rounded-2xl items-center justify-center ${sub.color === 'transparent' ? 'bg-muted' : 'bg-white/30'}`}>
                    <Text className="text-xl">📁</Text>
                  </View>
                  <View>
                    <Text className={`font-bold text-lg ${sub.isExpanded ? 'text-[#081226]' : 'text-foreground'}`}>
                      {sub.name}
                    </Text>
                    <Badge variant={sub.isExpanded ? "secondary" : "outline"} className="mt-1">
                      <Text className={sub.isExpanded ? "text-[#081226]" : ""}>{sub.plan}</Text>
                    </Badge>
                  </View>
                </View>
                <View className="items-end">
                  <Text 
                    style={{ fontVariant: ['tabular-nums'] }}
                    className={`font-bold text-lg ${sub.isExpanded ? 'text-[#081226]' : 'text-foreground'}`}
                  >
                    {sub.price}
                  </Text>
                  <Text className={`text-xs ${sub.isExpanded ? 'text-[#435875]' : 'text-muted-foreground'}`}>
                    {sub.period}
                  </Text>
                </View>
              </View>

              {sub.isExpanded && (
                <View className="mt-6 gap-6">
                  <View className="flex-row items-center justify-between">
                    <View className="flex-row items-center gap-2">
                      <Text className="text-[#435875]">Payment info:</Text>
                      <Text 
                        style={{ fontVariant: ['tabular-nums'] }}
                        className="font-bold text-[#081226]"
                      >
                        {sub.paymentInfo}
                      </Text>
                    </View>
                    <Button variant="outline" className="rounded-full border-[#081226] h-10 px-4" onPress={handlePress}>
                      <Text className="text-xs font-bold text-[#081226]">Manage</Text>
                    </Button>
                  </View>

                  <View className="flex-row items-center justify-between">
                    <View className="flex-row items-center gap-2">
                      <Text className="text-[#435875]">Plan details:</Text>
                      <Text className="font-bold text-[#081226]">{sub.planDetails}</Text>
                    </View>
                    <Button variant="outline" className="rounded-full border-[#081226] h-10 px-4" onPress={handlePress}>
                      <Text className="text-xs font-bold text-[#081226]">Change</Text>
                    </Button>
                  </View>

                  <Button className="bg-[#081226] rounded-full py-4 mt-2" onPress={handlePress}>
                    <Text className="text-white font-bold">Cancel Subscription</Text>
                  </Button>
                </View>
              )}
            </View>
          )}
        />
      </View>
    </SafeAreaView>
  );
}
